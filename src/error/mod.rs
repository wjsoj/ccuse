use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Profile not found: {0}")]
    ProfileNotFound(String),

    #[error("Profile already exists: {0}")]
    ProfileAlreadyExists(String),

    #[error("Failed to read CC-Switch database: {0}")]
    CcSwitchReadError(String),

    #[error("CC-Switch database not found")]
    CcSwitchDbNotFound,

    #[error("Failed to find Claude Code executable")]
    ClaudeNotFound,

    #[error("Failed to launch Claude Code: {0}")]
    LaunchError(String),

    #[error("Config error: {0}")]
    ConfigError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Database error: {0}")]
    DatabaseError(#[from] rusqlite::Error),

    #[error("Inquire error: {0}")]
    InquireError(#[from] inquire::InquireError),
}

pub type Result<T> = std::result::Result<T, Error>;
