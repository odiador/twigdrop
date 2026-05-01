# Twigdrop Architecture

Twigdrop is a TUI tool for managing local git branches. It follows a modular design to separate concerns between state management, UI rendering, and Git interaction.

## Directory Structure

```text
src/
├── main.rs          # Entry point and terminal orchestration
├── app.rs           # Application state (App struct)
├── models.rs        # Core data types (Branch, BranchStatus)
├── actions/         # Business logic for git operations
│   ├── mod.rs       # Exporting actions
│   └── commands.rs  # Implementation of delete, checkout, etc.
├── git/             # Git introspection logic
│   ├── mod.rs       # High-level data building
│   ├── commands.rs  # Low-level git command runner
│   └── status.rs    # Status detection (merged, stashed, etc.)
├── handlers/        # Input event processing
│   ├── mod.rs       # Event loop routing
│   ├── keyboard.rs  # Keyboard input handling
│   └── mouse.rs     # Mouse input and coordinate mapping
└── ui/              # Rendering logic
    ├── mod.rs       # Main draw entry point
    ├── screens.rs   # Layouts for Normal, Help, Manage, Diff
    └── components.rs # Reusable widgets and styling
```

## Data Flow

1.  **Initialization**: `main.rs` initializes the terminal and the `App` state.
2.  **Git Introspection**: The `git/` module queries the local repository to build a list of `Branch` objects with their `BranchStatus`.
3.  **Event Loop**: `main.rs` listens for events (keyboard/mouse).
4.  **Handling**: `handlers/` processes events, potentially updating the `App` state or triggering an `Action`.
5.  **Rendering**: `ui/` takes the `App` state and renders the current frame to the terminal.

## Key Design Principles

- **Immutability**: The UI should not modify the state; it only reads from `App`.
- **Separation of Concerns**: Git commands are isolated from the UI logic.
- **Robustness**: Error messages are passed through the `App` state to be displayed to the user.
