use crate::claude::Launcher;
use crate::config::Storage;
use crate::error::Result;

/// Use a profile to launch Claude Code.
///
/// # Errors
///
/// Returns an error if profile does not exist or Claude Code fails to launch.
pub fn use_profile(name: &str, bypass: bool, args: &[String]) -> Result<()> {
    let storage = Storage::new()?;

    let profile = storage
        .get_profile(name)?
        .ok_or_else(|| crate::error::Error::ProfileNotFound(name.into()))?;

    println!(
        "Using profile: {}",
        profile.display_name.as_ref().unwrap_or(&profile.name)
    );
    Launcher::launch(&profile, bypass, args)?;

    Ok(())
}
