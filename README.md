<p align="center">
  <img src="logo.svg" alt="Clippie" width="120" />
</p>

<h1 align="center">Clippie</h1>

<p align="center">Fast keyboard-driven clipboard history manager for macOS.</p>

## Quick Start

```bash
curl -fsSL https://raw.githubusercontent.com/jensbech/clippie/main/install | bash
clippie setup
```

## Commands

```bash
clippie              # Browse clipboard history (fuzzy search with /)
clippie setup        # Install and configure daemon
clippie start/stop   # Start/stop the daemon
clippie status       # Show daemon status
clippie clear        # Delete old entries
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j`/`k` or `↓`/`↑` | Navigate |
| `Enter` | Copy and exit |
| `/` | Fuzzy search |
| `r` | Refresh |
| `q`/`Esc` | Quit |

## Features

- ⚡ Fast startup and search
- 🔍 Fuzzy search with multi-position highlighting
- 🔄 Auto-refresh every 5 seconds
- 📦 System-wide clipboard detection
- 💾 SQLite database
- 🎨 Clean TUI with keyboard navigation

## Locations

- Database: `~/.clippie/clipboard.db`
- Logs: `~/.clippie/daemon.log` and `daemon.err`

## Requirements

- macOS 10.15+
- sqlite3 (pre-installed)

## Development

```bash
cargo build --release
just build-local    # faster, current arch only
just release        # optimized release build
```
