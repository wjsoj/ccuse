use crate::config::Storage;
use crate::error::Result;
use colored::Colorize;

/// List all available profiles.
///
/// # Errors
///
/// Returns an error if profiles cannot be loaded from storage.
pub fn list_profiles() -> Result<()> {
    let storage = Storage::new()?;
    let profiles = storage.load_profiles()?;

    if profiles.is_empty() {
        println!("{}", "No profiles found. Run 'ccuse update' to sync from CC-Switch or 'ccuse add' to create one.".yellow());
        return Ok(());
    }

    println!("{}", "Available profiles:".bold());
    println!();

    for profile in &profiles {
        let name = profile.display_name.as_ref().unwrap_or(&profile.name);

        // Only show source if explicitly set
        let source_str = match &profile.source {
            Some(crate::config::ProfileSource::CcSwitch) => Some("ccswitch".cyan()),
            Some(crate::config::ProfileSource::Manual) => Some("manual".blue()),
            None => None,
        };

        match source_str {
            Some(colored) => println!("  {} ({})", name.green(), colored),
            None => println!("  {}", name.green()),
        }

        if !profile.env.is_empty() {
            let env_count = profile.env.len();
            println!("    Environment variables: {env_count}");
        }

        if let Some(timeout) = profile.api_timeout_ms {
            println!("    API timeout: {timeout}ms");
        }

        println!();
    }

    Ok(())
}
