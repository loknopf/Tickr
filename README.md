# Tickr

Tickr is a terminal-based time tracker with a TUI and a small CLI for quick data entry.

## Features

- TUI mode for browsing and tracking time
- Detail view edit popup (label + category)
- Categories tab with in-app category creation
- CLI commands to add projects, tasks, and categories
- SQLite storage with automatic migrations

## Install

### From GitHub Releases

Quick install (Linux/macOS):

```bash
curl -fsSL https://raw.githubusercontent.com/loknopf/Tickr/main/scripts/install.sh | bash
```

Quick install (Windows PowerShell):

```powershell
irm https://raw.githubusercontent.com/loknopf/Tickr/main/scripts/install.ps1 | iex
```

Both installers add the install directory to your PATH by default. Use `--no-add-to-path`
for the shell script or `-AddToPath:$false` for PowerShell to opt out.

Manual install:

1. Download the latest archive for your OS/CPU from the Releases page.
2. Extract the archive and place `tickr` (or `tickr.exe` on Windows) somewhere in your `PATH`.
3. Verify with:

```bash
tickr --help
```

### Build from source

```bash
cargo install --path .
```

## Release (maintainers)

Releases are built by GitHub Actions when you push a version tag:

```bash
git tag v0.1.0
git push origin v0.1.0
```

The workflow builds Linux, macOS, and Windows binaries and uploads them to the GitHub release.

## Usage

Run the TUI:

```bash
cargo run
```

### TUI Controls

- `p` Projects tab
- `t` Tickrs tab
- `w` Worked tab
- `c` Categories tab
- `r` Refresh current view
- `q` Quit

Projects/Worked/Categories lists:

- `Up`/`Down` Move selection
- `Enter` Open selection (Projects/Worked)
- `Esc` Back

Tickrs list:

- `Up`/`Down` Move selection
- `Enter` Open detail
- `Space` Start/End selected task

Detail view:

- `Space` Start/End task
- `s` Stop running task
- `g` Jump to project
- `e` Edit label/category

Edit popup:

- Type to edit label
- `Up`/`Down` Select category
- `Enter` Save
- `Esc` Cancel

Categories tab:

- `n` New category

New category popup:

- Type name/color (hex like `#RRGGBB` or `RRGGBB`)
- `Tab` Switch field
- `Enter` Save
- `Esc` Cancel

Add a project:

```bash
cargo run -- project add "My Project"
```

Add a task entry:

```bash
cargo run -- task add "My Project" "Write docs" --start "2026-02-14T09:00:00+01:00" --end "2026-02-14T10:00:00+01:00" --category "Writing"
```

Add a category (optionally with hex color):

```bash
cargo run -- category "Writing" "#FFAA00"
```

## Data

The database is stored in the user's local data directory under `tickr/tickr.db` and falls back to `./tickr.db` if no data directory is found.
