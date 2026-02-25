use ccuse::cli::commands::{
    add_profile, list_profiles, remove_all_profiles, remove_profile, rename_profile, run_ccusage,
    update_profiles, use_profile,
};
use ccuse::cli::{Args, Commands, CompletionInstaller};
use ccuse::config::Storage;
use clap::Parser;
use colored::Colorize;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .init();

    // Parse arguments
    let args = Args::parse();

    // Set up verbose logging if requested
    if args.verbose {
        tracing::info!("Verbose mode enabled");
    }

    // Run the appropriate command
    let result = match args.command {
        Commands::Use { name, bypass, args } => use_profile(&name, bypass, &args),

        Commands::Update => update_profiles(),

        Commands::List => list_profiles(),

        Commands::Add => add_profile(),

        Commands::Remove { name, all } => {
            if all {
                remove_all_profiles()
            } else if let Some(n) = name {
                remove_profile(&n)
            } else {
                eprintln!("Error: specify a profile name or use --all to remove all profiles");
                std::process::exit(1);
            }
        }

        Commands::Rename { old_name, new_name } => rename_profile(&old_name, &new_name),

        Commands::ConfigDir => {
            let storage = Storage::default();
            println!("{}", storage.config_dir().display());
            Ok(())
        }

        Commands::Completions => {
            if let Err(e) = CompletionInstaller::run() {
                eprintln!("Error: {e}");
                std::process::exit(1);
            }
            Ok(())
        }

        Commands::Usage { args } => run_ccusage(&args),
    };

    if let Err(e) = result {
        eprintln!("{} {e}", "Error:".red().bold());
        std::process::exit(1);
    }
}
