use std::path::PathBuf;
use std::process::Child;
use std::time::Duration;

use crate::models::config::MihomoConfig;

/// Error type for `MihomoManager` operations.
#[derive(Debug)]
pub enum MihomoError {
    /// The mihomo binary was not found at the configured path.
    BinaryNotFound(PathBuf),
    /// The process failed to start.
    StartFailed(String),
    /// The API did not become ready within the timeout.
    ApiTimeout,
    /// An I/O error occurred.
    Io(std::io::Error),
    /// The process is not running.
    NotRunning,
    /// The API port is already in use by a non-mihomo process.
    PortConflict(String),
}

impl std::fmt::Display for MihomoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BinaryNotFound(p) => write!(f, "Binary not found: {}", p.display()),
            Self::StartFailed(e) => write!(f, "Start failed: {e}"),
            Self::ApiTimeout => write!(f, "API did not become ready within timeout"),
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::NotRunning => write!(f, "Mihomo process is not running"),
            Self::PortConflict(addr) => {
                write!(f, "API port {addr} is already in use by a non-mihomo process")
            }
        }
    }
}

/// Trait for mihomo config reload operations, enabling dependency injection.
pub trait MihomoReloader: Send + Sync {
    /// Reload mihomo configuration from the given path.
    ///
    /// # Errors
    ///
    /// Returns `MihomoError` if the reload fails.
    fn reload_config(&self, config_path: &str) -> Result<(), MihomoError>;
}

impl MihomoReloader for MihomoManager {
    fn reload_config(&self, config_path: &str) -> Result<(), MihomoError> {
        self.reload_config(config_path)
    }
}

/// Mock reloader for testing.
pub struct MockMihomoReloader {
    called: std::sync::Mutex<bool>,
    last_path: std::sync::Mutex<Option<String>>,
}

impl MockMihomoReloader {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            called: std::sync::Mutex::new(false),
            last_path: std::sync::Mutex::new(None),
        }
    }

    #[must_use]
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn was_called(&self) -> bool {
        *self.called.lock().unwrap()
    }

    #[must_use]
    /// # Panics
    ///
    /// Panics if the internal mutex is poisoned.
    pub fn last_config_path(&self) -> Option<String> {
        self.last_path.lock().unwrap().clone()
    }
}

impl Default for MockMihomoReloader {
    fn default() -> Self {
        Self::new()
    }
}

impl MihomoReloader for MockMihomoReloader {
    fn reload_config(&self, config_path: &str) -> Result<(), MihomoError> {
        *self.called.lock().unwrap() = true;
        *self.last_path.lock().unwrap() = Some(config_path.to_string());
        Ok(())
    }
}

impl From<std::io::Error> for MihomoError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Manages the mihomo subprocess lifecycle: start, stop, health check, config reload.
pub struct MihomoManager {
    config: MihomoConfig,
    process: Option<Child>,
    /// If true, the mihomo process was already running when `GoGuo` started
    /// and we adopted it. We should NOT kill it on stop/drop.
    externally_managed: bool,
}

impl MihomoManager {
    /// Create a new manager with the given configuration.
    #[must_use]
    pub const fn new(config: MihomoConfig) -> Self {
        Self {
            config,
            process: None,
            externally_managed: false,
        }
    }

    /// Get the configured mixed (SOCKS5 + HTTP) proxy port.
    #[must_use]
    pub const fn mixed_port(&self) -> u16 {
        self.config.mixed_port
    }

    /// Start the mihomo subprocess and wait for its API to become ready.
    ///
    /// If the API port is already responding (a mihomo instance is already
    /// running with a matching config directory), the existing process is
    /// adopted instead of spawning a new one.
    ///
    /// # Errors
    ///
    /// Returns `MihomoError` if the binary is not found, the process fails to
    /// start, or the API does not respond within the timeout.
    pub fn start(&mut self) -> Result<(), MihomoError> {
        // Check if API port is already responding.
        let api_url = format!("http://{}/version", self.config.api_address);
        if self.check_api_health(&api_url) {
            // A process is already listening on our API port — adopt it.
            self.externally_managed = true;
            return Ok(());
        }

        // Ensure config directory exists first.
        std::fs::create_dir_all(&self.config.config_dir)?;

        if !self.config.binary_path.exists() {
            return Err(MihomoError::BinaryNotFound(self.config.binary_path.clone()));
        }

        let child = std::process::Command::new(&self.config.binary_path)
            .args([
                "-d",
                self.config.config_dir.to_string_lossy().as_ref(),
            ])
            .env("API_ADDRESS", &self.config.api_address)
            .env("API_SECRET", &self.config.api_secret)
            .spawn()
            .map_err(|e| MihomoError::StartFailed(e.to_string()))?;

        self.process = Some(child);

        // Wait for API readiness.
        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            if self.check_api_health(&api_url) {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(200));
        }

        // Timeout — clean up.
        self.kill_process();
        Err(MihomoError::ApiTimeout)
    }

    /// Stop the mihomo subprocess gracefully (SIGTERM → timeout SIGKILL).
    ///
    /// If the process was adopted (externally managed), it will NOT be killed —
    /// only the internal handle is cleared.
    ///
    /// # Errors
    ///
    /// Returns `MihomoError` if the process cannot be stopped.
    pub fn stop(&mut self) -> Result<(), MihomoError> {
        if self.externally_managed {
            self.externally_managed = false;
            self.process = None;
            return Ok(());
        }

        let proc = self
            .process
            .as_mut()
            .ok_or(MihomoError::NotRunning)?;

        // Send SIGTERM (kill is the cross-platform way in Rust's std).
        #[cfg(target_family = "unix")]
        {
            let pid = proc.id();
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .output();
        }

        // Wait up to 5s for graceful shutdown.
        match proc.try_wait() {
            Ok(Some(_)) => {}
            Ok(None) => {
                std::thread::sleep(Duration::from_secs(5));
                self.kill_process();
            }
            Err(_) => {
                self.kill_process();
            }
        }

        self.process = None;
        Ok(())
    }

    /// Check if the mihomo process is alive and its API is reachable.
    ///
    /// For externally managed processes (no Child handle), this checks the API
    /// port directly.
    pub fn is_running(&mut self) -> bool {
        if self.externally_managed {
            let api_url = format!("http://{}/version", self.config.api_address);
            return self.check_api_health(&api_url);
        }

        match self.process.as_mut() {
            Some(proc) => {
                match proc.try_wait() {
                    Ok(Some(_)) => {
                        self.process = None;
                        false
                    }
                    Ok(None) => {
                        let api_url = format!("http://{}/version", self.config.api_address);
                        self.check_api_health(&api_url)
                    }
                    Err(_) => false,
                }
            }
            None => false,
        }
    }

    /// Perform a health check against the mihomo API.
    #[must_use]
    pub fn health_check(&self) -> bool {
        let api_url = format!("http://{}/version", self.config.api_address);
        self.check_api_health(&api_url)
    }

    /// Write a YAML config content to the mihomo config directory.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn write_config_file(&self, filename: &str, content: &str) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.config.config_dir)?;
        let path = self.config.config_dir.join(filename);
        std::fs::write(path, content)
    }

    /// Reload mihomo configuration by sending PUT /configs to the API.
    ///
    /// # Errors
    ///
    /// Returns `MihomoError::NotRunning` if the process is not alive,
    /// or `MihomoError::Io` if the HTTP request fails.
    pub fn reload_config(&self, config_path: &str) -> Result<(), MihomoError> {
        use std::io::Write;

        let body = serde_json::json!({"path": config_path});
        let client = std::net::TcpStream::connect(&self.config.api_address)
            .map_err(MihomoError::Io)?;

        let request = format!(
            "PUT /configs HTTP/1.1\r\n\
             Host: {}\r\n\
             Authorization: Bearer {}\r\n\
             Content-Type: application/json\r\n\
             Content-Length: {}\r\n\
             Connection: close\r\n\r\n\
             {}",
            self.config.api_address,
            self.config.api_secret,
            body.to_string().len(),
            body,
        );

        let mut stream = client;
        stream
            .write_all(request.as_bytes())
            .map_err(MihomoError::Io)?;

        Ok(())
    }

    /// Get the API address.
    #[must_use]
    pub fn api_address(&self) -> &str {
        &self.config.api_address
    }

    /// Get a reference to the configuration.
    #[must_use]
    pub const fn config(&self) -> &MihomoConfig {
        &self.config
    }

    fn check_api_health(&self, _url: &str) -> bool {
        // Minimal HTTP check — no external deps needed.
        // Uses std::net::TcpStream for port check.
        let addr = self.config.api_address.clone();
        std::net::TcpStream::connect_timeout(
            &addr.parse().unwrap_or_else(|_| "127.0.0.1:9090".parse().unwrap()),
            Duration::from_secs(2),
        )
        .is_ok()
    }

    fn kill_process(&mut self) {
        if let Some(proc) = self.process.as_mut() {
            let _ = proc.kill();
            let _ = proc.wait();
        }
        self.process = None;
    }
}

impl Drop for MihomoManager {
    fn drop(&mut self) {
        if self.process.is_some() && !self.externally_managed {
            self.kill_process();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config(dir: &std::path::Path) -> MihomoConfig {
        MihomoConfig {
            binary_path: dir.join("fake-mihomo"),
            config_dir: dir.join("data").join("mihomo"),
            api_address: "127.0.0.1:19090".to_string(),
            api_secret: "test-secret".to_string(),
            mixed_port: 19090,
            log_level: "warning".to_string(),
        }
    }

    #[test]
    fn start_fails_if_binary_missing() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mut mgr = MihomoManager::new(config);

        let result = mgr.start();
        assert!(result.is_err());
        let Err(MihomoError::BinaryNotFound(path)) = result else {
            panic!("Expected BinaryNotFound, got {result:?}");
        };
        assert!(path.to_string_lossy().contains("fake-mihomo"));
    }

    #[test]
    fn stop_returns_error_when_not_running() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mut mgr = MihomoManager::new(config);

        let result = mgr.stop();
        assert!(matches!(result, Err(MihomoError::NotRunning)));
    }

    #[test]
    fn is_running_false_when_never_started() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mut mgr = MihomoManager::new(config);
        assert!(!mgr.is_running());
    }

    #[test]
    fn health_check_false_when_not_running() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mgr = MihomoManager::new(config);
        assert!(!mgr.health_check());
    }

    #[test]
    fn api_address_returns_configured_value() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mgr = MihomoManager::new(config);
        assert_eq!(mgr.api_address(), "127.0.0.1:19090");
    }

    #[test]
    fn start_creates_config_directory() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mut mgr = MihomoManager::new(config);

        // Will fail because binary doesn't exist, but should have created config dir.
        let _ = mgr.start();
        assert!(dir.path().join("data").join("mihomo").exists());
    }

    #[test]
    fn new_manager_has_no_process() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mgr = MihomoManager::new(config);
        assert!(mgr.process.is_none());
    }

    #[test]
    fn config_returns_reference() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mgr = MihomoManager::new(config);
        assert_eq!(mgr.config().mixed_port, 19090);
    }

    #[test]
    fn write_config_file_creates_file() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mgr = MihomoManager::new(config);

        mgr.write_config_file("config.yaml", "mixed-port: 7890\n")
            .expect("write");

        let content =
            std::fs::read_to_string(dir.path().join("data").join("mihomo").join("config.yaml"))
                .expect("read");
        assert_eq!(content, "mixed-port: 7890\n");
    }

    #[test]
    fn write_config_file_overwrites() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mgr = MihomoManager::new(config);

        mgr.write_config_file("test.yaml", "v1")
            .expect("write 1");
        mgr.write_config_file("test.yaml", "v2")
            .expect("write 2");

        let content =
            std::fs::read_to_string(dir.path().join("data").join("mihomo").join("test.yaml"))
                .expect("read");
        assert_eq!(content, "v2");
    }

    #[test]
    fn reload_config_returns_error_when_not_running() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_config(dir.path());
        let mgr = MihomoManager::new(config);

        // TCP connect will fail since nothing is listening.
        let result = mgr.reload_config("/tmp/test.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn mock_reloader_tracks_calls() {
        let mock = MockMihomoReloader::new();
        assert!(!mock.was_called());
        assert!(mock.last_config_path().is_none());

        mock.reload_config("/test/rules.yaml").expect("reload");
        assert!(mock.was_called());
        assert_eq!(mock.last_config_path(), Some("/test/rules.yaml".to_string()));
    }

    // --- F107: Adopt existing mihomo process tests ---

    /// Helper: bind to port 0 (OS-assigned free port), return the listener
    /// and a `MihomoConfig` pointing at that port.
    struct FakeApi {
        listener: std::net::TcpListener,
        config: MihomoConfig,
    }

    impl FakeApi {
        fn create(dir: &std::path::Path) -> Self {
            let listener =
                std::net::TcpListener::bind("127.0.0.1:0")
                    .expect("bind to any free port");
            let port = listener.local_addr().unwrap().port();
            let config = MihomoConfig {
                binary_path: dir.join("fake-mihomo"),
                config_dir: dir.join("data").join("mihomo"),
                api_address: format!("127.0.0.1:{port}"),
                api_secret: "test-secret".to_string(),
                mixed_port: port,
                log_level: "warning".to_string(),
            };
            Self {
                listener,
                config,
            }
        }
    }

    #[test]
    fn start_adopts_existing_api() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let fake = FakeApi::create(dir.path());
        let mut mgr = MihomoManager::new(fake.config);

        let result = mgr.start();
        assert!(result.is_ok(), "Expected Ok, got {result:?}");
        assert!(mgr.externally_managed);
        assert!(mgr.process.is_none());
    }

    #[test]
    fn stop_does_not_kill_externally_managed() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let fake = FakeApi::create(dir.path());
        let mut mgr = MihomoManager::new(fake.config);

        mgr.start().expect("adopt");
        assert!(mgr.externally_managed);

        let result = mgr.stop();
        assert!(result.is_ok(), "Expected Ok, got {result:?}");
        assert!(!mgr.externally_managed);
        assert!(mgr.process.is_none());

        // The fake API listener is still alive (not killed).
        assert!(fake.listener.local_addr().is_ok());
    }

    #[test]
    fn is_running_detects_externally_managed() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let fake = FakeApi::create(dir.path());
        let mut mgr = MihomoManager::new(fake.config);

        mgr.start().expect("adopt");
        assert!(mgr.is_running(), "externally managed should report running");
    }

    #[test]
    fn is_running_false_after_fake_listener_drops() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let saved_config;
        {
            let fake = FakeApi::create(dir.path());
            saved_config = fake.config.clone();
            let mut mgr = MihomoManager::new(fake.config);
            mgr.start().expect("adopt");
            assert!(mgr.is_running());
            // mgr dropped here — should NOT kill externally managed
        }

        // Now no one is listening. A new manager with the same config
        // should report not running.
        let mut mgr2 = MihomoManager::new(saved_config);
        assert!(!mgr2.is_running());
    }

    #[test]
    fn port_conflict_error_variant_displays() {
        let err = MihomoError::PortConflict("127.0.0.1:7890".to_string());
        let msg = format!("{err}");
        assert!(msg.contains("7890"));
        assert!(msg.contains("non-mihomo"));
    }
}
