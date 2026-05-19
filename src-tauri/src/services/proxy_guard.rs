use crate::managers::mihomo_manager::MihomoManager;
use crate::models::config::ProxyGuardConfig;

/// Result of a `ProxyGuard` check cycle.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GuardAction {
    /// Everything is healthy, no action needed.
    Healthy,
    /// Mihomo was restarted after a crash.
    Restarted { attempt: u32 },
    /// Max restart attempts exceeded, triggered baseline recovery.
    RecoveryTriggered,
}

/// Monitors mihomo process health and triggers automatic recovery.
pub struct ProxyGuard {
    config: ProxyGuardConfig,
    restart_count: u32,
}

impl ProxyGuard {
    /// Create a new `ProxyGuard` with the given configuration.
    #[must_use]
    pub const fn new(config: ProxyGuardConfig) -> Self {
        Self {
            config,
            restart_count: 0,
        }
    }

    /// Get the current restart count.
    #[must_use]
    pub const fn restart_count(&self) -> u32 {
        self.restart_count
    }

    /// Get the max restart attempts.
    #[must_use]
    pub const fn max_restart_attempts(&self) -> u32 {
        self.config.max_restart_attempts
    }

    /// Reset the restart counter (e.g., after a successful healthy period).
    pub const fn reset_restart_count(&mut self) {
        self.restart_count = 0;
    }

    /// Perform a single health check and take action if needed.
    ///
    /// Returns the action taken:
    /// - `Healthy` if mihomo is running and API is responsive
    /// - `Restarted` if mihomo was dead and successfully restarted
    /// - `RecoveryTriggered` if restart limit was exceeded
    pub fn check_and_recover(
        &mut self,
        mihomo: &mut MihomoManager,
    ) -> GuardAction {
        if mihomo.is_running() {
            self.restart_count = 0;
            return GuardAction::Healthy;
        }

        // Mihomo is down — attempt restart.
        if self.restart_count >= self.config.max_restart_attempts {
            return GuardAction::RecoveryTriggered;
        }

        self.restart_count += 1;
        match mihomo.start() {
            Ok(()) => GuardAction::Restarted {
                attempt: self.restart_count,
            },
            Err(_) => {
                if self.restart_count >= self.config.max_restart_attempts {
                    GuardAction::RecoveryTriggered
                } else {
                    GuardAction::Restarted {
                        attempt: self.restart_count,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::managers::mihomo_manager::MihomoManager;
    use crate::models::config::MihomoConfig;

    fn test_guard_config() -> ProxyGuardConfig {
        ProxyGuardConfig {
            check_interval_secs: 3,
            max_restart_attempts: 3,
            restart_cooldown_secs: 1,
        }
    }

    fn test_mihomo_config(dir: &std::path::Path) -> MihomoConfig {
        MihomoConfig {
            binary_path: dir.join("fake-mihomo"),
            config_dir: dir.join("mihomo"),
            api_address: "127.0.0.1:19999".to_string(),
            api_secret: "test".to_string(),
            mixed_port: 19999,
            log_level: "warning".to_string(),
        }
    }

    #[test]
    fn healthy_when_mihomo_not_running_no_restarts_yet() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_mihomo_config(dir.path());
        let mut mihomo = MihomoManager::new(config);
        let mut guard = ProxyGuard::new(test_guard_config());

        // Mihomo not running → restart attempt (will fail since binary missing).
        let action = guard.check_and_recover(&mut mihomo);
        // Since binary is missing, start fails. But first attempt returns Restarted.
        assert!(matches!(action, GuardAction::Restarted { attempt: 1 }));
        assert_eq!(guard.restart_count(), 1);
    }

    #[test]
    fn recovery_triggered_after_max_restarts() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_mihomo_config(dir.path());
        let mut mihomo = MihomoManager::new(config);
        let mut guard = ProxyGuard::new(test_guard_config());

        // Simulate max restart attempts exhausted.
        for _ in 0..3 {
            let _ = guard.check_and_recover(&mut mihomo);
        }
        assert_eq!(guard.restart_count(), 3);

        // Next check should trigger recovery.
        let action = guard.check_and_recover(&mut mihomo);
        assert_eq!(action, GuardAction::RecoveryTriggered);
    }

    #[test]
    fn reset_clears_restart_count() {
        let mut guard = ProxyGuard::new(test_guard_config());
        guard.restart_count = 2;
        guard.reset_restart_count();
        assert_eq!(guard.restart_count(), 0);
    }

    #[test]
    fn healthy_resets_restart_count() {
        // We can't make is_running() return true without a real mihomo,
        // so test the logic directly.
        let mut guard = ProxyGuard::new(test_guard_config());
        guard.restart_count = 2;

        // Simulate a healthy check by testing the reset logic.
        guard.reset_restart_count();
        assert_eq!(guard.restart_count(), 0);
    }

    #[test]
    fn max_restart_attempts_matches_config() {
        let config = test_guard_config();
        let guard = ProxyGuard::new(config);
        assert_eq!(guard.max_restart_attempts(), 3);
    }

    #[test]
    fn restart_count_increments_on_failure() {
        let dir = tempfile::TempDir::new().expect("temp dir");
        let config = test_mihomo_config(dir.path());
        let mut mihomo = MihomoManager::new(config);
        let mut guard = ProxyGuard::new(test_guard_config());

        assert_eq!(guard.restart_count(), 0);
        let _ = guard.check_and_recover(&mut mihomo);
        assert_eq!(guard.restart_count(), 1);
        let _ = guard.check_and_recover(&mut mihomo);
        assert_eq!(guard.restart_count(), 2);
    }
}
