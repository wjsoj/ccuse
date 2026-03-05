use crate::config::{Profile, ProfileSource};
use crate::error::{Error, Result};
use rusqlite::Connection;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct CcSwitchDb {
    db_path: PathBuf,
}

impl CcSwitchDb {
    /// Create a new `CcSwitchDb` instance.
    ///
    /// # Errors
    ///
    /// Returns an error if home directory cannot be found or CC-Switch database does not exist.
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| Error::ConfigError("Cannot find home directory".into()))?;

        let db_path = home.join(".cc-switch").join("cc-switch.db");

        if !db_path.exists() {
            return Err(Error::CcSwitchDbNotFound);
        }

        Ok(Self { db_path })
    }

    #[must_use]
    pub fn exists() -> bool {
        dirs::home_dir().is_some_and(|home| {
            let db_path = home.join(".cc-switch").join("cc-switch.db");
            db_path.exists()
        })
    }

    /// Get all Claude profiles from CC-Switch database.
    ///
    /// # Errors
    ///
    /// Returns an error if database cannot be opened or queried.
    pub fn get_profiles(&self) -> Result<Vec<Profile>> {
        let conn = Connection::open(&self.db_path)?;

        let mut stmt = conn.prepare(
            "SELECT id, name, settings_config
             FROM providers
             WHERE app_type = 'claude'",
        )?;

        let profiles = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let name: String = row.get(1)?;
                let settings_config: String = row.get(2)?;

                Ok((id, name, settings_config))
            })?
            .filter_map(std::result::Result::ok)
            .filter_map(|(id, name, settings_config)| {
                Self::parse_provider_config(&id, &name, &settings_config).ok()
            })
            .collect();

        Ok(profiles)
    }

    fn parse_provider_config(_id: &str, name: &str, settings_config: &str) -> Result<Profile> {
        #[derive(Deserialize)]
        struct ProviderConfig {
            #[serde(rename = "env")]
            env: Option<HashMap<String, String>>,
            #[serde(rename = "permissions")]
            permissions: Option<crate::config::Permissions>,
            #[serde(rename = "enabledPlugins")]
            enabled_plugins: Option<HashMap<String, bool>>,
            #[serde(rename = "alwaysThinkingEnabled")]
            always_thinking_enabled: Option<bool>,
            #[serde(rename = "apiTimeoutMs")]
            api_timeout_ms: Option<u64>,
        }

        let config: ProviderConfig = serde_json::from_str(settings_config).map_err(|e| {
            Error::CcSwitchReadError(format!("Failed to parse settings_config: {e}"))
        })?;

        // Replace spaces with underscores in profile name
        let name = name.replace(' ', "_");

        Ok(Profile {
            name,
            env: config.env.unwrap_or_default(),
            permissions: config.permissions.unwrap_or_default(),
            enabled_plugins: config.enabled_plugins,
            always_thinking_enabled: config.always_thinking_enabled,
            api_timeout_ms: config.api_timeout_ms,
            category: None,
            source: Some(ProfileSource::CcSwitch),
        })
    }
}

impl Default for CcSwitchDb {
    fn default() -> Self {
        // Safe default - uses a dummy path that won't exist
        Self {
            db_path: PathBuf::new(),
        }
    }
}
