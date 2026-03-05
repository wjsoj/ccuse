use crate::config::{Profile, Storage};
use crate::error::Result;
use colored::Colorize;
use inquire::Text;
use serde_json::json;
use std::env;
use std::fs;
use std::process::Command;

/// Get the system's default text editor
fn get_editor() -> String {
    // Try environment variables first
    if let Ok(editor) = env::var("VISUAL") {
        return editor;
    }
    if let Ok(editor) = env::var("EDITOR") {
        return editor;
    }

    // Platform-specific defaults
    #[cfg(target_os = "windows")]
    {
        "notepad.exe".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "open -e".to_string()
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Try common editors on Linux
        for editor in ["nano", "vim", "vi"] {
            if which::which(editor).is_ok() {
                return editor.to_string();
            }
        }
        "vi".to_string()
    }
}

/// Add a new profile interactively.
///
/// # Errors
///
/// Returns an error if profile already exists, user input fails, or profile cannot be saved.
pub fn add_profile() -> Result<()> {
    let storage = Storage::new()?;

    // Get profile name
    let name = Text::new("Profile name:").prompt()?;

    // Check if already exists
    if storage.get_profile(&name)?.is_some() {
        return Err(crate::error::Error::ProfileAlreadyExists(name));
    }

    // Create minimal template - only requires token and base_url
    let template = json!({
        "name": name,
        "source": "manual",
        "env": {
            "ANTHROPIC_AUTH_TOKEN": "",
            "ANTHROPIC_BASE_URL": ""
        }
    });

    // Create settings.json in profile directory
    let settings_path = storage.ensure_profile_settings_dir(&name)?;
    let original_content = serde_json::to_string_pretty(&template)?;
    fs::write(&settings_path, &original_content)?;

    println!("\n{} Opening editor to configure profile...", "→".cyan());
    println!("{} {}", "File:".bold(), settings_path.display());
    println!(
        "{} Save and close the editor when done. If you want to cancel, delete all content and save.\n",
        "Tip:".yellow()
    );

    // Open editor
    let editor = get_editor();
    let editor_parts: Vec<&str> = editor.split_whitespace().collect();
    let (cmd, args) = if editor_parts.len() > 1 {
        (editor_parts[0], &editor_parts[1..])
    } else {
        (editor_parts[0], &[][..])
    };

    let status = Command::new(cmd)
        .args(args)
        .arg(&settings_path)
        .status()
        .map_err(|e| {
            fs::remove_file(&settings_path).ok();
            storage.profile_settings_dir(&name).exists().then(|| {
                fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
            });
            crate::error::Error::ConfigError(format!("Failed to open editor: {e}"))
        })?;

    if !status.success() {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        return Err(crate::error::Error::ConfigError(
            "Editor exited with error".into(),
        ));
    }

    // Read and parse the edited file
    let content = fs::read_to_string(&settings_path)?;

    // Check if user deleted content (cancelled)
    if content.trim().is_empty() {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        println!("{}", "Profile creation cancelled.".yellow());
        return Ok(());
    }

    // Check if content unchanged (user didn't edit)
    if content.trim() == original_content.trim() {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        println!(
            "{}",
            "No changes made. Profile creation cancelled.".yellow()
        );
        return Ok(());
    }

    // Parse the edited content
    let user_json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        crate::error::Error::ConfigError(format!("Invalid JSON: {e}"))
    })?;

    // Build merged JSON - only include fields that are explicitly set
    let mut merged_json = serde_json::Map::new();
    merged_json.insert("name".to_string(), json!(name));

    // Always include env
    if let Some(env) = user_json.get("env") {
        merged_json.insert("env".to_string(), env.clone());
    } else {
        merged_json.insert("env".to_string(), json!({}));
    }

    // Only include optional fields if they are present and not null
    if let Some(permissions) = user_json.get("permissions") {
        if !permissions.is_null() {
            merged_json.insert("permissions".to_string(), permissions.clone());
        }
    }

    if let Some(enabled_plugins) = user_json.get("enabled_plugins") {
        if !enabled_plugins.is_null() {
            merged_json.insert("enabled_plugins".to_string(), enabled_plugins.clone());
        }
    }

    if let Some(always_thinking) = user_json.get("always_thinking_enabled") {
        if !always_thinking.is_null() {
            merged_json.insert(
                "always_thinking_enabled".to_string(),
                always_thinking.clone(),
            );
        }
    }

    if let Some(timeout) = user_json.get("api_timeout_ms") {
        if !timeout.is_null() {
            merged_json.insert("api_timeout_ms".to_string(), timeout.clone());
        }
    }

    if let Some(category) = user_json.get("category") {
        if !category.is_null() {
            merged_json.insert("category".to_string(), category.clone());
        }
    }

    // Include source
    if let Some(source) = user_json.get("source") {
        merged_json.insert("source".to_string(), source.clone());
    } else {
        merged_json.insert("source".to_string(), json!("manual"));
    }

    let profile: Profile = serde_json::from_value(json!(merged_json)).map_err(|e| {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        crate::error::Error::ConfigError(format!("Invalid JSON: {e}"))
    })?;

    // Validate that both token and base_url are provided
    let has_token = profile
        .env
        .get("ANTHROPIC_AUTH_TOKEN")
        .map(|v| !v.is_empty())
        .unwrap_or(false);
    let has_base_url = profile
        .env
        .get("ANTHROPIC_BASE_URL")
        .map(|v| !v.is_empty())
        .unwrap_or(false);

    if !has_token {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        return Err(crate::error::Error::ConfigError(
            "ANTHROPIC_AUTH_TOKEN is required".into(),
        ));
    }

    if !has_base_url {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        return Err(crate::error::Error::ConfigError(
            "ANTHROPIC_BASE_URL is required".into(),
        ));
    }

    // Save profile (the profile is already saved to settings.json earlier,
    // but we need to ensure it's properly saved with all fields)
    let settings_path = storage.ensure_profile_settings_dir(&name)?;
    fs::write(&settings_path, serde_json::to_string_pretty(&profile)?)?;

    println!(
        "{}",
        format!("✓ Profile '{name}' created successfully!").green()
    );

    Ok(())
}
