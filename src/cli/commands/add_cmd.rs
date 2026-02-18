use crate::config::{Profile, Storage};
use crate::error::Result;
use chrono::Utc;
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

    // Create template JSON with only env fields
    let template = json!({
        "env": {
            "ANTHROPIC_AUTH_TOKEN": "sk-apikey",
            "ANTHROPIC_BASE_URL": "https://yourapi",
            "ANTHROPIC_MODEL": "",
            "ANTHROPIC_DEFAULT_HAIKU_MODEL": "",
            "ANTHROPIC_DEFAULT_SONNET_MODEL": "",
            "ANTHROPIC_DEFAULT_OPUS_MODEL": ""
        }
    });

    // Create settings.json in profile directory
    let settings_path = storage.ensure_profile_settings_dir(&name)?;
    let original_content = serde_json::to_string_pretty(&template)?;
    fs::write(&settings_path, &original_content)?;

    println!(
        "\n{} Opening editor to configure profile...",
        "→".cyan()
    );
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
        println!("{}", "No changes made. Profile creation cancelled.".yellow());
        return Ok(());
    }

    // Parse the edited content and merge with defaults
    let user_json: serde_json::Value = serde_json::from_str(&content).map_err(|e| {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        crate::error::Error::ConfigError(format!("Invalid JSON: {e}"))
    })?;

    // Create default values for missing fields
    let default_json = json!({
        "name": name,
        "display_name": null,
        "permissions": {
            "enabled": null,
            "mcp": null,
            "command": null
        },
        "enabled_plugins": null,
        "always_thinking_enabled": null,
        "api_timeout_ms": null,
        "category": null,
        "source": "manual",
        "created_at": Utc::now(),
        "updated_at": Utc::now()
    });

    // Merge: user values override defaults
    let merged_json = json!({
        "name": name,
        "display_name": user_json.get("display_name").or_else(|| default_json.get("display_name")),
        "env": user_json.get("env").unwrap_or(&json!({})),
        "permissions": user_json.get("permissions").unwrap_or_else(|| default_json.get("permissions").unwrap()),
        "enabled_plugins": user_json.get("enabled_plugins").or_else(|| default_json.get("enabled_plugins")),
        "always_thinking_enabled": user_json.get("always_thinking_enabled").or_else(|| default_json.get("always_thinking_enabled")),
        "api_timeout_ms": user_json.get("api_timeout_ms").or_else(|| default_json.get("api_timeout_ms")),
        "category": user_json.get("category").or_else(|| default_json.get("category")),
        "source": user_json.get("source").unwrap_or_else(|| default_json.get("source").unwrap()),
        "created_at": user_json.get("created_at").unwrap_or_else(|| default_json.get("created_at").unwrap()),
        "updated_at": Utc::now()
    });

    let profile: Profile = serde_json::from_value(merged_json).map_err(|e| {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        crate::error::Error::ConfigError(format!("Invalid JSON: {e}"))
    })?;

    // Validate that at least some env vars are set
    if profile.env.is_empty() {
        fs::remove_file(&settings_path).ok();
        fs::remove_dir_all(storage.profile_settings_dir(&name)).ok();
        return Err(crate::error::Error::ConfigError(
            "No environment variables configured".into(),
        ));
    }

    // Add profile name to ccuse.json
    let mut names = storage.load_profiles()?.iter().map(|p| p.name.clone()).collect::<Vec<_>>();
    names.push(name.clone());

    // Save just the names list
    let ccuse_path = storage.config_dir().join("ccuse.json");
    fs::write(ccuse_path, serde_json::to_string_pretty(&names)?)?;

    println!(
        "{}",
        format!("✓ Profile '{name}' created successfully!").green()
    );

    Ok(())
}
