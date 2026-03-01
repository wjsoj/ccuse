use crate::config::{Profile, ProfileSource, Storage};
use crate::db::CcSwitchDb;
use crate::error::Result;
use colored::Colorize;

/// Update profiles from CC-Switch database.
///
/// # Errors
///
/// Returns an error if CC-Switch database cannot be accessed or profiles cannot be saved.
pub fn update_profiles() -> Result<()> {
    let storage = Storage::new()?;

    if !CcSwitchDb::exists() {
        println!(
            "{}",
            "CC-Switch database not found. No profiles to update.".yellow()
        );
        return Ok(());
    }

    let ccswitch = CcSwitchDb::new()?;
    let new_profiles = ccswitch.get_profiles()?;

    if new_profiles.is_empty() {
        println!("{}", "No profiles found in CC-Switch database.".yellow());
        return Ok(());
    }

    // Load existing profiles
    let existing_profiles = storage.load_profiles()?;

    // Separate CC-Switch profiles and manual profiles
    let manual_profiles: Vec<Profile> = existing_profiles
        .iter()
        .filter(|p| p.source.as_ref() == Some(&ProfileSource::Manual))
        .cloned()
        .collect();

    // Merge: keep manual profiles, replace/update CC-Switch profiles
    let mut updated_profiles = manual_profiles;

    for new_profile in new_profiles {
        // Replace spaces with underscores in the name for easier input
        let name_with_underscores = new_profile.name.replace(' ', "_");

        // Check if profile from same source exists, update or add
        if let Some(idx) = updated_profiles
            .iter()
            .position(|p| p.name == new_profile.name || p.name == name_with_underscores)
        {
            // Update existing - also replace spaces in name
            let mut updated_profile = new_profile;
            updated_profile.name = name_with_underscores.clone();
            updated_profile.display_name = Some(name_with_underscores);
            updated_profiles[idx] = updated_profile;
        } else {
            // Add new with underscores instead of spaces
            let mut updated_profile = new_profile;
            updated_profile.name = name_with_underscores.clone();
            updated_profile.display_name = Some(name_with_underscores);
            updated_profiles.push(updated_profile);
        }
    }

    storage.save_profiles(&updated_profiles)?;

    println!(
        "{}",
        format!(
            "Updated {} profiles from CC-Switch.",
            updated_profiles.len()
        )
        .green()
    );

    Ok(())
}
