use crate::models::baseline::{Platform, StateItem, StateItemCategory};

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "linux")]
pub mod linux_base;

#[cfg(target_os = "linux")]
pub mod wsl;

#[cfg(target_os = "linux")]
#[allow(unused_imports)] // Re-exported for downstream adapters (WslAdapter, LinuxAdapter)
pub(crate) use linux_base::ShellExecutor;

/// Metadata describing a state item that a platform adapter can detect or restore.
pub struct StateItemDefinition {
    pub id: String,
    pub category: StateItemCategory,
    pub description: String,
}

/// Abstraction over platform-specific state item read/write operations.
///
/// Each adapter targets one platform (Windows, WSL, Linux) and knows how to
/// read and optionally write specific network configuration state items.
pub trait PlatformAdapter: Send + Sync {
    /// Returns the target platform for this adapter.
    fn platform(&self) -> Platform;

    /// Returns metadata for all state items this adapter can detect.
    fn state_item_definitions(&self) -> Vec<StateItemDefinition>;

    /// Read all detectable state items from the current platform.
    ///
    /// Returns a `StateItem` for each definition. If reading a specific item
    /// fails, it should still be included with an error description in the value.
    fn read_state_items(&self) -> Vec<StateItem>;

    /// Write a single state item value back to the platform.
    ///
    /// Only items classified as `Restorable` should be passed here.
    ///
    /// # Errors
    ///
    /// Returns an error string if the item is not restorable or the write fails.
    fn write_state(&self, item: &StateItem) -> Result<(), String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A minimal mock adapter for verifying the trait interface.
    struct MockAdapter {
        platform: Platform,
        items: Vec<StateItem>,
    }

    impl MockAdapter {
        fn new(platform: Platform) -> Self {
            Self {
                platform,
                items: vec![],
            }
        }

        fn with_items(mut self, items: Vec<StateItem>) -> Self {
            self.items = items;
            self
        }
    }

    impl PlatformAdapter for MockAdapter {
        fn platform(&self) -> Platform {
            self.platform.clone()
        }

        fn state_item_definitions(&self) -> Vec<StateItemDefinition> {
            vec![StateItemDefinition {
                id: "mock-item".to_string(),
                category: StateItemCategory::Restorable,
                description: "Mock state item for testing".to_string(),
            }]
        }

        fn read_state_items(&self) -> Vec<StateItem> {
            self.items.clone()
        }

        fn write_state(&self, item: &StateItem) -> Result<(), String> {
            if item.category != StateItemCategory::Restorable {
                return Err(format!("Cannot write non-restorable item: {}", item.id));
            }
            Ok(())
        }
    }

    #[test]
    fn mock_adapter_returns_platform() {
        let adapter = MockAdapter::new(Platform::Windows);
        assert_eq!(adapter.platform(), Platform::Windows);
    }

    #[test]
    fn mock_adapter_returns_definitions() {
        let adapter = MockAdapter::new(Platform::Wsl);
        let defs = adapter.state_item_definitions();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].id, "mock-item");
        assert_eq!(defs[0].category, StateItemCategory::Restorable);
    }

    #[test]
    fn mock_adapter_reads_items() {
        let items = vec![StateItem {
            id: "test-proxy".to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({"ProxyEnable": 0}),
            collected_at: "2026-05-19T12:00:00Z".to_string(),
            classification_reason: "test".to_string(),
        }];
        let adapter = MockAdapter::new(Platform::Windows).with_items(items);
        let read = adapter.read_state_items();
        assert_eq!(read.len(), 1);
        assert_eq!(read[0].id, "test-proxy");
    }

    #[test]
    fn mock_adapter_writes_restorable_item() {
        let item = StateItem {
            id: "test-proxy".to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Restorable,
            value: serde_json::json!({"ProxyEnable": 1}),
            collected_at: "2026-05-19T12:00:00Z".to_string(),
            classification_reason: "test".to_string(),
        };
        let adapter = MockAdapter::new(Platform::Windows);
        assert!(adapter.write_state(&item).is_ok());
    }

    #[test]
    fn mock_adapter_rejects_non_restorable_write() {
        let item = StateItem {
            id: "test-dns".to_string(),
            platform: Platform::Windows,
            category: StateItemCategory::Detectable,
            value: serde_json::json!("cache data"),
            collected_at: "2026-05-19T12:00:00Z".to_string(),
            classification_reason: "test".to_string(),
        };
        let adapter = MockAdapter::new(Platform::Windows);
        assert!(adapter.write_state(&item).is_err());
    }

    #[test]
    fn trait_object_dispatch_works() {
        let adapter: Box<dyn PlatformAdapter> = Box::new(MockAdapter::new(Platform::Linux));
        assert_eq!(adapter.platform(), Platform::Linux);
        let defs = adapter.state_item_definitions();
        assert!(!defs.is_empty());
    }
}
