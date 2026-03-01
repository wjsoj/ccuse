use crate::config::Storage;
use crate::error::Result;
use colored::Colorize;
use std::fs;

/// Rename a profile.
///
/// # Errors
///
/// Returns an error if old profile does not exist, new name already exists, or profile cannot be updated.
pub fn rename_profile(old_name: &str, new_name: &str) -> Result<()> {
    let storage = Storage::new()?;

    // Check if old profile exists
    let Some(mut profile) = storage.get_profile(old_name)? else {
        return Err(crate::error::Error::ProfileNotFound(old_name.into()));
    };

    // Check if new name already exists
    if storage.get_profile(new_name)?.is_some() {
        return Err(crate::error::Error::ProfileAlreadyExists(new_name.into()));
    }

    // Rename the profile directory
    let old_dir = storage.profile_settings_dir(old_name);
    let new_dir = storage.profile_settings_dir(new_name);

    // If destination directory exists (orphaned data), remove it first
    if new_dir.exists() {
        fs::remove_dir_all(&new_dir)?;
    }

    if old_dir.exists() {
        fs::rename(&old_dir, &new_dir)?;
    }

    // Update profile name in settings.json
    profile.name = new_name.to_string();
    if profile.display_name.is_some() {
        profile.display_name = Some(new_name.to_string());
    }

    // Save updated profile to new location
    let settings_path = storage.ensure_profile_settings_dir(new_name)?;
    fs::write(&settings_path, serde_json::to_string_pretty(&profile)?)?;

    // Remove old profile directory
    let old_dir = storage.profile_settings_dir(old_name);
    if old_dir.exists() {
        fs::remove_dir_all(&old_dir)?;
    }

    println!(
        "{}",
        format!("Profile '{old_name}' renamed to '{new_name}' successfully.").green()
    );

    Ok(())
}
