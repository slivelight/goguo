//! WSL network proxy strategy determination.
//!
//! Given a [`WslNetworkMode`] and a reachability flag, decides whether the
//! proxy should be explicitly configured, skipped, or fall back to explicit
//! configuration with a reason.

#[cfg(target_os = "linux")]
use serde::{Deserialize, Serialize};

#[cfg(target_os = "linux")]
use super::wsl_detector::WslNetworkMode;

/// Strategy for proxy configuration in a WSL environment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProxyStrategy {
    /// NAT mode: need explicit proxy config for all WSL items
    ExplicitConfig,
    /// Mirrored mode + reachable: skip proxy config
    SkipConfig,
    /// Mirrored mode but unreachable: fall back to explicit config
    FallbackToExplicit { reason: String },
}

/// Determines the proxy strategy based on WSL network mode and reachability.
///
/// This is a pure function with no side effects.
///
/// # Logic
///
/// - `NotInstalled` → `ExplicitConfig` (fallback)
/// - `Nat` → `ExplicitConfig`
/// - `Mirrored` + reachable → `SkipConfig`
/// - `Mirrored` + not reachable → `FallbackToExplicit { reason }`
#[must_use]
pub fn determine_strategy(network_mode: &WslNetworkMode, reachable: bool) -> ProxyStrategy {
    match (network_mode, reachable) {
        (WslNetworkMode::Mirrored, true) => ProxyStrategy::SkipConfig,
        (WslNetworkMode::Mirrored, false) => ProxyStrategy::FallbackToExplicit {
            reason: "host proxy unreachable on mirrored interface".to_string(),
        },
        (WslNetworkMode::Nat | WslNetworkMode::NotInstalled, _) => ProxyStrategy::ExplicitConfig,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Core strategy logic ------------------------------------------------

    #[test]
    fn nat_mode_returns_explicit_config() {
        let strategy = determine_strategy(&WslNetworkMode::Nat, true);
        assert_eq!(strategy, ProxyStrategy::ExplicitConfig);
    }

    #[test]
    fn mirrored_reachable_returns_skip_config() {
        let strategy = determine_strategy(&WslNetworkMode::Mirrored, true);
        assert_eq!(strategy, ProxyStrategy::SkipConfig);
    }

    #[test]
    fn mirrored_unreachable_returns_fallback_to_explicit() {
        let strategy = determine_strategy(&WslNetworkMode::Mirrored, false);
        match strategy {
            ProxyStrategy::FallbackToExplicit { reason } => {
                assert!(!reason.is_empty(), "reason should not be empty");
            }
            other => panic!("expected FallbackToExplicit, got {other:?}"),
        }
    }

    #[test]
    fn not_installed_returns_explicit_config() {
        let strategy = determine_strategy(&WslNetworkMode::NotInstalled, true);
        assert_eq!(strategy, ProxyStrategy::ExplicitConfig);
    }

    // ---- Serde roundtrip ----------------------------------------------------

    #[test]
    fn serde_roundtrip_explicit_config() {
        let original = ProxyStrategy::ExplicitConfig;
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: ProxyStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn serde_roundtrip_skip_config() {
        let original = ProxyStrategy::SkipConfig;
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: ProxyStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }

    #[test]
    fn serde_roundtrip_fallback_to_explicit() {
        let original = ProxyStrategy::FallbackToExplicit {
            reason: "host proxy unreachable on mirrored interface".to_string(),
        };
        let json = serde_json::to_string(&original).expect("serialize");
        let decoded: ProxyStrategy = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(original, decoded);
    }
}
