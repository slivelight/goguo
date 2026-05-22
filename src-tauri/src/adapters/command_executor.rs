//! Command executor abstraction for remote adapter bridge operations.
//!
//! Provides a trait for executing commands, reading/writing files, and querying
//! environment info — abstracted so that remote bridges (`WslBridge`, `PowershellBridge`)
//! can be tested with mock executors.

use std::path::PathBuf;

// ---------------------------------------------------------------------------
// CommandExecutor trait
// ---------------------------------------------------------------------------

/// Abstraction over command execution and file I/O for adapter bridge operations.
///
/// Remote adapters (`WslRemoteAdapter`, `WindowsRemoteAdapter`) use this trait to
/// perform all I/O, allowing full testability via `MockCommandExecutor`.
pub(crate) trait CommandExecutor: Send + Sync {
    /// Execute a program with arguments and return its stdout.
    fn execute(&self, program: &str, args: &[&str]) -> Result<String, String>;

    /// Query an environment variable.
    fn env_var(&self, key: &str) -> Option<String>;

    /// Return the current user's home directory.
    fn home_dir(&self) -> Option<PathBuf>;

    /// Read a file's contents as a string.
    fn read_file(&self, path: &str) -> Result<String, String>;

    /// Write content to a file.
    fn write_file(&self, path: &str, content: &str) -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// SystemCommandExecutor (local, production)
// ---------------------------------------------------------------------------

/// Production executor using `std::process::Command` and `std::fs`.
/// Delegates directly to the local system — used by local adapters.
pub struct SystemCommandExecutor;

impl CommandExecutor for SystemCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let output = std::process::Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute {program}: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Command {program} failed: {stderr}"))
        }
    }

    fn env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .map(PathBuf::from)
            .ok()
    }

    fn read_file(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read {path}: {e}"))
    }

    fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        std::fs::write(path, content).map_err(|e| format!("Failed to write {path}: {e}"))
    }
}

// ---------------------------------------------------------------------------
// WslBridgeExecutor (Windows → WSL)
// ---------------------------------------------------------------------------

/// Executor that bridges commands from Windows to WSL via `wsl -e`.
///
/// All `execute()` calls are prefixed with `wsl -e`.
/// File operations use `wsl -e cat` / `wsl -e bash -c "echo ... | sudo tee ..."`.
pub struct WslBridgeExecutor {
    distro: Option<String>,
}

impl WslBridgeExecutor {
    #[must_use]
    pub const fn new() -> Self {
        Self { distro: None }
    }

    /// Specify a WSL distro name for `wsl -d <distro>`.
    #[must_use]
    pub fn with_distro(mut self, distro: impl Into<String>) -> Self {
        self.distro = Some(distro.into());
        self
    }

    fn wsl_args(&self, extra: &[&str]) -> Vec<String> {
        let mut args = Vec::new();
        if let Some(ref d) = self.distro {
            args.push("-d".to_string());
            args.push(d.clone());
        }
        args.push("-e".to_string());
        args.extend(extra.iter().map(|s| (*s).to_string()));
        args
    }
}

impl Default for WslBridgeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

impl CommandExecutor for WslBridgeExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let mut all_args = vec![program];
        all_args.extend(args);
        let str_args: Vec<&str> = all_args.iter().map(|s| &**s).collect();
        let wsl_args = self.wsl_args(&str_args);
        let output = std::process::Command::new("wsl")
            .args(&wsl_args)
            .output()
            .map_err(|e| format!("Failed to execute wsl: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("wsl command failed: {stderr}"))
        }
    }

    fn env_var(&self, key: &str) -> Option<String> {
        // Query WSL env via wsl -e printenv
        let output = std::process::Command::new("wsl")
            .args(self.wsl_args(&["printenv", key]))
            .output()
            .ok()?;
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if val.is_empty() {
                None
            } else {
                Some(val)
            }
        } else {
            None
        }
    }

    fn home_dir(&self) -> Option<PathBuf> {
        let output = std::process::Command::new("wsl")
            .args(self.wsl_args(&["bash", "-c", "echo $HOME"]))
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(PathBuf::from(path))
        } else {
            None
        }
    }

    fn read_file(&self, path: &str) -> Result<String, String> {
        self.execute("cat", &[path])
    }

    fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        // Use sudo tee for files that may need root (e.g., /etc/resolv.conf)
        let cmd = format!("echo '{}'", content.replace('\'', "'\\''"));
        let tee_cmd = format!("{cmd} | sudo tee '{path}' > /dev/null");
        self.execute("bash", &["-c", &tee_cmd])?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PowershellBridgeExecutor (WSL → Windows)
// ---------------------------------------------------------------------------

/// Executor that bridges commands from WSL/Linux to Windows via `powershell.exe`.
///
/// `execute()` wraps commands in `powershell.exe -Command "..."`.
/// File operations use `/mnt/c/` path mapping for direct access.
pub struct PowershellBridgeExecutor;

impl CommandExecutor for PowershellBridgeExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let mut cmd_str = program.to_string();
        for arg in args {
            cmd_str.push(' ');
            if arg.contains(' ') || arg.contains('"') {
                cmd_str.push('\'');
                cmd_str.push_str(arg);
                cmd_str.push('\'');
            } else {
                cmd_str.push_str(arg);
            }
        }
        let output = std::process::Command::new("powershell.exe")
            .args(["-Command", &cmd_str])
            .output()
            .map_err(|e| format!("Failed to execute powershell.exe: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("powershell.exe command failed: {stderr}"))
        }
    }

    fn env_var(&self, key: &str) -> Option<String> {
        let output = std::process::Command::new("powershell.exe")
            .args(["-Command", &format!("echo $env:{key}")])
            .output()
            .ok()?;
        if output.status.success() {
            let val = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if val.is_empty() {
                None
            } else {
                Some(val)
            }
        } else {
            None
        }
    }

    fn home_dir(&self) -> Option<PathBuf> {
        let output = std::process::Command::new("powershell.exe")
            .args(["-Command", "echo $env:USERPROFILE"])
            .output()
            .ok()?;
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Some(PathBuf::from(path))
        } else {
            None
        }
    }

    fn read_file(&self, path: &str) -> Result<String, String> {
        // Convert Windows path to /mnt/c/ path for direct access from WSL
        let wsl_path = Self::win_to_wsl_path(path);
        std::fs::read_to_string(&wsl_path)
            .map_err(|e| format!("Failed to read {wsl_path}: {e}"))
    }

    fn write_file(&self, path: &str, content: &str) -> Result<(), String> {
        // Try /mnt/c/ path first for direct access
        let wsl_path = Self::win_to_wsl_path(path);
        if std::path::Path::new(&wsl_path).exists() || wsl_path.starts_with("/mnt/") {
            std::fs::write(&wsl_path, content)
                .map_err(|e| format!("Failed to write {wsl_path}: {e}"))
        } else {
            // Fallback to powershell Set-Content
            let escaped = content.replace('\'', "''");
            self.execute(
                "Set-Content",
                &["-Path", path, "-Value", &format!("'{escaped}'")],
            )?;
            Ok(())
        }
    }
}

impl PowershellBridgeExecutor {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }

    /// Convert a Windows path (C:\...) to a WSL /mnt/c/... path.
    fn win_to_wsl_path(path: &str) -> String {
        let path = path.replace('\\', "/");
        path.strip_prefix("C:/")
            .map(|s| format!("/mnt/c/{s}"))
            .or_else(|| path.strip_prefix("D:/").map(|s| format!("/mnt/d/{s}")))
            .unwrap_or(path)
    }
}

impl Default for PowershellBridgeExecutor {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// MockCommandExecutor (shared test double)
// ---------------------------------------------------------------------------

/// Mock executor for testing remote adapters.
///
/// Placed at module level (not inside `tests`) so other adapter test modules
/// can import it: `use crate::adapters::command_executor::MockCommandExecutor;`
#[cfg(test)]
use std::collections::HashMap;

#[cfg(test)]
pub struct MockCommandExecutor {
    outputs: HashMap<String, Result<String, String>>,
    env_vars: HashMap<String, String>,
    home: Option<PathBuf>,
    file_contents: HashMap<String, String>,
}

#[cfg(test)]
impl MockCommandExecutor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            outputs: HashMap::new(),
            env_vars: HashMap::new(),
            home: None,
            file_contents: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_output(mut self, program: &str, args: &[&str], result: Result<String, String>) -> Self {
        let key = Self::command_key(program, args);
        self.outputs.insert(key, result);
        self
    }

    #[must_use]
    pub fn with_env_var(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    #[must_use]
    pub fn with_home(mut self, home: PathBuf) -> Self {
        self.home = Some(home);
        self
    }

    #[must_use]
    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        self.file_contents.insert(path.to_string(), content.to_string());
        self
    }

    fn command_key(program: &str, args: &[&str]) -> String {
        let mut key = program.to_string();
        for arg in args {
            key.push(' ');
            key.push_str(arg);
        }
        key
    }
}

#[cfg(test)]
impl Default for MockCommandExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl CommandExecutor for MockCommandExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let key = Self::command_key(program, args);
        self.outputs
            .get(&key)
            .cloned()
            .unwrap_or_else(|| Err(format!("No mock output for: {key}")))
    }

    fn env_var(&self, key: &str) -> Option<String> {
        self.env_vars.get(key).cloned()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        self.home.clone()
    }

    fn read_file(&self, path: &str) -> Result<String, String> {
        self.file_contents
            .get(path)
            .cloned()
            .ok_or_else(|| format!("No mock file: {path}"))
    }

    fn write_file(&self, _path: &str, _content: &str) -> Result<(), String> {
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mock_executor_returns_configured_output() {
        let mock = MockCommandExecutor::new()
            .with_output("cat", &["/etc/resolv.conf"], Ok("nameserver 8.8.8.8".to_string()));
        let result = mock.execute("cat", &["/etc/resolv.conf"]);
        assert_eq!(result.unwrap(), "nameserver 8.8.8.8");
    }

    #[test]
    fn mock_executor_returns_error_for_unknown_command() {
        let mock = MockCommandExecutor::new();
        let result = mock.execute("unknown", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn mock_executor_returns_env_var() {
        let mock = MockCommandExecutor::new().with_env_var("HOME", "/home/user");
        assert_eq!(mock.env_var("HOME"), Some("/home/user".to_string()));
        assert_eq!(mock.env_var("MISSING"), None);
    }

    #[test]
    fn mock_executor_returns_home_dir() {
        let mock = MockCommandExecutor::new().with_home(PathBuf::from("/home/user"));
        assert_eq!(mock.home_dir(), Some(PathBuf::from("/home/user")));
    }

    #[test]
    fn mock_executor_reads_file() {
        let mock = MockCommandExecutor::new().with_file("/etc/hosts", "127.0.0.1 localhost");
        assert_eq!(mock.read_file("/etc/hosts").unwrap(), "127.0.0.1 localhost");
    }

    #[test]
    fn mock_executor_write_file_succeeds() {
        let mock = MockCommandExecutor::new();
        assert!(mock.write_file("/tmp/test", "content").is_ok());
    }

    #[test]
    fn trait_object_dispatch_works() {
        let executor: Box<dyn CommandExecutor> = Box::new(
            MockCommandExecutor::new().with_output("echo", &["hello"], Ok("hello".to_string())),
        );
        assert_eq!(executor.execute("echo", &["hello"]).unwrap(), "hello");
    }

    #[test]
    fn wsl_bridge_construction_without_distro() {
        let bridge = WslBridgeExecutor::new();
        assert!(bridge.distro.is_none());
    }

    #[test]
    fn wsl_bridge_construction_with_distro() {
        let bridge = WslBridgeExecutor::new().with_distro("Ubuntu");
        assert_eq!(bridge.distro.as_deref(), Some("Ubuntu"));
    }

    #[test]
    fn wsl_bridge_args_without_distro() {
        let bridge = WslBridgeExecutor::new();
        let args = bridge.wsl_args(&["bash", "-c", "echo hello"]);
        assert_eq!(args, vec!["-e", "bash", "-c", "echo hello"]);
    }

    #[test]
    fn wsl_bridge_args_with_distro() {
        let bridge = WslBridgeExecutor::new().with_distro("Ubuntu");
        let args = bridge.wsl_args(&["bash"]);
        assert_eq!(args, vec!["-d", "Ubuntu", "-e", "bash"]);
    }

    #[test]
    fn powershell_bridge_win_to_wsl_path_c_drive() {
        assert_eq!(
            PowershellBridgeExecutor::win_to_wsl_path(r"C:\Windows\System32\drivers\etc\hosts"),
            "/mnt/c/Windows/System32/drivers/etc/hosts"
        );
    }

    #[test]
    fn powershell_bridge_win_to_wsl_path_d_drive() {
        assert_eq!(
            PowershellBridgeExecutor::win_to_wsl_path(r"D:\data\config.yaml"),
            "/mnt/d/data/config.yaml"
        );
    }

    #[test]
    fn powershell_bridge_win_to_wsl_path_already_unix() {
        assert_eq!(
            PowershellBridgeExecutor::win_to_wsl_path("/mnt/c/test"),
            "/mnt/c/test"
        );
    }
}
