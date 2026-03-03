use crate::config::Storage;
use crate::error::Result;
use std::fs;
use std::path::PathBuf;

/// Load a profile's settings to replace the default Claude settings.
///
/// # Errors
///
/// Returns an error if profile does not exist, settings cannot be read, or writing fails.
pub fn load_profile(name: &str, backup: bool) -> Result<()> {
    let storage = Storage::new()?;

    // Get profile settings path
    let profile_settings_path = storage.profile_settings_path(name);

    if !profile_settings_path.exists() {
        return Err(crate::error::Error::ConfigError(format!(
            "Settings file not found for profile '{}': {}",
            name,
            profile_settings_path.display()
        )));
    }

    // Read profile settings content
    let content = fs::read_to_string(&profile_settings_path)?;

    // Get Claude config directory (~/.claude)
    let claude_dir = get_claude_dir()?;

    // Check if settings.json already exists
    let settings_path = claude_dir.join("settings.json");

    // Handle backup if requested
    if backup && settings_path.exists() {
        let backup_path = claude_dir.join("settings.json.backup");
        fs::copy(&settings_path, &backup_path)?;
        println!("Backed up existing settings to: {}", backup_path.display());
    }

    // Write new settings
    fs::write(&settings_path, &content)?;

    println!(
        "Loaded settings from profile '{}' to: {}",
        name,
        settings_path.display()
    );

    Ok(())
}

/// Get the Claude config directory (~/.claude).
///
/// # Errors
///
/// Returns an error if the home directory cannot be determined.
fn get_claude_dir() -> Result<PathBuf> {
    let home = std::env::var_os("HOME").ok_or_else(|| {
        crate::error::Error::ConfigError("Failed to determine home directory".into())
    })?;

    let claude_dir = PathBuf::from(home).join(".claude");

    // Create directory if it doesn't exist
    if !claude_dir.exists() {
        fs::create_dir_all(&claude_dir)?;
    }

    Ok(claude_dir)
}
