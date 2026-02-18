use colored::Colorize;
use inquire::Confirm;
use std::fs;
use std::path::{Path, PathBuf};

const ZSH_COMPLETION: &str = r#"#compdef ccuse

# zsh completion for ccuse
# Dynamic completion that fetches profile names from ccuse list

local -a subcommands
subcommands=(
  'use:Use a profile to launch Claude Code'
  'update:Update profiles from CC-Switch database'
  'list:List all available profiles'
  'add:Add a new profile interactively'
  'remove:Remove a profile'
  'rename:Rename a profile'
  'config-dir:Show configuration directory'
  'completions:Install shell completions'
)

# Get profile names dynamically from ccuse list
local -a profiles
profiles=(${${(f)"$(ccuse list 2>/dev/null | sed -n 's/^  \([^ ]*\).*/\1/p')"}:#})

case "$words[1]" in
  use|remove|rename)
    _describe 'profile' profiles
    ;;
  *)
    _describe 'command' subcommands
    ;;
esac
"#;

const BASH_COMPLETION: &str = r#"_ccuse() {
    local cur prev opts
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    local -a subcommands
    subcommands=(
        use
        update
        list
        add
        remove
        rename
        config-dir
        completions
    )

    local -a profiles
    profiles=($(ccuse list 2>/dev/null | sed -n 's/^  \([^ ]*\).*/\1/p' | grep -v '^$'))

    case "${prev}" in
        ccuse)
            COMPREPLY=($(compgen -W "${subcommands[*]}" -- "${cur}"))
            ;;
        use|remove|rename)
            COMPREPLY=($(compgen -W "${profiles[*]}" -- "${cur}"))
            ;;
    esac

    return 0
}

complete -F _ccuse ccuse
"#;

const FISH_COMPLETION: &str = r#"complete -c ccuse -f -n '__fish_use_subcommand' -a 'use' -d 'Use a profile to launch Claude Code'
complete -c ccuse -f -n '__fish_use_subcommand' -a 'update' -d 'Update profiles from CC-Switch database'
complete -c ccuse -f -n '__fish_use_subcommand' -a 'list' -d 'List all available profiles'
complete -c ccuse -f -n '__fish_use_subcommand' -a 'add' -d 'Add a new profile interactively'
complete -c ccuse -f -n '__fish_use_subcommand' -a 'remove' -d 'Remove a profile'
complete -c ccuse -f -n '__fish_use_subcommand' -a 'rename' -d 'Rename a profile'
complete -c ccuse -f -n '__fish_use_subcommand' -a 'config-dir' -d 'Show configuration directory'
complete -c ccuse -f -n '__fish_use_subcommand' -a 'completions' -d 'Install shell completions'

complete -c ccuse -f -n '__fish_seen_subcommand_from use remove rename' -a '(ccuse list 2>/dev/null | sed -n "s/^  \\([^ ]*\\).*/\\1/p")'
"#;

pub struct CompletionInstaller;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shell {
    Zsh,
    Bash,
    Fish,
}

impl Shell {
    #[must_use]
    pub fn detect() -> Option<Self> {
        let shell = std::env::var("SHELL").ok()?;
        if shell.contains("zsh") {
            Some(Self::Zsh)
        } else if shell.contains("bash") {
            Some(Self::Bash)
        } else if shell.contains("fish") {
            Some(Self::Fish)
        } else {
            None
        }
    }

    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::Zsh => "zsh",
            Self::Bash => "bash",
            Self::Fish => "fish",
        }
    }

    #[must_use]
    pub fn completion(&self) -> &'static str {
        match self {
            Self::Zsh => ZSH_COMPLETION,
            Self::Bash => BASH_COMPLETION,
            Self::Fish => FISH_COMPLETION,
        }
    }

    #[must_use]
    pub fn config_path(&self, home: &Path) -> (PathBuf, &'static str) {
        match self {
            Self::Zsh => (home.join(".zsh/completions/_ccuse"), "~/.zsh/completions/"),
            Self::Bash => (
                home.join(".local/share/bash-completion/completions/ccuse"),
                "~/.local/share/bash-completion/completions/",
            ),
            Self::Fish => (
                home.join(".config/fish/completions/ccuse.fish"),
                "~/.config/fish/completions/",
            ),
        }
    }

    #[must_use]
    pub fn init_line(&self, path: &Path) -> String {
        match self {
            Self::Zsh => format!(
                "# Add to ~/.zshrc or create ~/.zsh/completions:\n# mkdir -p ~/.zsh/completions\n# cp {} ~/.zsh/completions/",
                path.display()
            ),
            Self::Bash => format!(
                "# Add to ~/.bashrc:\nsource {}",
                path.display()
            ),
            Self::Fish => "# Fish completions are auto-loaded from ~/.config/fish/completions/".to_string(),
        }
    }
}

impl CompletionInstaller {
    /// Run the completion installer.
    ///
    /// # Errors
    ///
    /// Returns an error if shell cannot be detected, user confirmation fails, or completion file cannot be written.
    pub fn run() -> std::result::Result<(), Box<dyn std::error::Error>> {
        println!("\n{}", "Shell Completions Installation".bold().green());
        println!("{}", "═".repeat(50));

        // Detect current shell
        let shell = Shell::detect().ok_or("Unable to detect shell type")?;
        println!("\nDetected shell: {}", shell.name().bold());

        // Show available options
        println!("\nSupported shells:");
        for (i, s) in [Shell::Zsh, Shell::Bash, Shell::Fish].iter().enumerate() {
            let marker = if *s == shell { " ✓" } else { "" };
            println!("  {}. {}{}", i + 1, s.name(), marker);
        }

        // Get home directory
        let home = dirs::home_dir().ok_or("Cannot find home directory")?;
        let (target_path, rel_dir) = shell.config_path(&home);

        // Show what will be done
        println!("\n{}", "Installation plan:".bold());
        println!(
            "  - Target file: {}",
            target_path.display().to_string().cyan()
        );
        println!("  - Directory: {}", rel_dir.cyan());

        // Check if file already exists
        let action = if target_path.exists() {
            "update (overwrite)"
        } else {
            "create"
        };
        println!("  - Action: {}", action.yellow());

        // Show preview of completion file
        println!("\n{}", "File preview (first 20 lines):".bold());
        let preview: Vec<&str> = shell.completion().lines().take(20).collect();
        for (i, line) in preview.iter().enumerate() {
            println!("{:3}: {}", i + 1, line);
        }
        if shell.completion().lines().count() > 20 {
            println!(
                "    ... ({} more lines)",
                shell.completion().lines().count() - 20
            );
        }

        // Require confirmation
        println!("\n");
        let confirmed = Confirm::new("Do you want to proceed with the installation?")
            .with_default(true)
            .prompt()?;

        if !confirmed {
            println!("\n{}", "Installation cancelled.".yellow());
            return Ok(());
        }

        // Create parent directory if needed
        if let Some(parent) = target_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
                println!(
                    "\nCreated directory: {}",
                    parent.display().to_string().cyan()
                );
            }
        }

        // Write completion file
        fs::write(&target_path, shell.completion())?;
        println!("\n{} Installed completions to:", "✓".green());
        println!("  {}", target_path.display().to_string().cyan());

        // Show init instructions
        println!("\n{}", "Next steps:".bold());
        match shell {
            Shell::Zsh => {
                println!("  1. Add to ~/.zshrc:");
                println!("     mkdir -p ~/.zsh/completions");
                println!("  2. Restart shell or run: source ~/.zshrc");
            }
            Shell::Bash => {
                println!("  Add to ~/.bashrc:");
                println!("    source {}", target_path.display());
            }
            Shell::Fish => {
                println!("  Fish completions are auto-loaded.");
                println!("  Restart your terminal.");
            }
        }

        println!();
        Ok(())
    }
}
