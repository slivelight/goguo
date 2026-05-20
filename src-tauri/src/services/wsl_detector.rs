//! WSL environment detection service.
//!
//! Detects whether the app is running inside Windows Subsystem for Linux,
//! determines the network mode (NAT / Mirrored), and extracts distro info.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The WSL networking mode reported by `.wslconfig`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum WslNetworkMode {
    /// WSL2 default networking — each distro gets a private NAT subnet.
    Nat,
    /// WSL2 mirrored mode — shares the host's network interfaces.
    Mirrored,
    /// WSL is not installed or we are not running inside WSL.
    NotInstalled,
}

/// Basic distribution information parsed from `/etc/os-release`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct DistroInfo {
    pub name: String,
    pub version: String,
    pub id: String,
    pub is_default: bool,
}

// ---------------------------------------------------------------------------
// File system abstraction (for testability)
// ---------------------------------------------------------------------------

/// Abstraction over file reads and home-directory queries used by [`WslDetector`].
pub trait FileReader {
    /// Read a file's full contents into a string.
    ///
    /// # Errors
    ///
    /// Returns an error string if the file cannot be read.
    fn read_to_string(&self, path: &Path) -> Result<String, String>;

    /// Return the current user's home directory, if discoverable.
    fn home_dir(&self) -> Option<PathBuf>;
}

/// Production implementation that delegates to `std::fs` and `std::env`.
pub struct SystemFileReader;

impl FileReader for SystemFileReader {
    fn read_to_string(&self, path: &Path) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    }

    fn home_dir(&self) -> Option<PathBuf> {
        std::env::var("HOME").ok().map(PathBuf::from)
    }
}

// ---------------------------------------------------------------------------
// Pure parsing functions
// ---------------------------------------------------------------------------

/// Returns `true` when `/proc/version` content indicates a WSL environment.
#[must_use]
pub fn parse_proc_version(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("microsoft") || lower.contains("wsl")
}

/// Extracts the `networkingMode` value from `.wslconfig` content.
///
/// Returns `None` when the key is absent or the section is missing.
#[must_use]
pub fn parse_wslconfig_network_mode(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("networkingMode") {
            // accept both `networkingMode=mirrored` and `networkingMode = mirrored`
            let value = rest.trim_start_matches([' ', '=', '\t']);
            if !value.is_empty() {
                return Some(value.to_string());
            }
        }
    }
    None
}

/// Parses key=value lines from an `/etc/os-release`-style file.
///
/// Returns `None` when `NAME` or `VERSION_ID` is missing.
#[must_use]
pub fn parse_os_release(content: &str) -> Option<DistroInfo> {
    let mut name: Option<String> = None;
    let mut version: Option<String> = None;
    let mut id: Option<String> = None;

    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(value) = strip_key(trimmed, "NAME") {
            name = Some(value);
        } else if let Some(value) = strip_key(trimmed, "VERSION_ID") {
            version = Some(value);
        } else if let Some(value) = strip_key(trimmed, "ID") {
            id = Some(value);
        }
    }

    let name = name?;
    let version = version?;
    let id = id.unwrap_or_else(|| name.to_lowercase());

    Some(DistroInfo {
        name,
        version,
        id,
        is_default: true,
    })
}

/// Strip `KEY=` from a line, returning the unquoted value, or `None`.
fn strip_key(line: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}=");
    line.strip_prefix(&prefix)
        .map(|rest| unquote(rest.trim()).to_string())
}

/// Remove surrounding double-quotes from a value, if present.
#[must_use]
fn unquote(s: &str) -> &str {
    s.strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(s)
}

// ---------------------------------------------------------------------------
// WslDetector
// ---------------------------------------------------------------------------

/// Detects the WSL environment, networking mode, and distribution info.
pub struct WslDetector<R: FileReader> {
    reader: R,
}

impl<R: FileReader> WslDetector<R> {
    #[must_use]
    pub const fn new(reader: R) -> Self {
        Self { reader }
    }

    /// Returns `true` when running inside a WSL instance.
    #[must_use]
    pub fn is_running_in_wsl(&self) -> bool {
        self.reader
            .read_to_string(Path::new("/proc/version"))
            .is_ok_and(|content| parse_proc_version(&content))
    }

    /// Determines the WSL networking mode.
    ///
    /// If we are not inside WSL, returns [`WslNetworkMode::NotInstalled`].
    #[must_use]
    pub fn detect_network_mode(&self) -> WslNetworkMode {
        if !self.is_running_in_wsl() {
            return WslNetworkMode::NotInstalled;
        }

        let mode = self
            .reader
            .home_dir()
            .and_then(|home| {
                let config_path = home.join(".wslconfig");
                self.reader
                    .read_to_string(&config_path)
                    .ok()
                    .and_then(|content| parse_wslconfig_network_mode(&content))
            })
            .unwrap_or_default();

        if mode == "mirrored" {
            WslNetworkMode::Mirrored
        } else {
            WslNetworkMode::Nat
        }
    }

    /// Reads `/etc/os-release` and returns distribution metadata.
    #[must_use]
    pub fn get_distro_info(&self) -> Option<DistroInfo> {
        self.reader
            .read_to_string(Path::new("/etc/os-release"))
            .ok()
            .and_then(|content| parse_os_release(&content))
    }
}

// ---------------------------------------------------------------------------
// Mock implementation (test-only)
// ---------------------------------------------------------------------------

#[cfg(test)]
#[derive(Default)]
pub struct MockFileReader {
    files: std::collections::HashMap<String, String>,
    home: Option<PathBuf>,
}

#[cfg(test)]
impl MockFileReader {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_file(mut self, path: &str, content: &str) -> Self {
        self.files.insert(path.to_string(), content.to_string());
        self
    }

    #[must_use]
    pub fn with_home(mut self, path: PathBuf) -> Self {
        self.home = Some(path);
        self
    }
}

#[cfg(test)]
impl FileReader for MockFileReader {
    fn read_to_string(&self, path: &Path) -> Result<String, String> {
        let key = path.to_string_lossy().to_string();
        self.files
            .get(&key)
            .cloned()
            .ok_or_else(|| format!("file not found: {key}"))
    }

    fn home_dir(&self) -> Option<PathBuf> {
        self.home.clone()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // ---- parse_proc_version ------------------------------------------------

    #[test]
    fn parse_proc_version_detects_microsoft() {
        let content = "Linux version 5.15.133.1-microsoft-standard-WSL2 (root@1234) (gcc 12)";
        assert!(parse_proc_version(content));
    }

    #[test]
    fn parse_proc_version_detects_wsl_lowercase() {
        let content = "Linux version 5.15.0-generic (wsl kernel build)";
        assert!(parse_proc_version(content));
    }

    #[test]
    fn parse_proc_version_rejects_non_wsl() {
        let content = "Linux version 6.1.0-generic (debian kernel)";
        assert!(!parse_proc_version(content));
    }

    #[test]
    fn parse_proc_version_empty_is_not_wsl() {
        assert!(!parse_proc_version(""));
    }

    // ---- parse_wslconfig_network_mode --------------------------------------

    #[test]
    fn parse_wslconfig_mirrored() {
        let content = "[wsl2]\nnetworkingMode=mirrored\nmemory=4GB";
        assert_eq!(
            parse_wslconfig_network_mode(content),
            Some("mirrored".to_string())
        );
    }

    #[test]
    fn parse_wslconfig_nat() {
        let content = "[wsl2]\nnetworkingMode=nat\nmemory=4GB";
        assert_eq!(
            parse_wslconfig_network_mode(content),
            Some("nat".to_string())
        );
    }

    #[test]
    fn parse_wslconfig_empty_returns_none() {
        assert_eq!(parse_wslconfig_network_mode(""), None);
    }

    #[test]
    fn parse_wslconfig_missing_key_returns_none() {
        let content = "[wsl2]\nmemory=4GB\nswap=2GB";
        assert_eq!(parse_wslconfig_network_mode(content), None);
    }

    // ---- parse_os_release --------------------------------------------------

    #[test]
    fn parse_os_release_ubuntu() {
        let content = r#"NAME="Ubuntu"
VERSION="22.04.3 LTS (Jammy Jellyfish)"
ID=ubuntu
ID_LIKE=debian
VERSION_ID="22.04"
PRETTY_NAME="Ubuntu 22.04.3 LTS"
"#;
        let info = parse_os_release(content).expect("should parse ubuntu");
        assert_eq!(info.name, "Ubuntu");
        assert_eq!(info.version, "22.04");
        assert_eq!(info.id, "ubuntu");
    }

    #[test]
    fn parse_os_release_debian() {
        let content = r#"PRETTY_NAME="Debian GNU/Linux 12 (bookworm)"
NAME="Debian GNU/Linux"
VERSION_ID="12"
VERSION="12 (bookworm)"
ID=debian
"#;
        let info = parse_os_release(content).expect("should parse debian");
        assert_eq!(info.name, "Debian GNU/Linux");
        assert_eq!(info.version, "12");
        assert_eq!(info.id, "debian");
    }

    #[test]
    fn parse_os_release_empty_returns_none() {
        assert!(parse_os_release("").is_none());
    }

    #[test]
    fn parse_os_release_missing_name_returns_none() {
        let content = "VERSION_ID=\"12\"\nID=debian\n";
        assert!(parse_os_release(content).is_none());
    }

    // ---- WslDetector integration via mock ----------------------------------

    #[test]
    fn detector_is_running_in_wsl_true() {
        let mock = MockFileReader::new().with_file(
            "/proc/version",
            "Linux version 5.15.133.1-microsoft-standard-WSL2",
        );
        let detector = WslDetector::new(mock);
        assert!(detector.is_running_in_wsl());
    }

    #[test]
    fn detector_is_running_in_wsl_false() {
        let mock = MockFileReader::new();
        let detector = WslDetector::new(mock);
        assert!(!detector.is_running_in_wsl());
    }

    #[test]
    fn detector_detect_network_mode_mirrored() {
        let mock = MockFileReader::new()
            .with_file(
                "/proc/version",
                "Linux version 5.15.133.1-microsoft-standard-WSL2",
            )
            .with_file("/home/test/.wslconfig", "[wsl2]\nnetworkingMode=mirrored\n")
            .with_home(PathBuf::from("/home/test"));
        let detector = WslDetector::new(mock);
        assert_eq!(detector.detect_network_mode(), WslNetworkMode::Mirrored);
    }

    #[test]
    fn detector_detect_network_mode_nat_default() {
        let mock = MockFileReader::new()
            .with_file(
                "/proc/version",
                "Linux version 5.15.133.1-microsoft-standard-WSL2",
            )
            .with_home(PathBuf::from("/home/test"));
        let detector = WslDetector::new(mock);
        assert_eq!(detector.detect_network_mode(), WslNetworkMode::Nat);
    }

    #[test]
    fn detector_detect_network_mode_not_installed() {
        let mock = MockFileReader::new();
        let detector = WslDetector::new(mock);
        assert_eq!(detector.detect_network_mode(), WslNetworkMode::NotInstalled);
    }

    #[test]
    fn detector_get_distro_info() {
        let mock = MockFileReader::new().with_file(
            "/etc/os-release",
            "NAME=\"Ubuntu\"\nVERSION_ID=\"22.04\"\nID=ubuntu\n",
        );
        let detector = WslDetector::new(mock);
        let info = detector.get_distro_info().expect("should return distro");
        assert_eq!(info.name, "Ubuntu");
        assert_eq!(info.version, "22.04");
        assert_eq!(info.id, "ubuntu");
    }

    #[test]
    fn detector_get_distro_info_missing_file() {
        let mock = MockFileReader::new();
        let detector = WslDetector::new(mock);
        assert!(detector.get_distro_info().is_none());
    }
}
