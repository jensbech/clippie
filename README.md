# Clippy

A fast, keyboard-driven clipboard history manager for macOS. Browse, search, and restore clipboard history with an interactive terminal UI.

## Features

- **Lightweight TUI** - Blazing fast keyboard-driven interface powered by React + Ink
- **Persistent History** - SQLite-backed clipboard history with full-text search
- **Background Daemon** - Automatic clipboard monitoring with LaunchAgent integration
- **Portable Config** - No hard-coded paths, fully configurable via setup wizard
- **Easy Management** - Simple commands for daemon control and history cleanup
- **Terminal Integration** - Works seamlessly with shell functions for quick access

## Requirements

- macOS 10.15+ (uses `pbcopy`/`pbpaste`)
- Node.js 18.0.0 or later
- sqlite3 (usually pre-installed on macOS)

## Installation

```bash
# Clone the repository
git clone https://git.bechsor.no/jens/clippy.git
cd clippy

# Install dependencies
pnpm install

# Build
pnpm build

# Link globally (optional, for npm bin commands)
npm link
```

## Quick Start

```bash
# 1. Run the setup wizard
clippy setup

# 2. Install the daemon
clippy-install

# 3. Start monitoring clipboard
clippy-start

# 4. Launch the history browser
clippy
```

## Usage

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Move down |
| `k` / `↑` | Move up |
| `Enter` | Copy selected entry to clipboard and exit |
| `/` | Filter/search entries |
| `r` | Refresh from database |
| `q` / `Esc` | Quit |

### Commands

#### Browser

```bash
clippy              # Launch the history browser
clippy setup        # Configure database location
```

#### Daemon Management

```bash
clippy-install      # Install LaunchAgent daemon
clippy-start        # Start the daemon
clippy-stop         # Stop the daemon
clippy-status       # Show daemon status and database info
```

#### Utilities

```bash
clippy-clear        # Interactive delete entries from history (uses fzf)
clippy-clear --all  # Delete entire clipboard history
```

## Configuration

### Configuration File

Clippy looks for config at `~/.config/clippy/config.json`:

```json
{
  "dbPath": "/path/to/clipboard-history.db",
  "version": "1.0.0"
}
```

### Configuration Priority (highest to lowest)

1. `CLIPPY_DB_PATH` environment variable
2. `~/.config/clippy/config.json` config file
3. Default: `~/.local/share/clippy/clipboard-history.db`

### Examples

Use custom database location:
```bash
CLIPPY_DB_PATH=/custom/path/to/db.db clippy
```

Daemon with custom path:
```bash
CLIPPY_DB_PATH=/custom/path/to/db.db clippy-daemon
```

## Terminal Integration

Add to `~/.zshrc` or `~/.bashrc` for quick access:

```bash
cb() {
  local result
  result=$(clippy)
  if [[ -n "$result" ]]; then
    print -z "$result"  # zsh
    # For bash: eval "fc -s -- '$result'"
  fi
}
```

Then use `cb` to open the clipboard history browser.

## Database Schema

Clippy uses SQLite with the following schema:

```sql
CREATE TABLE clipboard_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    content TEXT NOT NULL,
    content_hash TEXT NOT NULL UNIQUE,
    first_copied INTEGER NOT NULL,
    last_copied INTEGER NOT NULL,
    copy_count INTEGER DEFAULT 1
);
CREATE INDEX idx_content_hash ON clipboard_history(content_hash);
CREATE INDEX idx_last_copied ON clipboard_history(last_copied DESC);
```

## Daemon Logs

The LaunchAgent writes logs to:
- Standard output: `~/Library/Logs/clippy-daemon.out.log`
- Standard error: `~/Library/Logs/clippy-daemon.err.log`

Check daemon status:
```bash
clippy-status
tail -f ~/Library/Logs/clippy-daemon.err.log
```

## Development

### Build

```bash
pnpm build
```

### Run Development

```bash
pnpm dev
```

### Project Structure

```
clippy/
├── src/
│   ├── cli.jsx           # Entry point with setup command
│   ├── App.jsx           # Main TUI component
│   ├── lib/
│   │   ├── config.js     # Configuration management
│   │   ├── db.js         # Database access
│   │   └── dates.js      # Date formatting
│   ├── components/       # React/Ink UI components
│   └── commands/
│       └── setup.js      # Interactive setup wizard
├── bin/
│   ├── clippy-daemon     # Clipboard monitoring daemon
│   ├── clippy-install    # Install LaunchAgent
│   ├── clippy-start      # Start daemon
│   ├── clippy-stop       # Stop daemon
│   ├── clippy-status     # Show daemon status
│   └── clippy-clear      # Delete history entries
└── package.json
```

## Troubleshooting

### Daemon Not Running

```bash
# Check status
clippy-status

# View logs
tail -f ~/Library/Logs/clippy-daemon.err.log

# Restart
clippy-stop
clippy-start
```

### No History Appearing

1. Ensure daemon is running: `clippy-status`
2. Copy something to clipboard: `echo "test" | pbcopy`
3. Wait 3-5 seconds for daemon to process
4. Refresh in browser: press `r`

### Database Issues

```bash
# Check database exists at configured path
cat ~/.config/clippy/config.json

# Verify database is accessible
sqlite3 ~/.local/share/clippy/clipboard-history.db "SELECT COUNT(*) FROM clipboard_history;"
```

## License

MIT - See LICENSE file for details

## Author

Jens Bech-Sørensen
