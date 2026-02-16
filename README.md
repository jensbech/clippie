<p align="center">
  <img src="logo.svg" alt="Clippie" width="120" />
</p>

<h1 align="center">Clippie</h1>

<p align="center">Fast keyboard-driven clipboard history manager for macOS.</p>

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/jensbech/clippie/main/install | bash

## Setup

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
