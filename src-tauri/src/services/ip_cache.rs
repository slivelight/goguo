use std::collections::HashMap;
use std::io;
use std::path::Path;

/// Default cache entry TTL: 24 hours in seconds.
const DEFAULT_TTL_SECS: i64 = 24 * 3600;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CacheEntry {
    pub ip: String,
    /// Unix timestamp (seconds) when this entry was scanned.
    pub scanned_at: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct IpCache {
    entries: HashMap<String, CacheEntry>,
}

impl IpCache {
    /// Load cache from a JSON file. Returns empty cache if the file doesn't exist
    /// or is malformed (silent fallback).
    #[must_use]
    pub fn load(path: &Path) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Persist cache to a JSON file.
    ///
    /// # Errors
    ///
    /// Returns an error if the parent directory cannot be created or the file cannot be written.
    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self)?;
        std::fs::write(path, json)
    }

    /// Get cached IP for a domain if it exists and hasn't expired (TTL 24h).
    #[must_use]
    pub fn get(&self, domain: &str) -> Option<&str> {
        let now = Self::now_secs();
        self.entries.get(domain).and_then(|entry| {
            if now - entry.scanned_at < DEFAULT_TTL_SECS {
                Some(entry.ip.as_str())
            } else {
                None
            }
        })
    }

    /// Get all valid (non-expired) cached entries as domain → IP mapping.
    #[must_use]
    pub fn get_all_valid(&self) -> HashMap<String, String> {
        let now = Self::now_secs();
        self.entries
            .iter()
            .filter(|(_, entry)| now - entry.scanned_at < DEFAULT_TTL_SECS)
            .map(|(domain, entry)| (domain.clone(), entry.ip.clone()))
            .collect()
    }

    /// Update or insert a cache entry with the current timestamp.
    pub fn update(&mut self, domain: String, ip: String) {
        self.entries.insert(
            domain,
            CacheEntry {
                ip,
                scanned_at: Self::now_secs(),
            },
        );
    }

    /// Remove cache entries for the specified domains.
    pub fn remove_domains(&mut self, domains: &[String]) {
        for domain in domains {
            self.entries.remove(domain);
        }
    }

    /// Check if any of the specified domains need scanning
    /// (not in cache or expired).
    #[must_use]
    pub fn needs_scan(&self, domains: &[String]) -> bool {
        let now = Self::now_secs();
        domains.iter().any(|domain| {
            self.entries
                .get(domain)
                .is_none_or(|entry| now - entry.scanned_at >= DEFAULT_TTL_SECS)
        })
    }

    /// Current time as Unix timestamp in seconds.
    fn now_secs() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs().cast_signed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn load_nonexistent_returns_empty() {
        let cache = IpCache::load(Path::new("/nonexistent/path/cache.json"));
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("ip-cache.json");

        let mut cache = IpCache::default();
        cache.update("github.com".to_string(), "1.2.3.4".to_string());
        cache.update("google.com".to_string(), "5.6.7.8".to_string());

        cache.save(&path).expect("save");
        let loaded = IpCache::load(&path);

        assert_eq!(loaded.get("github.com"), Some("1.2.3.4"));
        assert_eq!(loaded.get("google.com"), Some("5.6.7.8"));
    }

    #[test]
    fn load_malformed_json_returns_empty() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("cache.json");
        std::fs::write(&path, "not valid json{{{").expect("write");

        let cache = IpCache::load(&path);
        assert!(cache.entries.is_empty());
    }

    #[test]
    fn get_returns_valid_entry() {
        let mut cache = IpCache::default();
        cache.update("test.com".to_string(), "10.0.0.1".to_string());
        assert_eq!(cache.get("test.com"), Some("10.0.0.1"));
    }

    #[test]
    fn get_returns_none_for_missing() {
        let cache = IpCache::default();
        assert_eq!(cache.get("missing.com"), None);
    }

    #[test]
    fn get_returns_none_for_expired() {
        let mut cache = IpCache::default();
        // Manually insert an expired entry
        cache.entries.insert(
            "expired.com".to_string(),
            CacheEntry {
                ip: "1.1.1.1".to_string(),
                scanned_at: 0, // epoch → definitely expired
            },
        );
        assert_eq!(cache.get("expired.com"), None);
    }

    #[test]
    fn get_all_valid_filters_expired() {
        let mut cache = IpCache::default();
        cache.update("valid.com".to_string(), "1.1.1.1".to_string());
        cache.entries.insert(
            "expired.com".to_string(),
            CacheEntry {
                ip: "2.2.2.2".to_string(),
                scanned_at: 0,
            },
        );

        let all = cache.get_all_valid();
        assert_eq!(all.len(), 1);
        assert_eq!(all.get("valid.com"), Some(&"1.1.1.1".to_string()));
        assert!(all.get("expired.com").is_none());
    }

    #[test]
    fn update_overwrites_existing() {
        let mut cache = IpCache::default();
        cache.update("test.com".to_string(), "1.1.1.1".to_string());
        cache.update("test.com".to_string(), "2.2.2.2".to_string());
        assert_eq!(cache.get("test.com"), Some("2.2.2.2"));
    }

    #[test]
    fn remove_domains_clears_entries() {
        let mut cache = IpCache::default();
        cache.update("a.com".to_string(), "1.1.1.1".to_string());
        cache.update("b.com".to_string(), "2.2.2.2".to_string());
        cache.remove_domains(&["a.com".to_string()]);
        assert_eq!(cache.get("a.com"), None);
        assert_eq!(cache.get("b.com"), Some("2.2.2.2"));
    }

    #[test]
    fn needs_scan_true_for_missing() {
        let cache = IpCache::default();
        assert!(cache.needs_scan(&["github.com".to_string()]));
    }

    #[test]
    fn needs_scan_false_for_all_valid() {
        let mut cache = IpCache::default();
        cache.update("github.com".to_string(), "1.2.3.4".to_string());
        assert!(!cache.needs_scan(&["github.com".to_string()]));
    }

    #[test]
    fn needs_scan_true_if_any_expired() {
        let mut cache = IpCache::default();
        cache.update("valid.com".to_string(), "1.1.1.1".to_string());
        cache.entries.insert(
            "expired.com".to_string(),
            CacheEntry {
                ip: "2.2.2.2".to_string(),
                scanned_at: 0,
            },
        );
        assert!(cache.needs_scan(&["valid.com".to_string(), "expired.com".to_string()]));
    }

    #[test]
    fn save_creates_parent_directory() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("nested").join("dir").join("cache.json");

        let cache = IpCache::default();
        cache.save(&path).expect("save");
        assert!(path.exists());
    }

    #[test]
    fn empty_cache_needs_scan() {
        let cache = IpCache::default();
        assert!(cache.needs_scan(&["github.com".to_string(), "google.com".to_string()]));
    }
}
