use crate::config::Profile;
use crate::error::{Error, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Storage {
    config_dir: PathBuf,
}

impl Storage {
    /// Create a new Storage instance.
    ///
    /// # Errors
    ///
    /// Returns an error if config directory cannot be determined or created.
    pub fn new() -> Result<Self> {
        let project_dirs = ProjectDirs::from("com", "ccuse", "ccuse")
            .ok_or_else(|| Error::ConfigError("Failed to determine config directory".into()))?;

        let config_dir = project_dirs.config_dir().to_path_buf();

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)?;
        }

        Ok(Self { config_dir })
    }

    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get the settings directory for a specific profile
    /// Path: ~/.config/ccuse/<profile-name>/
    #[must_use]
    pub fn profile_settings_dir(&self, profile_name: &str) -> PathBuf {
        self.config_dir.join(profile_name)
    }

    /// Get the settings.json path for a specific profile
    /// Path: ~/.config/ccuse/<profile-name>/settings.json
    #[must_use]
    pub fn profile_settings_path(&self, profile_name: &str) -> PathBuf {
        self.profile_settings_dir(profile_name)
            .join("settings.json")
    }

    /// Ensure the profile settings directory exists and return the settings.json path
    ///
    /// # Errors
    ///
    /// Returns an error if the directory cannot be created.
    pub fn ensure_profile_settings_dir(&self, profile_name: &str) -> Result<PathBuf> {
        let dir = self.profile_settings_dir(profile_name);
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(self.profile_settings_path(profile_name))
    }

    /// Load a single profile from its settings.json
    ///
    /// # Errors
    ///
    /// Returns an error if settings.json cannot be read or deserialized.
    fn load_profile_from_file(&self, name: &str) -> Result<Profile> {
        let path = self.profile_settings_path(name);
        if !path.exists() {
            return Err(Error::ProfileNotFound(name.into()));
        }
        let content = fs::read_to_string(&path)?;
        let profile: Profile = serde_json::from_str(&content)?;
        Ok(profile)
    }

    /// Save a single profile to its settings.json
    ///
    /// # Errors
    ///
    /// Returns an error if profile cannot be serialized or written to file.
    fn save_profile_to_file(&self, profile: &Profile) -> Result<()> {
        let path = self.ensure_profile_settings_dir(&profile.name)?;
        let content = serde_json::to_string_pretty(profile)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Load all profiles from storage by scanning config directory.
    ///
    /// # Errors
    ///
    /// Returns an error if profiles cannot be loaded.
    pub fn load_profiles(&self) -> Result<Vec<Profile>> {
        let mut profiles = Vec::new();

        // Scan config directory for profile directories
        if !self.config_dir.exists() {
            return Ok(profiles);
        }

        for entry in fs::read_dir(&self.config_dir)? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_dir() {
                continue;
            }

            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip hidden directories
            if dir_name.starts_with('.') {
                continue;
            }

            // Try to load profile from settings.json
            let settings_path = path.join("settings.json");
            if settings_path.exists() {
                match self.load_profile_from_file(dir_name) {
                    Ok(profile) => profiles.push(profile),
                    Err(e) => {
                        eprintln!("Warning: Failed to load profile '{}': {}", dir_name, e);
                    }
                }
            }
        }

        Ok(profiles)
    }

    /// Save profiles to storage.
    ///
    /// # Errors
    ///
    /// Returns an error if profiles cannot be saved.
    pub fn save_profiles(&self, profiles: &[Profile]) -> Result<()> {
        // Save each profile to its own settings.json
        for profile in profiles {
            self.save_profile_to_file(profile)?;
        }

        Ok(())
    }

    /// Get a profile by name.
    ///
    /// # Errors
    ///
    /// Returns an error if profile cannot be loaded.
    pub fn get_profile(&self, name: &str) -> Result<Option<Profile>> {
        // Try to load profile directly from settings.json
        match self.load_profile_from_file(name) {
            Ok(profile) => Ok(Some(profile)),
            Err(Error::ProfileNotFound(_)) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Add a new profile.
    ///
    /// # Errors
    ///
    /// Returns an error if profile already exists or cannot be saved.
    pub fn add_profile(&self, profile: Profile) -> Result<()> {
        // Check if profile already exists by trying to load it
        if self.get_profile(&profile.name)?.is_some() {
            return Err(Error::ProfileAlreadyExists(profile.name));
        }

        // Save profile to its settings.json
        self.save_profile_to_file(&profile)?;

        Ok(())
    }

    /// Update an existing profile.
    ///
    /// # Errors
    ///
    /// Returns an error if profile does not exist or cannot be saved.
    pub fn update_profile(&self, profile: Profile) -> Result<()> {
        // Check if profile exists
        if self.get_profile(&profile.name)?.is_none() {
            return Err(Error::ProfileNotFound(profile.name));
        }

        // Update profile in its settings.json
        self.save_profile_to_file(&profile)?;

        Ok(())
    }

    /// Remove a profile by name.
    ///
    /// # Errors
    ///
    /// Returns an error if profile does not exist or cannot be removed.
    pub fn remove_profile(&self, name: &str) -> Result<()> {
        // Check if profile exists
        if self.get_profile(name)?.is_none() {
            return Err(Error::ProfileNotFound(name.into()));
        }

        // Remove profile directory
        let profile_dir = self.profile_settings_dir(name);
        if profile_dir.exists() {
            fs::remove_dir_all(&profile_dir)?;
        }

        Ok(())
    }

    /// Remove all profiles.
    ///
    /// # Errors
    ///
    /// Returns an error if profiles cannot be removed.
    pub fn remove_all_profiles(&self) -> Result<()> {
        // Load all profiles first
        let profiles = self.load_profiles()?;

        // Remove all profile directories
        for profile in profiles {
            let profile_dir = self.profile_settings_dir(&profile.name);
            if profile_dir.exists() {
                fs::remove_dir_all(&profile_dir)?;
            }
        }

        Ok(())
    }
}

impl Default for Storage {
    fn default() -> Self {
        // Safe default - uses system temp directory
        Self {
            config_dir: std::env::temp_dir().join("ccuse"),
        }
    }
}
