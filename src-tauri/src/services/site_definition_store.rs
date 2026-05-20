use crate::models::site::SiteDefinition;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;

pub struct SiteDefinitionStore {
    built_in: HashMap<String, SiteDefinition>,
    custom_dir: PathBuf,
}

impl SiteDefinitionStore {
    #[must_use]
    pub fn new(custom_dir: PathBuf) -> Self {
        let mut built_in = HashMap::new();
        built_in.insert("github".to_string(), SiteDefinition::github_default());
        built_in.insert("npmjs".to_string(), SiteDefinition::npmjs_default());
        built_in.insert("claude".to_string(), SiteDefinition::claude_default());
        built_in.insert("chatgpt".to_string(), SiteDefinition::chatgpt_default());
        built_in.insert("docker".to_string(), SiteDefinition::docker_default());
        built_in.insert("google".to_string(), SiteDefinition::google_default());
        built_in.insert("stackoverflow".to_string(), SiteDefinition::stackoverflow_default());
        built_in.insert("pypi".to_string(), SiteDefinition::pypi_default());
        built_in.insert("crates".to_string(), SiteDefinition::crates_default());
        built_in.insert("oracle".to_string(), SiteDefinition::oracle_default());
        built_in.insert("wikipedia".to_string(), SiteDefinition::wikipedia_default());
        built_in.insert("whatsapp".to_string(), SiteDefinition::whatsapp_default());
        built_in.insert("instagram".to_string(), SiteDefinition::instagram_default());
        built_in.insert("canva".to_string(), SiteDefinition::canva_default());
        built_in.insert("twitter-x".to_string(), SiteDefinition::twitter_x_default());

        Self { built_in, custom_dir }
    }

    #[must_use]
    pub fn get(&self, id: &str) -> Option<SiteDefinition> {
        if let Some(site) = self.built_in.get(id) {
            return Some(site.clone());
        }
        self.load_custom(id).ok().flatten()
    }

    #[must_use]
    pub fn list_all(&self) -> Vec<SiteDefinition> {
        let mut result: Vec<SiteDefinition> = self.built_in.values().cloned().collect();
        
        if let Ok(custom_ids) = self.list_custom_ids() {
            for id in custom_ids {
                if !self.built_in.contains_key(&id) {
                    if let Ok(Some(site)) = self.load_custom(&id) {
                        result.push(site);
                    }
                }
            }
        }
        
        result.sort_by(|a, b| a.id.cmp(&b.id));
        result
    }

    /// Saves a custom site definition to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the custom directory cannot be created,
    /// or if the JSON file cannot be written.
    pub fn save_custom(&self, site: &SiteDefinition) -> io::Result<()> {
        fs::create_dir_all(&self.custom_dir)?;
        let path = self.custom_dir.join(format!("{}.json", site.id));
        let json = serde_json::to_string_pretty(site)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Deletes a custom site definition from disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be removed.
    ///
    /// # Returns
    ///
    /// `true` if a custom definition was deleted, `false` if the ID is built-in
    /// or no custom definition exists.
    pub fn delete_custom(&self, id: &str) -> io::Result<bool> {
        if self.built_in.contains_key(id) {
            return Ok(false);
        }
        
        let path = self.custom_dir.join(format!("{id}.json"));
        if path.exists() {
            fs::remove_file(path)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn load_custom(&self, id: &str) -> io::Result<Option<SiteDefinition>> {
        let path = self.custom_dir.join(format!("{id}.json"));
        if !path.exists() {
            return Ok(None);
        }
        
        let json = fs::read_to_string(path)?;
        let site: SiteDefinition = serde_json::from_str(&json)?;
        Ok(Some(site))
    }

    fn list_custom_ids(&self) -> io::Result<Vec<String>> {
        if !self.custom_dir.exists() {
            return Ok(vec![]);
        }
        
        let mut ids = vec![];
        for entry in fs::read_dir(&self.custom_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().is_some_and(|ext| ext == "json") {
                if let Some(stem) = path.file_stem() {
                    if let Some(id) = stem.to_str() {
                        ids.push(id.to_string());
                    }
                }
            }
        }
        Ok(ids)
    }

    #[must_use]
    pub fn built_in_ids(&self) -> Vec<String> {
        self.built_in.keys().cloned().collect()
    }

    #[must_use]
    pub fn developer_template_ids() -> Vec<String> {
        vec![
            "github".to_string(),
            "npmjs".to_string(),
            "claude".to_string(),
            "chatgpt".to_string(),
            "oracle".to_string(),
            "docker".to_string(),
            "stackoverflow".to_string(),
            "pypi".to_string(),
            "crates".to_string(),
        ]
    }

    #[must_use]
    pub fn office_template_ids() -> Vec<String> {
        vec![
            "google".to_string(),
            "wikipedia".to_string(),
            "whatsapp".to_string(),
            "instagram".to_string(),
            "canva".to_string(),
            "twitter-x".to_string(),
        ]
    }

    #[must_use]
    pub fn load_developer_template(&self) -> Vec<SiteDefinition> {
        Self::developer_template_ids()
            .iter()
            .filter_map(|id| self.get(id))
            .collect()
    }

    #[must_use]
    pub fn load_office_template(&self) -> Vec<SiteDefinition> {
        Self::office_template_ids()
            .iter()
            .filter_map(|id| self.get(id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::site::{DomainCategory, HealthCheckConfig};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn store_built_in_ids_count() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        let ids = store.built_in_ids();
        assert_eq!(ids.len(), 15);
        assert!(ids.contains(&"github".to_string()));
        assert!(ids.contains(&"chatgpt".to_string()));
        assert!(ids.contains(&"stackoverflow".to_string()));
        assert!(ids.contains(&"twitter-x".to_string()));
    }

    #[test]
    fn store_get_built_in() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        
        let github = store.get("github").expect("github exists");
        assert_eq!(github.id, "github");
        assert_eq!(github.name, "GitHub");
        assert!(github.domain_count() >= 5);
    }

    #[test]
    fn store_get_missing_returns_none() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        assert!(store.get("nonexistent").is_none());
    }

    #[test]
    fn store_list_all_built_in_only() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        let all = store.list_all();
        assert_eq!(all.len(), 15);
    }

    #[test]
    fn store_save_and_load_custom() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec!["custom.example.com".to_string()]);
        
        let custom = SiteDefinition {
            id: "custom".to_string(),
            name: "Custom".to_string(),
            domains,
            health_check: Some(HealthCheckConfig {
                url: "https://custom.example.com".to_string(),
                timeout_secs: 5,
                failure_threshold: 3,
            }),
        };
        
        store.save_custom(&custom).expect("save");
        
        let loaded = store.get("custom").expect("loaded");
        assert_eq!(loaded.id, "custom");
        assert_eq!(loaded.name, "Custom");
        assert!(loaded.all_domains().contains(&"custom.example.com".to_string()));
    }

    #[test]
    fn store_list_all_includes_custom() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec!["custom.com".to_string()]);
        
        let custom = SiteDefinition {
            id: "custom".to_string(),
            name: "Custom".to_string(),
            domains,
            health_check: None,
        };
        
        store.save_custom(&custom).expect("save");
        let all = store.list_all();
        assert_eq!(all.len(), 16);
        assert!(all.iter().any(|s| s.id == "custom"));
    }

    #[test]
    fn store_delete_custom() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec!["custom.com".to_string()]);
        
        let custom = SiteDefinition {
            id: "custom".to_string(),
            name: "Custom".to_string(),
            domains,
            health_check: None,
        };
        
        store.save_custom(&custom).expect("save");
        assert!(store.get("custom").is_some());
        
        let deleted = store.delete_custom("custom").expect("delete");
        assert!(deleted);
        assert!(store.get("custom").is_none());
    }

    #[test]
    fn store_delete_built_in_fails() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        
        let deleted = store.delete_custom("github").expect("delete");
        assert!(!deleted);
        assert!(store.get("github").is_some());
    }

    #[test]
    fn store_delete_nonexistent() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        
        let deleted = store.delete_custom("nonexistent").expect("delete");
        assert!(!deleted);
    }

    #[test]
    fn developer_template_ids_count() {
        let ids = SiteDefinitionStore::developer_template_ids();
        assert_eq!(ids.len(), 9);
        assert!(ids.contains(&"github".to_string()));
        assert!(ids.contains(&"npmjs".to_string()));
        assert!(ids.contains(&"claude".to_string()));
        assert!(ids.contains(&"chatgpt".to_string()));
        assert!(ids.contains(&"docker".to_string()));
        assert!(ids.contains(&"stackoverflow".to_string()));
        assert!(ids.contains(&"pypi".to_string()));
        assert!(ids.contains(&"crates".to_string()));
        assert!(ids.contains(&"oracle".to_string()));
    }

    #[test]
    fn office_template_ids_count() {
        let ids = SiteDefinitionStore::office_template_ids();
        assert_eq!(ids.len(), 6);
        assert!(ids.contains(&"google".to_string()));
        assert!(ids.contains(&"wikipedia".to_string()));
        assert!(ids.contains(&"whatsapp".to_string()));
        assert!(ids.contains(&"instagram".to_string()));
        assert!(ids.contains(&"canva".to_string()));
        assert!(ids.contains(&"twitter-x".to_string()));
    }

    #[test]
    fn load_developer_template_returns_all_sites() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        let sites = store.load_developer_template();
        assert_eq!(sites.len(), 9);
        assert!(sites.iter().any(|s| s.id == "github"));
        assert!(sites.iter().any(|s| s.id == "chatgpt"));
        assert!(sites.iter().any(|s| s.id == "stackoverflow"));
    }

    #[test]
    fn load_office_template_returns_all_sites() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        let sites = store.load_office_template();
        assert_eq!(sites.len(), 6);
        assert!(sites.iter().any(|s| s.id == "google"));
        assert!(sites.iter().any(|s| s.id == "wikipedia"));
        assert!(sites.iter().any(|s| s.id == "twitter-x"));
    }

    #[test]
    fn template_sites_domain_count() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        
        let dev_sites = store.load_developer_template();
        let total_domains: usize = dev_sites.iter().map(SiteDefinition::domain_count).sum();
        assert!(total_domains >= 20, "developer template should have at least 20 domains");
        
        let office_sites = store.load_office_template();
        let office_domains: usize = office_sites.iter().map(SiteDefinition::domain_count).sum();
        assert!(office_domains >= 10, "office template should have at least 10 domains");
    }

    #[test]
    fn store_custom_overrides_same_id_not_allowed() {
        let dir = tempdir().expect("tempdir");
        let store = SiteDefinitionStore::new(dir.path().to_path_buf());
        
        let mut domains = HashMap::new();
        domains.insert(DomainCategory::Core, vec!["evil.com".to_string()]);
        
        let fake_github = SiteDefinition {
            id: "github".to_string(),
            name: "Evil GitHub".to_string(),
            domains,
            health_check: None,
        };
        
        store.save_custom(&fake_github).expect("save");
        
        let loaded = store.get("github").expect("loaded");
        assert_eq!(loaded.name, "GitHub");
        assert!(loaded.all_domains().contains(&"github.com".to_string()));
    }
}