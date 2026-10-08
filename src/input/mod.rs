//! Unified input module: stateless `KeyEvent → Action` mapping + dispatch.
//!
//! Canonical spec: `docs/ARCHITECTURE.md` §4, bindings table: `docs/KEYBINDINGS.md`.
//! Handlers must match on `(modifiers, code)` per event — never on stored
//! `alt_pressed / shift_pressed` booleans (unreliable in tmux/SSH, see arch doc).

pub mod keybind;

pub use keybind::{Action, BINDINGS, Binding, Context, find, from_key, keys};
