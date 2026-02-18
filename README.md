# CCUse - Claude Code Profile Manager

A CLI tool to manage and switch between different Claude Code configurations (profiles).

## Features

- **Profile Management**: Create, list, rename, and remove Claude Code profiles
- **Quick Switch**: Switch between profiles with a single command
- **Interactive Creation**: Add new profiles through an interactive prompt
- **CC-Switch Integration**: Import profiles from CC-Switch database
- **Persistent Storage**: Profiles are stored in a local JSON file
- **Shell Completions**: Support for Bash, Zsh, Fish shell completions

## Installation

### From AUR (Arch Linux)

```bash
# Pre-built binary (recommended - faster, no build dependencies)
yay -S ccuse-bin
# or
paru -S ccuse-bin

# Build from source
yay -S ccuse
# or
paru -S ccuse
```

### From Source

```bash
git clone https://github.com/wjsoj/ccuse.git
cd ccuse
cargo install --path .
```

### Install Shell Completions

```bash
# Auto-install to system directories
ccuse install-completions

# Or generate manually
ccuse completions bash > ~/.local/share/bash-completion/completions/ccuse
ccuse completions zsh > ~/.zsh/completions/_ccuse
ccuse completions fish > ~/.config/fish/completions/ccuse.fish
```

## Quick Start

### List available profiles

```bash
ccuse list
```

### Use a profile to launch Claude Code

```bash
ccuse use my-profile
```

### Add a new profile

```bash
ccuse add
```

### Update profiles from CC-Switch

```bash
ccuse update
```

### Remove a profile

```bash
ccuse remove my-profile
```

## Commands

| Command | Description |
|---------|-------------|
| `ccuse use <name>` | Launch Claude Code with the specified profile |
| `ccuse list` | List all available profiles |
| `ccuse add` | Add a new profile interactively |
| `ccuse update` | Update profiles from CC-Switch database |
| `ccuse remove <name>` | Remove the specified profile |
| `ccuse rename <old> <new>` | Rename a profile |
| `ccuse config-dir` | Show the configuration directory path |
| `ccuse completions <shell>` | Generate shell completion script |
| `ccuse install-completions` | Install shell completions to system directories |

### use

Launch Claude Code with a specific profile.

```bash
ccuse use <profile-name> [options] [-- <args>...]
```

**Options:**
- `-b, --bypass` - Skip permissions check
- `<args>...` - Additional arguments to pass to Claude Code

**Examples:**

```bash
# Use default profile
ccuse use work

# Bypass permissions check
ccuse use work --bypass

# Pass additional arguments to Claude Code
ccuse use work -- --verbose
```

### list

List all available profiles.

```bash
ccuse list
```

Shows all profiles with their names and whether they are the default.

### add

Add a new profile interactively. You will be prompted for:
- Profile name
- Claude Code executable path
- Additional arguments (optional)

```bash
ccuse add
```

### update

Import profiles from the CC-Switch database (if available).

```bash
ccuse update
```

### remove

Remove an existing profile or all profiles.

```bash
# Remove a single profile
ccuse remove <profile-name>

# Remove all profiles and delete the data file
ccuse remove --all
```

### rename

Rename an existing profile.

```bash
ccuse rename <old-name> <new-name>
```

**Examples:**

```bash
# Rename a profile
ccuse rename work work-personal
```

### config-dir

Show the configuration directory path.

```bash
ccuse config-dir
```

### completions

Generate shell completion script.

```bash
ccuse completions <shell>
```

Supported shells: `bash`, `zsh`, `fish`, `powershell`, `elvish`

### install-completions

Automatically install shell completions to system directories.

```bash
ccuse install-completions
```

## Configuration

Configuration is stored in:
- Linux/macOS: `~/.config/ccuse/`
- Windows: `%APPDATA%\ccuse\`

### Profile Structure

Profiles are stored in `~/.config/ccuse/ccuse.json`. Each profile contains:

```json
{
  "name": "work",
  "env": {
    "ANTHROPIC_API_KEY": "sk-ant-xxx"
  },
  "permissions": {
    "enabled": true,
    "mcp": ["allowed-server"],
    "command": ["git", "npm"]
  },
  "enabled_plugins": ["plugin-name"],
  "always_thinking_enabled": true,
  "api_timeout_ms": 30000
}
```

Each profile generates a corresponding Claude Code settings file at `~/.config/ccuse/<profile-name>/settings.json`.

## Development

### Build

```bash
# Debug build
cargo build

# Release build
cargo build --release
```

### Test

```bash
cargo test
```

### Lint

```bash
cargo clippy
```

### Format

```bash
cargo fmt
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License

## Related Projects

- [Claude Code](https://claude.ai/code) - Official Anthropic CLI tool
- [CC-Switch](https://github.com/farion1231/cc-switch) - Predecessor project
