# Twigdrop

Twigdrop is a fast, interactive Terminal User Interface (TUI) tool written in Rust to help you safely manage, inspect, and clean up your local git branches. It provides a visual representation of your branch statuses relative to their upstreams, preventing accidental deletion of unmerged work.

## Features

- **Visual Branch Status**: Quickly see if a branch is merged, ahead of upstream, has unique commits, or if its upstream is gone.
- **Batch Deletion**: Mark multiple branches and delete them all at once — with SHA backups (`~/.twigdrop/backups/`) and `[u] undo` toast.
- **Manage Menu**: Checkout, diff/AI analysis, safe delete, rename, push, set upstream, merge into current, copy name, stash.
- **Detailed Diff & Info**: Inspector drawer, commit graph, stash browser and side-by-side diff with syntax highlighting.
- **Dark + Light Themes**: `theme_mode` in config or Settings (`dark` default).
- **Accessibility & Safety**: Status is never color-only (`▲ ⨯ ↑↓ ✓ [P]` + labels); protected branches can be extended in config.

## 🚀 Installation

## 🚀 Installation

Ensure you have [Rust and Cargo installed](https://rustup.rs/). Then, clone the repository and build:

```bash
git clone <repository-url>
cd twigdrop
cargo build --release
```

You can run it directly:

```bash
cargo run
```

Or install it globally:

```bash
cargo install --path .
```

## Usage

Navigate to any git repository and run:

```bash
twigdrop
```

You can also pass a path to a specific repository:

```bash
twigdrop /path/to/your/repo
```

Options: `--help` / `--version`.

### Keybindings

Source of truth: `docs/KEYBINDINGS.md` (generated from `src/input/keybind.rs`).
Press `?` in-app for the full reference.

| Key              | Action                                              |
| ---------------- | --------------------------------------------------- |
| `↑`/`k`,`↓`/`j`  | Move selection                                      |
| `Space`          | Toggle mark for batch deletion                      |
| `Enter`          | Manage branch (spotlight: checkout/delete/rename/push/…) |
| type `[a-z0-9]`  | Instant filter · `Esc` clears                       |
| `u`              | Undo last delete (while the toast is live)          |
| `Ctrl+D`         | Safe delete (with confirmation)                     |
| `Ctrl+C`         | New branch · `Ctrl+P` prune · `Ctrl+S` cycle sort   |
| `Ctrl+I`         | AI analysis · `Ctrl+F` filters · `Ctrl+K` palette   |
| `Shift+S/C/R`    | Stashes / Commits / Rebase · `Alt+D` toggle view    |
| `Shift+Tab`      | App switcher · `Ctrl+H`/`?` help · `Ctrl+Q` quit    |
| `Left Click`     | Select / double-click to manage                     |

### Legend

- `▲` (Red): **Has Unique Commits** - This branch has commits not present on the remote. Deleting it means losing work!
- `⨯` (Gray): **Gone** - The remote tracking branch has been deleted.
- `↑` (Yellow): **Ahead of upstream** - Local branch has commits that haven't been pushed yet.
- `✓` (Blue): **Merged** - Contains no unmerged commits. Safe to delete.
- `●` (Green): **Normal / Safe** - Up-to-date or generic safe state.

---

Inspired by [deadbranch](https://github.com/armgabrielyan/deadbranch).
Made by: **odiador** ❤️ for the community.
