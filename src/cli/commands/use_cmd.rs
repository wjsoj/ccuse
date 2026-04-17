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

    // Process args to extract -b flag and convert to --dangerously-skip-permissions
    let mut actual_bypass = bypass;
    let mut filtered_args = Vec::new();

    for arg in args {
        if arg == "-b" {
            actual_bypass = true;
            // Add the proper Claude Code flag
            filtered_args.push("--dangerously-skip-permissions".to_string());
        } else {
            filtered_args.push(arg.clone());
        }
    }

    println!("Using profile: {}", profile.name);
    Launcher::launch(&profile, actual_bypass, &filtered_args)?;

    Ok(())
}

/// Use a profile to launch Happy.
///
/// # Errors
///
/// Returns an error if profile does not exist or Happy fails to launch.
pub fn usehappy_profile(name: &str, bypass: bool, args: &[String]) -> Result<()> {
    let storage = Storage::new()?;

    let profile = storage
        .get_profile(name)?
        .ok_or_else(|| crate::error::Error::ProfileNotFound(name.into()))?;

    // Process args to extract -b flag and convert to --dangerously-skip-permissions
    let mut actual_bypass = bypass;
    let mut filtered_args = Vec::new();

    for arg in args {
        if arg == "-b" {
            actual_bypass = true;
            // Add the proper Happy flag
            filtered_args.push("--dangerously-skip-permissions".to_string());
        } else {
            filtered_args.push(arg.clone());
        }
    }

    println!("Using profile (Happy): {}", profile.name);
    Launcher::launch_happy(&profile, actual_bypass, &filtered_args)?;

    Ok(())
}
