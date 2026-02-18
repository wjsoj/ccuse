use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ccuse")]
#[command(about = "Manage and switch Claude Code configurations", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short, long, global = true)]
    pub verbose: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Use a profile to launch Claude Code
    Use {
        /// Profile name to use
        name: String,

        /// Skip permissions check (equivalent to --dangerously-skip-permissions)
        #[arg(short = 'b', long = "bypass", global = false)]
        bypass: bool,

        /// Additional arguments to pass to Claude Code
        #[arg(last = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },

    /// Update profiles from CC-Switch database
    Update,

    /// List all available profiles
    List,

    /// Add a new profile interactively
    Add,

    /// Remove a profile
    Remove {
        /// Name of the profile to remove
        name: Option<String>,

        /// Remove all profiles and delete the data file
        #[arg(long = "all", short = 'a')]
        all: bool,
    },

    /// Rename a profile
    Rename {
        /// Current name of the profile
        old_name: String,

        /// New name for the profile
        new_name: String,
    },

    /// Show configuration directory
    ConfigDir,

    /// Install shell completions interactively
    Completions,
}
