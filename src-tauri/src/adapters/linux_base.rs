//! Linux base adapter providing shared read/write primitives for `WslAdapter` and `LinuxAdapter`.
//!
//! This module does NOT implement the `PlatformAdapter` trait. It is a composable
//! base layer that provides shell execution abstraction, pure parsing functions,
//! and file I/O operations common to both WSL and native Linux environments.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use serde_json::{json, Value};

// ---------------------------------------------------------------------------
// ShellExecutor trait
// ---------------------------------------------------------------------------

/// Abstraction over shell command execution and environment queries.
///
/// Enables dependency injection for testing: production code uses
/// `SystemShellExecutor`, tests use `MockShellExecutor`.
pub(crate) trait ShellExecutor {
    /// Execute a program with arguments and return its stdout.
    fn execute(&self, program: &str, args: &[&str]) -> Result<String, String>;
    /// Query an environment variable.
    fn env_var(&self, key: &str) -> Option<String>;
    /// Return the current user's home directory.
    fn home_dir(&self) -> Option<PathBuf>;
}

// ---------------------------------------------------------------------------
// SystemShellExecutor (production)
// ---------------------------------------------------------------------------

/// Production shell executor using `std::process::Command` and `std::env`.
pub struct SystemShellExecutor;

impl ShellExecutor for SystemShellExecutor {
    fn execute(&self, program: &str, args: &[&str]) -> Result<String, String> {
        let output = Command::new(program)
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute {program}: {e}"))?;
        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("{program} failed: {stderr}"))
        }
    }

    fn env_var(&self, key: &str) -> Option<String> {
        std::env::var(key).ok()
    }

    fn home_dir(&self) -> Option<PathBuf> {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

// ---------------------------------------------------------------------------
// MockShellExecutor (test helper)
// ---------------------------------------------------------------------------

/// Test double for `ShellExecutor` with configurable command outputs,
/// environment variables, and home directory.
#[cfg(test)]
mod mock_executor {
    use std::collections::HashMap;
    use std::path::PathBuf;

    use super::ShellExecutor;

    #[derive(Default)]
    pub struct MockShellExecutor {
        pub(super) outputs: HashMap<String, Result<String, String>>,
        pub(super) env_vars: HashMap<String, String>,
        pub(super) home: Option<PathBuf>,
    }

    impl MockShellExecutor {
        #[must_use]
        pub fn new() -> Self {
            Self::default()
        }

        #[must_use]
        pub fn with_output(
            mut self,
            program: &str,
            args: &[&str],
            result: Result<String, String>,
        ) -> Self {
            let key = format!("{} {}", program, args.join(" "));
            self.outputs.insert(key, result);
            self
        }

        #[must_use]
        pub fn with_env_var(mut self, key: &str, value: &str) -> Self {
            self.env_vars.insert(key.to_string(), value.to_string());
            self
        }

        #[must_use]
        pub fn with_home(mut self, path: PathBuf) -> Self {
            self.home = Some(path);
            self
        }
    }

    impl ShellExecutor for MockShellExecutor {
        fn execute(&self, program: &str, args: &[&str]) -> Result<String, String> {
            let key = format!("{} {}", program, args.join(" "));
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
    }
}

#[cfg(test)]
pub use mock_executor::MockShellExecutor;

// ---------------------------------------------------------------------------
// WritePermission
// ---------------------------------------------------------------------------

/// Result of checking write permission for a system file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WritePermission {
    /// The current user can write to the file.
    Granted,
    /// Root privileges are required to write to the file.
    NeedRoot { suggested_command: String },
}

// ---------------------------------------------------------------------------
// Pure parsing functions
// ---------------------------------------------------------------------------

/// Parse `/etc/resolv.conf` content and extract nameserver entries.
#[must_use]
pub fn parse_resolv_conf(content: &str) -> Value {
    let mut nameservers: Vec<String> = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("nameserver") {
            let addr = rest.trim();
            if !addr.is_empty() {
                nameservers.push(addr.to_string());
            }
        }
    }

    json!({
        "nameservers": nameservers,
    })
}

/// Parse `/etc/environment` content and extract KEY=VALUE pairs.
#[must_use]
pub fn parse_etc_environment(content: &str) -> Value {
    let mut pairs = serde_json::Map::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, val)) = trimmed.split_once('=') {
            let key = key.trim().to_string();
            let val = val.trim().trim_matches('"').to_string();
            pairs.insert(key, Value::String(val));
        }
    }

    Value::Object(pairs)
}

/// Parse `git config --global` output for proxy settings.
#[must_use]
#[allow(clippy::similar_names)] // http_proxy/https_proxy are domain-standard names
pub fn parse_git_proxy_output(output: &str) -> Value {
    let mut http_proxy = String::new();
    let mut https_proxy = String::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("http.proxy") {
            http_proxy = rest.trim_start_matches(['=', ' ']).trim().to_string();
        } else if let Some(rest) = trimmed.strip_prefix("https.proxy") {
            https_proxy = rest.trim_start_matches(['=', ' ']).trim().to_string();
        }
    }

    json!({
        "http_proxy": http_proxy,
        "https_proxy": https_proxy,
    })
}

// ---------------------------------------------------------------------------
// LinuxBaseAdapter
// ---------------------------------------------------------------------------

/// Shared Linux read/write primitives for `WslAdapter` and `LinuxAdapter`.
///
/// Generic over `ShellExecutor` to enable dependency injection in tests.
pub(crate) struct LinuxBaseAdapter<E: ShellExecutor> {
    executor: E,
}

#[allow(dead_code)] // Methods used by future WslAdapter and LinuxAdapter consumers
#[allow(clippy::similar_names)] // http_proxy/https_proxy are domain-standard names
impl<E: ShellExecutor> LinuxBaseAdapter<E> {
    /// Create a new adapter with the given shell executor.
    #[must_use]
    pub const fn new(executor: E) -> Self {
        Self { executor }
    }

    /// Get a reference to the underlying executor.
    #[must_use]
    pub const fn executor(&self) -> &E {
        &self.executor
    }

    // -----------------------------------------------------------------------
    // Read operations
    // -----------------------------------------------------------------------

    /// Read proxy-related environment variables (`http_proxy`, `https_proxy`, `no_proxy`).
    #[must_use]
    pub fn read_proxy_env_vars(&self) -> Value {
        let http = self
            .executor
            .env_var("http_proxy")
            .or_else(|| self.executor.env_var("HTTP_PROXY"))
            .unwrap_or_default();
        let https = self
            .executor
            .env_var("https_proxy")
            .or_else(|| self.executor.env_var("HTTPS_PROXY"))
            .unwrap_or_default();
        let no_proxy = self
            .executor
            .env_var("no_proxy")
            .or_else(|| self.executor.env_var("NO_PROXY"))
            .unwrap_or_default();

        json!({
            "http_proxy": http,
            "https_proxy": https,
            "no_proxy": no_proxy,
        })
    }

    /// Read git global proxy settings via `git config --global`.
    #[must_use]
    pub fn read_git_proxy(&self) -> Value {
        let http_result = self
            .executor
            .execute("git", &["config", "--global", "http.proxy"]);
        let https_result = self
            .executor
            .execute("git", &["config", "--global", "https.proxy"]);

        let http_proxy = http_result
            .map(|s| s.trim().to_string())
            .unwrap_or_default();
        let https_proxy = https_result
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        json!({
            "http_proxy": http_proxy,
            "https_proxy": https_proxy,
        })
    }

    /// Read and parse `/etc/resolv.conf` from the given path.
    #[must_use]
    #[allow(clippy::unused_self)] // Kept for API consistency with other read methods
    pub fn read_resolv_conf(&self, path: &Path) -> Value {
        match fs::read_to_string(path) {
            Ok(content) => {
                let mut result = parse_resolv_conf(&content);
                if let Some(obj) = result.as_object_mut() {
                    obj.insert(
                        "path".to_string(),
                        json!(path.to_string_lossy().to_string()),
                    );
                }
                result
            }
            Err(e) => json!({
                "error": format!("Failed to read {}: {e}", path.display()),
                "nameservers": [],
            }),
        }
    }

    /// Read and parse `/etc/environment` from the given path.
    #[must_use]
    #[allow(clippy::unused_self)] // Kept for API consistency with other read methods
    pub fn read_etc_environment(&self, path: &Path) -> Value {
        match fs::read_to_string(path) {
            Ok(content) => parse_etc_environment(&content),
            Err(e) => json!({
                "error": format!("Failed to read {}: {e}", path.display()),
            }),
        }
    }

    /// Read proxy lines from shell RC files (.bashrc, .zshrc).
    #[must_use]
    pub fn read_shell_rc_proxy(&self) -> Value {
        let Some(home) = self.executor.home_dir() else {
            return json!({ "error": "Cannot determine home directory" });
        };

        let mut proxy_lines: Vec<Value> = Vec::new();
        let rc_files = [".bashrc", ".zshrc"];

        for rc_name in &rc_files {
            let rc_path = home.join(rc_name);
            if let Ok(content) = fs::read_to_string(&rc_path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    let lower = trimmed.to_lowercase();
                    if (lower.contains("http_proxy")
                        || lower.contains("https_proxy")
                        || lower.contains("no_proxy"))
                        && (lower.starts_with("export") || lower.contains('='))
                    {
                        proxy_lines.push(json!({
                            "file": rc_name,
                            "line": trimmed,
                        }));
                    }
                }
            }
        }

        json!({
            "proxy_lines": proxy_lines,
            "count": proxy_lines.len(),
        })
    }

    /// Check network reachability to a URL, returning reachable status and latency.
    #[must_use]
    pub fn check_reachability(&self, url: &str) -> Value {
        let start = Instant::now();
        let result = self.executor.execute(
            "curl",
            &[
                "-s",
                "-o",
                "/dev/null",
                "-w",
                "%{http_code}",
                "--connect-timeout",
                "5",
                "--max-time",
                "10",
                url,
            ],
        );
        let elapsed = start.elapsed();
        #[allow(clippy::cast_possible_truncation)] // ms from Instant will never exceed u64
        let latency_ms = elapsed.as_millis() as u64;

        match result {
            Ok(output) => {
                let status_code: u32 = output.trim().parse().unwrap_or(0);
                let reachable = (200..400).contains(&status_code);
                json!({
                    "reachable": reachable,
                    "latency_ms": latency_ms,
                    "status_code": status_code,
                    "url": url,
                })
            }
            Err(e) => json!({
                "reachable": false,
                "latency_ms": latency_ms,
                "error": e,
                "url": url,
            }),
        }
    }

    // -----------------------------------------------------------------------
    // Write operations
    // -----------------------------------------------------------------------

    /// Write git global proxy settings.
    #[allow(clippy::similar_names)] // http_proxy/https_proxy are domain-standard names
    pub fn write_git_proxy(
        &self,
        http_proxy: &str,
        https_proxy: &str,
    ) -> Result<(), String> {
        if !http_proxy.is_empty() {
            self.executor
                .execute("git", &["config", "--global", "http.proxy", http_proxy])?;
        }
        if !https_proxy.is_empty() {
            self.executor
                .execute("git", &["config", "--global", "https.proxy", https_proxy])?;
        }
        Ok(())
    }

    /// Check write permission for a file path.
    #[must_use]
    pub fn check_write_permission(path: &Path) -> WritePermission {
        // Try opening for append to test writability
        match fs::OpenOptions::new().append(true).open(path) {
            Ok(_) => WritePermission::Granted,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                let cmd = format!("sudo tee {}", path.display());
                WritePermission::NeedRoot { suggested_command: cmd }
            }
            Err(_) => {
                // File may not exist yet; check parent directory
                path.parent().map_or(WritePermission::Granted, |parent| {
                    let test_file = parent.join(".goguo_write_test_tmp");
                    match fs::OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&test_file)
                    {
                        Ok(_) => {
                            let _ = fs::remove_file(&test_file);
                            WritePermission::Granted
                        }
                        Err(e2) if e2.kind() == std::io::ErrorKind::PermissionDenied => {
                            let cmd = format!("sudo tee {}", path.display());
                            WritePermission::NeedRoot { suggested_command: cmd }
                        }
                        Err(_) => {
                            let cmd = format!("sudo tee {}", path.display());
                            WritePermission::NeedRoot { suggested_command: cmd }
                        }
                    }
                })
            }
        }
    }

    /// Write content to a resolv.conf path, checking permissions first.
    #[allow(clippy::unused_self)] // Kept for API consistency with write methods
    pub fn write_resolv_conf(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<WritePermission, String> {
        let perm = Self::check_write_permission(path);
        if matches!(perm, WritePermission::NeedRoot { .. }) {
            return Ok(perm);
        }
        fs::write(path, content)
            .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
        Ok(perm)
    }

    /// Write content to an /etc/environment path, checking permissions first.
    #[allow(clippy::unused_self)] // Kept for API consistency with write methods
    pub fn write_etc_environment(
        &self,
        path: &Path,
        content: &str,
    ) -> Result<WritePermission, String> {
        let perm = Self::check_write_permission(path);
        if matches!(perm, WritePermission::NeedRoot { .. }) {
            return Ok(perm);
        }
        fs::write(path, content)
            .map_err(|e| format!("Failed to write {}: {e}", path.display()))?;
        Ok(perm)
    }

    /// Write proxy environment variables to an environment file (e.g., `/etc/environment`).
    ///
    /// Reads existing content, removes old proxy-related lines, appends new ones,
    /// and writes back. Non-proxy lines are preserved.
    #[allow(clippy::unused_self)]
    pub fn write_proxy_env(
        &self,
        path: &Path,
        http_proxy: &str,
        https_proxy: &str,
        no_proxy: &str,
    ) -> Result<WritePermission, String> {
        let existing = fs::read_to_string(path).unwrap_or_default();
        let mut lines: Vec<String> = existing
            .lines()
            .filter(|line| !is_proxy_env_line(line))
            .map(String::from)
            .collect();

        if !http_proxy.is_empty() {
            lines.push(format!("http_proxy=\"{http_proxy}\""));
            lines.push(format!("HTTP_PROXY=\"{http_proxy}\""));
        }
        if !https_proxy.is_empty() {
            lines.push(format!("https_proxy=\"{https_proxy}\""));
            lines.push(format!("HTTPS_PROXY=\"{https_proxy}\""));
        }
        if !no_proxy.is_empty() {
            lines.push(format!("no_proxy=\"{no_proxy}\""));
            lines.push(format!("NO_PROXY=\"{no_proxy}\""));
        }

        let content = lines.join("\n");
        self.write_etc_environment(path, &content)
    }
}

/// Check if a line is a proxy-related environment variable assignment.
fn is_proxy_env_line(line: &str) -> bool {
    let lower = line.to_lowercase();
    lower.starts_with("http_proxy=")
        || lower.starts_with("https_proxy=")
        || lower.starts_with("no_proxy=")
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // MockShellExecutor injection tests
    // -----------------------------------------------------------------------

    #[test]
    fn mock_executor_returns_configured_output() {
        let mock = MockShellExecutor::new().with_output(
            "git",
            &["config", "--global", "http.proxy"],
            Ok("http://proxy:8080".to_string()),
        );
        let result = mock
            .execute("git", &["config", "--global", "http.proxy"])
            .unwrap();
        assert_eq!(result, "http://proxy:8080");
    }

    #[test]
    fn mock_executor_returns_error_for_unknown_command() {
        let mock = MockShellExecutor::new();
        let result = mock.execute("nonexistent", &[]);
        assert!(result.is_err());
    }

    #[test]
    fn mock_executor_returns_env_vars() {
        let mock = MockShellExecutor::new()
            .with_env_var("http_proxy", "http://proxy:8080")
            .with_home(PathBuf::from("/home/testuser"));
        assert_eq!(
            mock.env_var("http_proxy"),
            Some("http://proxy:8080".to_string())
        );
        assert_eq!(mock.home_dir(), Some(PathBuf::from("/home/testuser")));
        assert_eq!(mock.env_var("nonexistent"), None);
    }

    // -----------------------------------------------------------------------
    // parse_resolv_conf tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_resolv_conf_normal() {
        let content = "# Generated by NetworkManager\nnameserver 8.8.8.8\nnameserver 8.8.4.4\n";
        let result = parse_resolv_conf(content);
        let ns = result["nameservers"].as_array().unwrap();
        assert_eq!(ns.len(), 2);
        assert_eq!(ns[0], "8.8.8.8");
        assert_eq!(ns[1], "8.8.4.4");
    }

    #[test]
    fn parse_resolv_conf_empty() {
        let result = parse_resolv_conf("");
        let ns = result["nameservers"].as_array().unwrap();
        assert!(ns.is_empty());
    }

    #[test]
    fn parse_resolv_conf_multiple_nameservers() {
        let content = "nameserver 1.1.1.1\nnameserver 1.0.0.1\nnameserver 8.8.8.8\n";
        let result = parse_resolv_conf(content);
        let ns = result["nameservers"].as_array().unwrap();
        assert_eq!(ns.len(), 3);
    }

    #[test]
    fn parse_resolv_conf_skips_comments() {
        let content =
            "# This is a comment\n; semicolon line\nnameserver 192.168.1.1\n# another comment\n";
        let result = parse_resolv_conf(content);
        let ns = result["nameservers"].as_array().unwrap();
        assert_eq!(ns.len(), 1);
        assert_eq!(ns[0], "192.168.1.1");
    }

    // -----------------------------------------------------------------------
    // parse_etc_environment tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_etc_environment_normal() {
        let content = "http_proxy=http://proxy:8080\nhttps_proxy=http://proxy:8080\n";
        let result = parse_etc_environment(content);
        assert_eq!(result["http_proxy"], "http://proxy:8080");
        assert_eq!(result["https_proxy"], "http://proxy:8080");
    }

    #[test]
    fn parse_etc_environment_empty() {
        let result = parse_etc_environment("");
        assert!(result.as_object().unwrap().is_empty());
    }

    #[test]
    fn parse_etc_environment_quoted_values() {
        let content = "http_proxy=\"http://proxy:8080\"\nPATH=\"/usr/local/bin:/usr/bin\"\n";
        let result = parse_etc_environment(content);
        assert_eq!(result["http_proxy"], "http://proxy:8080");
        assert_eq!(result["PATH"], "/usr/local/bin:/usr/bin");
    }

    // -----------------------------------------------------------------------
    // parse_git_proxy_output tests
    // -----------------------------------------------------------------------

    #[test]
    fn parse_git_proxy_output_normal() {
        let output = "http.proxy=http://proxy:8080\nhttps.proxy=http://proxy:8443\n";
        let result = parse_git_proxy_output(output);
        assert_eq!(result["http_proxy"], "http://proxy:8080");
        assert_eq!(result["https_proxy"], "http://proxy:8443");
    }

    #[test]
    fn parse_git_proxy_output_empty() {
        let result = parse_git_proxy_output("");
        assert_eq!(result["http_proxy"], "");
        assert_eq!(result["https_proxy"], "");
    }

    #[test]
    fn parse_git_proxy_output_both_proxies() {
        let output = "http.proxy=socks5://127.0.0.1:1080\nhttps.proxy=socks5://127.0.0.1:1080\n";
        let result = parse_git_proxy_output(output);
        assert_eq!(result["http_proxy"], "socks5://127.0.0.1:1080");
        assert_eq!(result["https_proxy"], "socks5://127.0.0.1:1080");
    }

    // -----------------------------------------------------------------------
    // read_proxy_env_vars via mock tests
    // -----------------------------------------------------------------------

    #[test]
    fn read_proxy_env_vars_with_values() {
        let mock = MockShellExecutor::new()
            .with_env_var("http_proxy", "http://proxy:8080")
            .with_env_var("https_proxy", "http://proxy:8443")
            .with_env_var("no_proxy", "localhost,127.0.0.1");
        let adapter = LinuxBaseAdapter::new(mock);
        let result = adapter.read_proxy_env_vars();
        assert_eq!(result["http_proxy"], "http://proxy:8080");
        assert_eq!(result["https_proxy"], "http://proxy:8443");
        assert_eq!(result["no_proxy"], "localhost,127.0.0.1");
    }

    #[test]
    fn read_proxy_env_vars_empty_when_not_set() {
        let mock = MockShellExecutor::new();
        let adapter = LinuxBaseAdapter::new(mock);
        let result = adapter.read_proxy_env_vars();
        assert_eq!(result["http_proxy"], "");
        assert_eq!(result["https_proxy"], "");
        assert_eq!(result["no_proxy"], "");
    }

    // -----------------------------------------------------------------------
    // read_git_proxy via mock tests
    // -----------------------------------------------------------------------

    #[test]
    fn read_git_proxy_with_values() {
        let mock = MockShellExecutor::new()
            .with_output(
                "git",
                &["config", "--global", "http.proxy"],
                Ok("http://proxy:8080\n".to_string()),
            )
            .with_output(
                "git",
                &["config", "--global", "https.proxy"],
                Ok("http://proxy:8443\n".to_string()),
            );
        let adapter = LinuxBaseAdapter::new(mock);
        let result = adapter.read_git_proxy();
        assert_eq!(result["http_proxy"], "http://proxy:8080");
        assert_eq!(result["https_proxy"], "http://proxy:8443");
    }

    #[test]
    fn read_git_proxy_empty_when_not_configured() {
        let mock = MockShellExecutor::new()
            .with_output(
                "git",
                &["config", "--global", "http.proxy"],
                Err("not set".to_string()),
            )
            .with_output(
                "git",
                &["config", "--global", "https.proxy"],
                Err("not set".to_string()),
            );
        let adapter = LinuxBaseAdapter::new(mock);
        let result = adapter.read_git_proxy();
        assert_eq!(result["http_proxy"], "");
        assert_eq!(result["https_proxy"], "");
    }

    // -----------------------------------------------------------------------
    // read_resolv_conf via temp file tests
    // -----------------------------------------------------------------------

    #[test]
    fn read_resolv_conf_from_temp_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("resolv.conf");
        fs::write(&path, "nameserver 8.8.8.8\nnameserver 8.8.4.4\n").expect("write");

        let mock = MockShellExecutor::new();
        let adapter = LinuxBaseAdapter::new(mock);
        let result = adapter.read_resolv_conf(&path);
        let ns = result["nameservers"].as_array().unwrap();
        assert_eq!(ns.len(), 2);
        assert_eq!(ns[0], "8.8.8.8");
    }

    #[test]
    fn read_resolv_conf_missing_file_returns_error() {
        let mock = MockShellExecutor::new();
        let adapter = LinuxBaseAdapter::new(mock);
        let result = adapter.read_resolv_conf(Path::new("/nonexistent/resolv.conf"));
        assert!(result["error"].is_string());
        let ns = result["nameservers"].as_array().unwrap();
        assert!(ns.is_empty());
    }

    // -----------------------------------------------------------------------
    // read_etc_environment via temp file tests
    // -----------------------------------------------------------------------

    #[test]
    fn read_etc_environment_from_temp_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("environment");
        fs::write(&path, "http_proxy=http://proxy:8080\nPATH=/usr/bin\n").expect("write");

        let mock = MockShellExecutor::new();
        let adapter = LinuxBaseAdapter::new(mock);
        let result = adapter.read_etc_environment(&path);
        assert_eq!(result["http_proxy"], "http://proxy:8080");
        assert_eq!(result["PATH"], "/usr/bin");
    }

    #[test]
    fn read_etc_environment_missing_file_returns_error() {
        let mock = MockShellExecutor::new();
        let adapter = LinuxBaseAdapter::new(mock);
        let result = adapter.read_etc_environment(Path::new("/nonexistent/environment"));
        assert!(result["error"].is_string());
    }

    // -----------------------------------------------------------------------
    // Root permission check tests
    // -----------------------------------------------------------------------

    #[test]
    fn check_write_permission_granted_for_writable_file() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let path = dir.path().join("writable.conf");
        fs::write(&path, "test").expect("write");

        let perm = LinuxBaseAdapter::<MockShellExecutor>::check_write_permission(&path);
        assert_eq!(perm, WritePermission::Granted);
    }

    #[test]
    fn check_write_permission_need_root_for_system_path() {
        let perm = LinuxBaseAdapter::<MockShellExecutor>::check_write_permission(
            Path::new("/etc/resolv.conf"),
        );
        // On most systems /etc/resolv.conf is not user-writable,
        // but some test environments may allow it. Verify the function runs
        // without panic and returns a valid variant.
        match perm {
            WritePermission::Granted => { /* acceptable in some environments */ }
            WritePermission::NeedRoot { suggested_command } => {
                assert!(suggested_command.contains("sudo"));
            }
        }
    }

    // ── write_proxy_env tests (F102) ───────────────────────────────────────

    #[test]
    fn is_proxy_env_line_recognizes_all_variants() {
        assert!(is_proxy_env_line("http_proxy=\"http://proxy:8080\""));
        assert!(is_proxy_env_line("HTTP_PROXY=\"http://proxy:8080\""));
        assert!(is_proxy_env_line("https_proxy=\"http://proxy:8443\""));
        assert!(is_proxy_env_line("HTTPS_PROXY=\"http://proxy:8443\""));
        assert!(is_proxy_env_line("no_proxy=\"localhost,127.0.0.1\""));
        assert!(is_proxy_env_line("NO_PROXY=\"localhost,127.0.0.1\""));
    }

    #[test]
    fn is_proxy_env_line_rejects_non_proxy_lines() {
        assert!(!is_proxy_env_line("PATH=/usr/bin"));
        assert!(!is_proxy_env_line(""));
        assert!(!is_proxy_env_line("# http_proxy=ignored"));
        assert!(!is_proxy_env_line("SOME_OTHER_VAR=\"value\""));
    }

    #[test]
    fn write_proxy_env_writes_to_empty_file() {
        let dir = tempfile::TempDir::new().expect("dir");
        let path = dir.path().join("environment");
        fs::write(&path, "").expect("create empty file");

        let adapter = LinuxBaseAdapter::new(MockShellExecutor::default());
        let perm = adapter.write_proxy_env(&path, "http://proxy:8080", "http://proxy:8443", "localhost").expect("write");
        assert_eq!(perm, WritePermission::Granted);

        let content = fs::read_to_string(&path).expect("read");
        assert!(content.contains("http_proxy=\"http://proxy:8080\""));
        assert!(content.contains("HTTP_PROXY=\"http://proxy:8080\""));
        assert!(content.contains("https_proxy=\"http://proxy:8443\""));
        assert!(content.contains("HTTPS_PROXY=\"http://proxy:8443\""));
        assert!(content.contains("no_proxy=\"localhost\""));
        assert!(content.contains("NO_PROXY=\"localhost\""));
    }

    #[test]
    fn write_proxy_env_preserves_existing_non_proxy_lines() {
        let dir = tempfile::TempDir::new().expect("dir");
        let path = dir.path().join("environment");
        fs::write(&path, "PATH=/usr/local/bin:/usr/bin\nLANG=en_US.UTF-8\n").expect("write existing");

        let adapter = LinuxBaseAdapter::new(MockShellExecutor::default());
        let perm = adapter.write_proxy_env(&path, "http://proxy:8080", "", "").expect("write");
        assert_eq!(perm, WritePermission::Granted);

        let content = fs::read_to_string(&path).expect("read");
        assert!(content.contains("PATH=/usr/local/bin:/usr/bin"));
        assert!(content.contains("LANG=en_US.UTF-8"));
        assert!(content.contains("http_proxy=\"http://proxy:8080\""));
        assert!(!content.contains("https_proxy="));
    }

    #[test]
    fn write_proxy_env_replaces_existing_proxy_lines() {
        let dir = tempfile::TempDir::new().expect("dir");
        let path = dir.path().join("environment");
        fs::write(&path, "PATH=/usr/bin\nhttp_proxy=\"http://old:8080\"\nHTTP_PROXY=\"http://old:8080\"\n").expect("write existing");

        let adapter = LinuxBaseAdapter::new(MockShellExecutor::default());
        adapter.write_proxy_env(&path, "http://new:9090", "", "").expect("write");

        let content = fs::read_to_string(&path).expect("read");
        assert!(content.contains("PATH=/usr/bin"));
        assert!(content.contains("http_proxy=\"http://new:9090\""));
        assert!(!content.contains("http://old:8080"));
        // Should not have duplicate old lines
        assert_eq!(content.matches("http_proxy=").count(), 1);
        assert_eq!(content.matches("HTTP_PROXY=").count(), 1);
    }

    #[test]
    fn write_proxy_env_empty_values_add_no_lines() {
        let dir = tempfile::TempDir::new().expect("dir");
        let path = dir.path().join("environment");
        fs::write(&path, "PATH=/usr/bin\n").expect("write existing");

        let adapter = LinuxBaseAdapter::new(MockShellExecutor::default());
        let perm = adapter.write_proxy_env(&path, "", "", "").expect("write");
        assert_eq!(perm, WritePermission::Granted);

        let content = fs::read_to_string(&path).expect("read");
        assert_eq!(content, "PATH=/usr/bin");
    }

    #[test]
    fn write_proxy_env_handles_nonexistent_file() {
        let dir = tempfile::TempDir::new().expect("dir");
        let path = dir.path().join("environment");
        // Don't create the file

        let adapter = LinuxBaseAdapter::new(MockShellExecutor::default());
        let perm = adapter.write_proxy_env(&path, "http://proxy:8080", "", "").expect("write");
        assert_eq!(perm, WritePermission::Granted);

        let content = fs::read_to_string(&path).expect("read");
        assert!(content.contains("http_proxy=\"http://proxy:8080\""));
    }
}
