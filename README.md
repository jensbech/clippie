# Clippy

A fast, keyboard-driven clipboard history manager for macOS.

## Requirements

- macOS 10.15+
- Node.js 18.0.0+
- sqlite3 (pre-installed on macOS)

## Installation

```bash
git clone <repo-url>
cd clippy
pnpm install
pnpm build
npm link  # optional: for global CLI access
```

## Setup (One Command)

```bash
clippy setup
```

This will:
- Ask for database location (or use default)
- Install the daemon
- Start monitoring clipboard
- Show completion status

Done! Everything is configured and running.

## Commands

```bash
clippy              # Browse clipboard history
clippy setup        # Configure database location
clippy start        # Start the daemon
clippy stop         # Stop the daemon
clippy status       # Show daemon status
clippy db <path>    # Swap to a database (or create new)
clippy clear        # Delete history entries
clippy clear --all  # Delete all history
```

### Swap Databases

Switch to an existing database or create a new one:

```bash
# Use or create database
clippy db ~/.local/share/clippy/backup.db
clippy db /tmp/test-clipboard.db
```

The daemon automatically restarts to use the new database.

### Help

```bash
clippy -h       # Show all commands
clippy --help   # Show help
```

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j`/`↓` | Move down |
| `k`/`↑` | Move up |
| `Enter` | Copy selected to clipboard |
| `/` | Search |
| `r` | Refresh |
| `q`/`Esc` | Quit |

## Configuration

Config location: `~/.config/clippy/config.json`

Priority (highest to lowest):
1. `CLIPPY_DB_PATH` env variable
2. Config file
3. Default: `~/.local/share/clippy/clipboard-history.db`

## Shell Integration

Add to `~/.zshrc`:

```bash
cb() {
  result=$(clippy)
  [[ -n "$result" ]] && print -z "$result"
}
```

Then use `cb` to quickly access history.

## Troubleshooting

Check daemon status and logs:
```bash
clippy-status
tail -f ~/Library/Logs/clippy-daemon.err.log
```

Restart daemon:
```bash
clippy-stop
clippy-start
```

## Development

```bash
pnpm build    # Build CLI
pnpm dev      # Run in dev mode
```

Project structure:
- `src/` - React/Ink TUI and CLI logic
- `bin/` - Daemon and management scripts
