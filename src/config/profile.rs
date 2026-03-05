use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Profile {
    pub name: String,
    pub env: HashMap<String, String>,
    pub permissions: Permissions,
    pub enabled_plugins: Option<HashMap<String, bool>>,
    pub always_thinking_enabled: Option<bool>,
    pub api_timeout_ms: Option<u64>,
    pub category: Option<String>,
    #[serde(default)]
    pub source: Option<ProfileSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileSource {
    CcSwitch,
    Manual,
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
