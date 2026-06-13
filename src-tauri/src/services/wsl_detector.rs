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

    /// Return the Windows user profile directory via `/mnt/c/Users/<name>`.
    ///
    /// In WSL this maps to the Windows host's `%USERPROFILE%`.
    /// Returns `None` when not in WSL or the path cannot be discovered.
    fn windows_user_profile_dir(&self) -> Option<PathBuf>;
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

    fn windows_user_profile_dir(&self) -> Option<PathBuf> {
        std::fs::read_dir("/mnt/c/Users").ok().and_then(|entries| {
            let mut candidates: Vec<PathBuf> = Vec::new();
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with('.') || name == "Public" || name == "Default" || name == "Default User" || name == "All Users" {
                    continue;
                }
                let wslconfig_path = entry.path().join(".wslconfig");
                if wslconfig_path.exists() {
                    candidates.push(entry.path());
                }
            }
            candidates.first().cloned()
        })
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

/// Parses `ip -4 addr show eth0` output to extract the IPv4 address.
///
/// Expected line format: `inet 192.168.x.y/20 brd ... scope global eth0`
#[must_use]
pub fn parse_ip_addr_output(output: &str) -> Option<String> {
    for line in output.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("inet ") {
            let ip_part = rest.split('/').next()?;
            if ip_part != "127.0.0.1" && !ip_part.is_empty() {
                return Some(ip_part.to_string());
            }
        }
    }
    None
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
    /// Resolution strategy:
    /// 1. Check Linux `$HOME/.wslconfig` (inside WSL)
    /// 2. Fallback: check Windows `/mnt/c/Users/<name>/.wslconfig` (host side)
    /// 3. Default: NAT (WSL2 default when no config or no networkingMode key)
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
            .or_else(|| {
                self.reader
                    .windows_user_profile_dir()
                    .and_then(|win_profile| {
                        let config_path = win_profile.join(".wslconfig");
                        self.reader
                            .read_to_string(&config_path)
                            .ok()
                            .and_then(|content| parse_wslconfig_network_mode(&content))
                    })
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

    /// Returns the WSL instance's primary IP address (eth0).
    ///
    /// In NAT mode, this IP is reachable from the Windows host.
    /// In mirrored mode, `127.0.0.1` is shared and this method
    /// returns `None` (caller should use `127.0.0.1` instead).
    ///
    /// Returns `None` when not in WSL or the IP cannot be determined.
    #[must_use]
    pub fn get_wsl_ip(&self) -> Option<String> {
        if !self.is_running_in_wsl() {
            return None;
        }
        if self.detect_network_mode() == WslNetworkMode::Mirrored {
            return None;
        }
        get_wsl_eth0_ip()
    }
}

/// Returns the first non-loopback IPv4 address from WSL's eth0 interface.
///
/// Uses `/proc/net/fib_trail` or falls back to parsing `ip addr show eth0`.
/// Pure function with no dependency on `FileReader` — reads network state
/// directly from the kernel.
#[must_use]
pub fn get_wsl_eth0_ip() -> Option<String> {
    std::process::Command::new("ip")
        .args(["-4", "addr", "show", "eth0"])
        .output()
        .ok()
        .and_then(|out| {
            let stdout = String::from_utf8_lossy(&out.stdout);
            parse_ip_addr_output(&stdout)
        })
}

// ---------------------------------------------------------------------------
// Mock implementation (test-only)
// ---------------------------------------------------------------------------

#[cfg(test)]
#[derive(Default)]
pub struct MockFileReader {
    files: std::collections::HashMap<String, String>,
    home: Option<PathBuf>,
    win_profile: Option<PathBuf>,
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

    #[must_use]
    pub fn with_win_profile(mut self, path: PathBuf) -> Self {
        self.win_profile = Some(path);
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

    fn windows_user_profile_dir(&self) -> Option<PathBuf> {
        self.win_profile.clone()
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

    // ---- detect_network_mode fallback to Windows .wslconfig ---------------

    #[test]
    fn detector_detect_network_mode_mirrored_from_windows_side() {
        let mock = MockFileReader::new()
            .with_file(
                "/proc/version",
                "Linux version 5.15.133.1-microsoft-standard-WSL2",
            )
            .with_home(PathBuf::from("/home/test"))
            .with_win_profile(PathBuf::from("/mnt/c/Users/testuser"))
            .with_file(
                "/mnt/c/Users/testuser/.wslconfig",
                "[wsl2]\nnetworkingMode=mirrored\nmemory=4GB",
            );
        let detector = WslDetector::new(mock);
        assert_eq!(detector.detect_network_mode(), WslNetworkMode::Mirrored);
    }

    #[test]
    fn detector_detect_network_mode_prefers_linux_side() {
        let mock = MockFileReader::new()
            .with_file(
                "/proc/version",
                "Linux version 5.15.133.1-microsoft-standard-WSL2",
            )
            .with_file("/home/test/.wslconfig", "[wsl2]\nnetworkingMode=nat\n")
            .with_home(PathBuf::from("/home/test"))
            .with_win_profile(PathBuf::from("/mnt/c/Users/testuser"))
            .with_file(
                "/mnt/c/Users/testuser/.wslconfig",
                "[wsl2]\nnetworkingMode=mirrored\n",
            );
        let detector = WslDetector::new(mock);
        // Linux side should be preferred; it says NAT
        assert_eq!(detector.detect_network_mode(), WslNetworkMode::Nat);
    }

    #[test]
    fn detector_detect_network_mode_windows_side_no_config_means_nat() {
        let mock = MockFileReader::new()
            .with_file(
                "/proc/version",
                "Linux version 5.15.133.1-microsoft-standard-WSL2",
            )
            .with_win_profile(PathBuf::from("/mnt/c/Users/testuser"));
        // No .wslconfig on Windows side either → default NAT
        let detector = WslDetector::new(mock);
        assert_eq!(detector.detect_network_mode(), WslNetworkMode::Nat);
    }

    // ---- get_wsl_ip --------------------------------------------------------

    #[test]
    fn get_wsl_ip_returns_none_when_not_in_wsl() {
        let mock = MockFileReader::new();
        let detector = WslDetector::new(mock);
        assert!(detector.get_wsl_ip().is_none());
    }

    #[test]
    fn get_wsl_ip_returns_none_when_mirrored() {
        let mock = MockFileReader::new()
            .with_file(
                "/proc/version",
                "Linux version 5.15.133.1-microsoft-standard-WSL2",
            )
            .with_file("/home/test/.wslconfig", "[wsl2]\nnetworkingMode=mirrored\n")
            .with_home(PathBuf::from("/home/test"));
        let detector = WslDetector::new(mock);
        assert!(detector.get_wsl_ip().is_none());
    }

    // ---- parse_ip_addr_output -----------------------------------------------

    #[test]
    fn parse_ip_addr_output_extracts_ip() {
        let output = "3: eth0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 65575 qdisc noqueue state UP group default qlen 1000\n    inet 192.168.177.224/20 brd 192.168.191.255 scope global eth0\n    inet6 fe80::215:5dff:feed:c2e6/64 scope link eth0\n";
        assert_eq!(
            parse_ip_addr_output(output),
            Some("192.168.177.224".to_string())
        );
    }

    #[test]
    fn parse_ip_addr_output_skips_loopback() {
        let output = "    inet 127.0.0.1/8 scope host lo\n";
        assert!(parse_ip_addr_output(output).is_none());
    }

    #[test]
    fn parse_ip_addr_output_empty_returns_none() {
        assert!(parse_ip_addr_output("").is_none());
    }

    #[test]
    fn parse_ip_addr_output_multiple_inet_picks_first_non_loopback() {
        let output = "    inet 127.0.0.1/8 scope host lo\n    inet 192.168.1.100/24 brd 192.168.1.255 scope global eth0\n";
        assert_eq!(
            parse_ip_addr_output(output),
            Some("192.168.1.100".to_string())
        );
    }
}
