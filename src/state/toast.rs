//! Transient toast notifications with optional undo.
//!
//! Toasts render as a one-line bar above the footer (`ui::render_chrome`)
//! and expire after [`TOAST_TTL`]. Destructive actions (branch delete)
//! attach [`UndoRestore`] data sourced from pre-delete SHAs so `u`
//! recreates the branches exactly where they were.

use std::time::{Duration, Instant};

/// How long a toast stays visible.
pub const TOAST_TTL: Duration = Duration::from_secs(8);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndoRestore {
    /// `(branch name, sha)` pairs captured before deletion.
    pub branches: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct Toast {
    pub kind: ToastKind,
    pub text: String,
    pub undo: Option<UndoRestore>,
    pub expires_at: Instant,
}

impl Toast {
    pub fn new(kind: ToastKind, text: String, undo: Option<UndoRestore>) -> Self {
        Self {
            kind,
            text,
            undo,
            expires_at: Instant::now() + TOAST_TTL,
        }
    }

    #[must_use]
    pub fn is_live(&self) -> bool {
        Instant::now() < self.expires_at
    }

    #[must_use]
    pub fn can_undo(&self) -> bool {
        self.undo.is_some() && self.is_live()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_with_undo_is_undoable_while_live() {
        let toast = Toast::new(
            ToastKind::Success,
            "Deleted 2 branches".to_string(),
            Some(UndoRestore {
                branches: vec![("feature/a".to_string(), "abc123".to_string())],
            }),
        );
        assert!(toast.is_live());
        assert!(toast.can_undo());
    }

    #[test]
    fn toast_without_undo_cannot_undo() {
        let toast = Toast::new(ToastKind::Info, "hello".to_string(), None);
        assert!(toast.is_live());
        assert!(!toast.can_undo());
    }
}
