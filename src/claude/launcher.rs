use crate::config::Profile;
use crate::config::Storage;
use crate::error::{Error, Result};
use std::collections::HashMap;
use std::env;
use std::process::{Command, Stdio};
use which::which;

pub struct Launcher;

impl Launcher {
    /// Find the Claude Code executable in the system.
    ///
    /// # Errors
    ///
    /// Returns an error if Claude Code executable cannot be found in PATH or `CLAUDE_CODE_PATH`.
    pub fn find_claude_executable() -> Result<String> {
        // Try common locations
        let candidates = vec!["claude", "claude-code", "Claude Code"];

        for candidate in &candidates {
            if which(candidate).is_ok() {
                return Ok((*candidate).to_string());
            }
        }

        // Try environment variable
        if let Ok(claude_path) = env::var("CLAUDE_CODE_PATH") {
            if std::path::Path::new(&claude_path).exists() {
                return Ok(claude_path);
            }
        }

        Err(Error::ClaudeNotFound)
    }

    /// Launch Claude Code with the specified profile.
    ///
    /// # Errors
    ///
    /// Returns an error if Claude Code cannot be found, settings cannot be found, or the process fails to launch.
    pub fn launch(profile: &Profile, bypass: bool, args: &[String]) -> Result<()> {
        let claude_cmd = Self::find_claude_executable()?;

        // Create storage to get profile settings path
        let storage = Storage::new()?;

        // Get profile-specific settings.json path (should already exist)
        let settings_path = storage.profile_settings_path(&profile.name);

        if !settings_path.exists() {
            return Err(Error::ConfigError(format!(
                "Settings file not found for profile '{}': {}",
                profile.name,
                settings_path.display()
            )));
        }

        // Build environment - inherit from parent, then override with profile env vars
        let mut env_vars: HashMap<String, String> = env::vars().collect();

        // Remove CLAUDECODE to allow launching Claude inside another Claude session
        env_vars.remove("CLAUDECODE");

        // Override with profile env vars (these contain the provider configuration)
        for (key, value) in &profile.env {
            env_vars.insert(key.clone(), value.clone());
        }

        // Build command arguments
        let mut claude_args = Vec::new();

        // Add --settings flag to use profile-specific settings
        claude_args.push("--settings".to_string());
        claude_args.push(settings_path.to_string_lossy().to_string());

        // Add bypass flag if requested
        if bypass {
            claude_args.push("--dangerously-skip-permissions".to_string());
        }

        // Add user-provided arguments
        claude_args.extend(args.iter().cloned());

        // Launch process
        let mut cmd = Command::new(&claude_cmd);
        cmd.args(&claude_args)
            .envs(&env_vars)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());

        let mut child = cmd.spawn().map_err(|e| Error::LaunchError(e.to_string()))?;

        // Wait for the child to complete so ccuse keeps the terminal alive
        child
            .wait()
            .map_err(|e| Error::LaunchError(e.to_string()))?;

        Ok(())
    }
}
