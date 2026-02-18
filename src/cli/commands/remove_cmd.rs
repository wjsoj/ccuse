use crate::config::Storage;
use crate::error::Result;
use colored::Colorize;
use inquire::Confirm;

/// Remove a profile by name.
///
/// # Errors
///
/// Returns an error if profile does not exist, user confirmation fails, or profile cannot be removed.
pub fn remove_profile(name: &str) -> Result<()> {
    let storage = Storage::new()?;

    // Check if profile exists
    if storage.get_profile(name)?.is_none() {
        return Err(crate::error::Error::ProfileNotFound(name.into()));
    }

    // Confirm deletion
    let confirm = Confirm::new(&format!(
        "Are you sure you want to delete profile '{name}'?"
    ))
    .with_default(false)
    .prompt()?;

    if !confirm {
        println!("{}", "Deletion cancelled.".yellow());
        return Ok(());
    }

    storage.remove_profile(name)?;

    println!(
        "{}",
        format!("Profile '{name}' removed successfully.").green()
    );

    Ok(())
}

/// Remove all profiles.
///
/// # Errors
///
/// Returns an error if user confirmation fails or profiles cannot be removed.
pub fn remove_all_profiles() -> Result<()> {
    let storage = Storage::new()?;

    // Confirm deletion
    let confirm =
        Confirm::new("Are you sure you want to remove ALL profiles and delete the data file?")
            .with_default(false)
            .prompt()?;

    if !confirm {
        println!("{}", "Deletion cancelled.".yellow());
        return Ok(());
    }

    storage.remove_all_profiles()?;

    println!("{}", "All profiles removed and data file deleted.".green());

    Ok(())
}
