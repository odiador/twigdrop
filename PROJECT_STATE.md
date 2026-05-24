# Twigdrop: Git Branch Manager TUI

Twigdrop is a sophisticated Terminal User Interface (TUI) tool for managing local git branches, built with Rust and Ratatui. It goes beyond simple branch listing by providing deep introspection, AI-assisted conflict resolution, and a rich file exploration experience.

## 🚀 Current Implementation Status

### 1. Branch Management
- **Intelligent Listing**: Displays branches with rich metadata:
  - Status indicators: `Merged`, `Gone`, `Ahead`, `Behind`, `Stashed`, `Local`.
  - Author and commit age.
  - Ahead/Behind commit counts relative to upstream.
- **Introspection**:
  - Detailed view of branch logs and stats.
  - Unpushed commits tracking.
- **Advanced Operations**:
  - **Bulk Deletion**: Select multiple branches for deletion.
  - **Snap Animation**: Visual "Snap" effect when deleting multiple branches.
  - **Pruning**: Cleanup of stale branches.
- **Filtering & Search**: Real-time fuzzy search and filtering by status.

### 2. Git Introspection & Merge Analysis
- **Virtual Merge Testing**: Uses `git merge-tree` to analyze if a branch can be merged cleanly into another without performing the actual merge.
- **Conflict Detection**: Identifies specific conflict blocks and files before attempting a merge.
- **Structured Diffing**: View file diffs with syntax highlighting and hunk headers.

### 3. AI Integration
- **Conflict Resolution**: Integration with LLMs (via `rig`) to analyze and resolve merge conflicts.
- **Code Analysis**: AI-powered analysis of branches and code changes.
- **Persistence**: Uses `rusqlite` to store AI analysis and metadata.

### 4. File Explorer & Code Preview
- **Dual Primary Modes**: Seamless toggle between `Branches` and `Files` views.
- **File Tree**: Recursive file explorer with Git status indicators (Added, Modified, Deleted).
- **Rich Preview**:
  - Syntax highlighting for 100+ languages using `syntect`.
  - **Gutter Indicators**: Real-time display of added/modified/deleted lines within the preview.
  - Scrolling and selection support.

### 5. Stash Management
- **Stash Browser**: List all stashes with detailed information.
- **Deep Inspection**: View specific files changed in a stash and their diffs.
- **Operations**: Apply stashes directly from the TUI.

### 6. UI/UX Features
- **Animations**: Snap-to-delete animation for satisfying bulk cleanup.
- **Keyboard & Mouse**: Full keyboard navigation with mouse support for clicking and scrolling.
- **Interactive Shell**: Run quick git commands (`pull`, `push`, `fetch`, etc.) directly from a quick-action menu.
- **Message System**: Feedback system for operations (success/error notifications).

## 🛠️ Tech Stack

- **Core**: Rust (Edition 2024)
- **TUI**: `ratatui` & `crossterm`
- **Git Logic**: `git2-rs` & raw git command orchestration.
- **Async**: `tokio` for background tasks (merge analysis, AI).
- **AI**: `rig` library for LLM interaction.
- **Database**: `rusqlite` for local data storage.
- **Highlighting**: `syntect` for high-performance syntax highlighting.

## 📁 Architecture

- **`src/app.rs`**: Centralized state management using the `App` struct.
- **`src/git/`**: Modular logic for branch introspection, file status, and stash management.
- **`src/ui/`**: Declarative rendering with separate components for screens and widgets.
- **`src/handlers/`**: Decoupled event handling for keyboard and mouse.
- **`src/actions/`**: High-level business logic for git operations.
