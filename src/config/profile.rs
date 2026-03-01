use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub display_name: Option<String>,
    pub env: HashMap<String, String>,
    pub permissions: Permissions,
    pub enabled_plugins: Option<HashMap<String, bool>>,
    pub always_thinking_enabled: Option<bool>,
    pub api_timeout_ms: Option<u64>,
    pub category: Option<String>,
    #[serde(default)]
    pub source: Option<ProfileSource>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileSource {
    CcSwitch,
    Manual,
}

impl Default for Profile {
    fn default() -> Self {
        Self {
            name: String::new(),
            display_name: None,
            env: HashMap::new(),
            permissions: Permissions::default(),
            enabled_plugins: None,
            always_thinking_enabled: None,
            api_timeout_ms: None,
            category: None,
            source: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Permissions {
    pub enabled: Option<bool>,
    #[serde(rename = "mcp")]
    pub mcp: Option<Vec<McpPermission>>,
    #[serde(rename = "command")]
    pub command: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPermission {
    pub name: String,
    pub enabled: Option<bool>,
}
