# Twigdrop

Twigdrop is a fast, interactive Terminal User Interface (TUI) tool written in Rust to help you safely manage, inspect, and clean up your local git branches. It provides a visual representation of your branch statuses relative to their upstreams, preventing accidental deletion of unmerged work.

## Features

- **Visual Branch Status**: Quickly see if a branch is merged, ahead of upstream, has unique commits, or if its upstream is gone.
- **Batch Deletion**: Mark multiple branches and delete them all at once.
- **Detailed Diff & Info**: View the `git log --stat -p` of any branch inside a side-by-side terminal pane to know exactly what you are deleting.
- **Quick Checkout**: Switch between branches with a single keystroke or a mouse click.
- **Accessibility & Safety**: Uses distinct unicode icons and colors to denote danger (e.g., branches with unique local commits are marked in red with `▲`).

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

### Keybindings

| Key / Mouse | Action |
|-------------|--------|
| `↑` / `k`   | Move selection up (or scroll up in info panel) |
| `↓` / `j`   | Move selection down (or scroll down in info panel) |
| `Space`     | Toggle mark on the selected branch for batch deletion |
| `d`         | Delete all marked branches |
| `c`         | Checkout the currently selected branch |
| `Tab`       | Toggle the Info/Diff panel for the selected branch |
| `Left Click`| Jump to a specific branch in the list |
| `h`         | Toggle the Help & Legend modal |
| `q` / `Esc` | Quit the application (or close opened modals) |

### Legend

- `▲` (Red): **Has Unique Commits** - This branch has commits not present on the remote. Deleting it means losing work!
- `⨯` (Gray): **Gone** - The remote tracking branch has been deleted.
- `↑` (Yellow): **Ahead of upstream** - Local branch has commits that haven't been pushed yet.
- `✓` (Blue): **Merged** - Contains no unmerged commits. Safe to delete.
- `●` (Green): **Normal / Safe** - Up-to-date or generic safe state.

---
Made by: **odiador** ❤️ for the community.
