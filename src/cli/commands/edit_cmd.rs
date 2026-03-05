use crate::config::Storage;
use crate::error::Result;
use colored::Colorize;
use std::process::Command;

/// Edit a profile's settings.json using the system editor.
///
/// # Errors
///
/// Returns an error if the profile doesn't exist or the editor fails to launch.
pub fn edit_profile(name: &str) -> Result<()> {
    let storage = Storage::new()?;

    // Check if profile exists
    if storage.get_profile(name)?.is_none() {
        return Err(crate::error::Error::ProfileNotFound(name.into()));
    }

    let settings_path = storage.profile_settings_path(name);

    // Determine the editor to use
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string());

    println!(
        "{} {} with {}...",
        "Opening".cyan(),
        name.green(),
        editor.yellow()
    );

    // Launch the editor
    let status = Command::new(&editor).arg(&settings_path).status()?;

    if !status.success() {
        return Err(crate::error::Error::ConfigError(format!(
            "Editor '{}' exited with non-zero status",
            editor
        )));
    }

    println!(
        "{} Profile '{}' edited successfully",
        "✓".green(),
        name.green()
    );

    Ok(())
}
