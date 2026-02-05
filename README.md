# Clippie

A fast, keyboard-driven clipboard history manager for macOS.

## Installation

### Build Release Binaries

To build binaries for both Intel and Apple Silicon Macs:

```bash
just build    # Build both x86_64 and aarch64
just release  # Build and create release/ directory with properly named binaries
```

This requires the Rust build targets to be installed (done automatically).

### Install from Binary

Once a release is available with binaries:

```bash
# Download and install latest binary
curl -fsSL https://git.bechsor.no/jens/clippie/releases/download/v1.0.0/clippie-1.0.0-aarch64-apple-darwin -o ~/.local/bin/clippie
chmod +x ~/.local/bin/clippie

# Run setup
clippie setup
```

Or clone and build locally:

```bash
git clone https://git.bechsor.no/jens/clippie.git
cd clippie
just build-local
cp target/release/clippie ~/.local/bin/
clippie setup
```

## Requirements

- macOS 10.15+
- Rust (will be installed automatically if not present)
- sqlite3 (pre-installed on macOS)

## Setup

After installation, configure the database location and install the daemon:

```bash
clippie setup
```

This will:
- Ask for database location (default: `~/.clippie/clipboard.db`)
- Optionally install the clipboard monitoring daemon
- Show completion status

## Commands

```bash
clippie              # Browse clipboard history
clippie setup        # Configure database location
clippie start        # Start the daemon
clippie stop         # Stop the daemon
clippie status       # Show daemon status
clippie db <path>    # Switch to a different database
clippie clear        # Delete old history entries
clippie clear --all  # Delete all history
clippie install      # Install the launchd daemon
```

## Switching Databases

Switch to an existing database or create a new one:

```bash
clippie db ~/.clippie/work.db
clippie db /tmp/test-clipboard.db
```

The daemon automatically restarts to use the new database.

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j`/`↓` | Move down |
| `k`/`↑` | Move up |
| `Enter` | Copy selected to clipboard and exit |
| `/` | Search/filter clipboard history |
| `Enter` | Confirm search |
| `Esc` | Cancel search |
| `q`/`Esc` | Quit |

## Configuration

Config location: `~/.config/clippy/config.json`

Database location: `~/.clippie/clipboard.db` (default)

Priority (highest to lowest):
1. `CLIPPIE_DB_PATH` environment variable
2. Config file (`~/.config/clippy/config.json`)
3. Default: `~/.clippie/clipboard.db`

## Development Commands

Use `just` to run common tasks:

```bash
just build       # Build binaries for both x86_64 and aarch64
just build-local # Build for current architecture only (faster)
just release     # Build and create release/ directory
just test        # Run tests
just lint        # Format code and run clippy
just clean       # Clean build artifacts
```

## Shell Integration

Add to `~/.zshrc` or `~/.bash_profile`:

```bash
cb() {
  result=$(clippie)
  [[ -n "$result" ]] && print -z "$result"
}
```

Then use `cb` to quickly access clipboard history and search.

## Troubleshooting

### Check daemon status:
```bash
clippie status
```

### View daemon logs:
```bash
tail -f ~/.clippie/daemon.err
tail -f ~/.clippie/daemon.log
```

### Restart daemon:
```bash
clippie stop
clippie start
```

### Reset to default database:
```bash
rm ~/.config/clippy/config.json
clippie setup
```

## Development

```bash
# Build release binary
cargo build --release

# Run tests
cargo test

# The binary is at target/release/clippie
./target/release/clippie --help
```

## Project Structure

- `src/main.rs` - CLI entry point
- `src/cli.rs` - Command definitions
- `src/daemon.rs` - Clipboard monitoring daemon
- `src/db.rs` - SQLite database management
- `src/tui/` - Terminal UI components
- `src/commands/` - Command implementations
- `install.sh` - Installation script

## License

MIT
