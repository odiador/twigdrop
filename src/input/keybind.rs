//! Central keybinding table: single source of truth for shortcuts.
//!
//! `docs/KEYBINDINGS.md` is generated from [`BINDINGS`]; `help.rs` and the
//! footer must consume this table instead of duplicating literals.
//! Pure function — no `App`/`UiState` access.

use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Where a binding applies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Context {
    Global,
    Branches,
    Diff,
    Modal,
}

/// Intent derived from a key press. Handlers dispatch on this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Quit,
    OpenPalette,
    OpenHelp,
    ToggleHelp,
    CloseOrBack,
    OpenFilter,
    CreateBranch,
    SafeDelete,
    Prune,
    CycleSort,
    AiAnalyze,
    OpenManage,
    ToggleSelect,
    NavigateUp,
    NavigateDown,
    OpenInspector,
    OpenSidebar,
    ToggleView,
    OpenStashManager,
    OpenCommits,
    OpenRebase,
    QuickActions,
    Shell,
    Switcher,
    ClearSearch,
}

/// One row of the canonical table.
pub struct Binding {
    pub id: &'static str,
    pub action: Action,
    pub context: Context,
    /// Canonical key description (human, matches KEYBINDINGS.md).
    pub keys: &'static str,
    /// i18n key for the description (present or future in `src/i18n`).
    pub i18n_key: &'static str,
}

/// Canonical table. Keep sorted by `id`; `docs/KEYBINDINGS.md` mirrors it.
pub static BINDINGS: &[Binding] = &[
    Binding {
        id: "ai.analyze",
        action: Action::AiAnalyze,
        context: Context::Branches,
        keys: "Ctrl+I",
        i18n_key: "branches.act_diff_ai_desc",
    },
    Binding {
        id: "branches.clear_search",
        action: Action::ClearSearch,
        context: Context::Branches,
        keys: "Esc (with query)",
        i18n_key: "branches.esc_to_clear",
    },
    Binding {
        id: "branches.create",
        action: Action::CreateBranch,
        context: Context::Branches,
        keys: "Ctrl+C",
        i18n_key: "modals.cmd_create_branch",
    },
    Binding {
        id: "branches.cycle_sort",
        action: Action::CycleSort,
        context: Context::Branches,
        keys: "Ctrl+S",
        i18n_key: "branches.sort_label",
    },
    Binding {
        id: "branches.filter",
        action: Action::OpenFilter,
        context: Context::Branches,
        keys: "Ctrl+F",
        i18n_key: "branches.header_sync",
    },
    Binding {
        id: "branches.inspector",
        action: Action::OpenInspector,
        context: Context::Branches,
        keys: "Right",
        i18n_key: "branches.inspector_hint",
    },
    Binding {
        id: "branches.manage",
        action: Action::OpenManage,
        context: Context::Branches,
        keys: "Enter",
        i18n_key: "branches.manage_title",
    },
    Binding {
        id: "branches.navigate_down",
        action: Action::NavigateDown,
        context: Context::Branches,
        keys: "Down / j",
        i18n_key: "common.move_keys",
    },
    Binding {
        id: "branches.navigate_up",
        action: Action::NavigateUp,
        context: Context::Branches,
        keys: "Up / k",
        i18n_key: "common.move_keys",
    },
    Binding {
        id: "branches.prune",
        action: Action::Prune,
        context: Context::Branches,
        keys: "Ctrl+P",
        i18n_key: "branches.act_checkout_desc",
    },
    Binding {
        id: "branches.safe_delete",
        action: Action::SafeDelete,
        context: Context::Branches,
        keys: "Ctrl+D",
        i18n_key: "branches.act_delete_desc",
    },
    Binding {
        id: "branches.sidebar",
        action: Action::OpenSidebar,
        context: Context::Branches,
        keys: "Left",
        i18n_key: "branches.sidebar_hint",
    },
    Binding {
        id: "branches.toggle_select",
        action: Action::ToggleSelect,
        context: Context::Branches,
        keys: "Space",
        i18n_key: "branches.header_sel",
    },
    Binding {
        id: "branches.type_filter",
        action: Action::ClearSearch,
        context: Context::Branches,
        keys: "type [a-z0-9/_.-]",
        i18n_key: "branches.type_to_filter",
    },
    Binding {
        id: "global.close_back",
        action: Action::CloseOrBack,
        context: Context::Global,
        keys: "Esc",
        i18n_key: "common.close",
    },
    Binding {
        id: "global.commits",
        action: Action::OpenCommits,
        context: Context::Global,
        keys: "Shift+C",
        i18n_key: "modals.cmd_commit",
    },
    Binding {
        id: "global.help_toggle_ctrl",
        action: Action::ToggleHelp,
        context: Context::Global,
        keys: "Ctrl+H",
        i18n_key: "modals.help_branch_ops_title",
    },
    Binding {
        id: "global.help_toggle_q",
        action: Action::OpenHelp,
        context: Context::Global,
        keys: "? / h (legacy, see debt)",
        i18n_key: "modals.help_branch_ops_title",
    },
    Binding {
        id: "global.palette",
        action: Action::OpenPalette,
        context: Context::Global,
        keys: "Ctrl+K",
        i18n_key: "modals.command_palette_title",
    },
    Binding {
        id: "global.quick_actions",
        action: Action::QuickActions,
        context: Context::Global,
        keys: ":",
        i18n_key: "modals.main_menu_quick_actions",
    },
    Binding {
        id: "global.quit",
        action: Action::Quit,
        context: Context::Global,
        keys: "Ctrl+Q",
        i18n_key: "common.quit",
    },
    Binding {
        id: "global.rebase",
        action: Action::OpenRebase,
        context: Context::Global,
        keys: "Shift+R",
        i18n_key: "modals.cmd_rebase",
    },
    Binding {
        id: "global.shell",
        action: Action::Shell,
        context: Context::Global,
        keys: "!",
        i18n_key: "modals.cmd_commit",
    },
    Binding {
        id: "global.stash_mgr",
        action: Action::OpenStashManager,
        context: Context::Global,
        keys: "Shift+S",
        i18n_key: "modals.cmd_stash",
    },
    Binding {
        id: "global.switcher",
        action: Action::Switcher,
        context: Context::Global,
        keys: "Shift+Tab",
        i18n_key: "modals.main_menu_app_switcher",
    },
    Binding {
        id: "global.toggle_view",
        action: Action::ToggleView,
        context: Context::Global,
        keys: "Alt+D",
        i18n_key: "modals.main_menu_app_switcher",
    },
    Binding {
        id: "diff.ai_autofix",
        action: Action::AiAnalyze,
        context: Context::Diff,
        keys: "Shift+F",
        i18n_key: "branches.act_diff_ai_desc",
    },
];

/// Look up a binding by id (for help screens / docs generation).
#[must_use]
pub fn find(id: &str) -> Option<&'static Binding> {
    BINDINGS.iter().find(|b| b.id == id)
}

/// Canonical key label for a binding id. Falls back to `"?"` so help
/// never renders empty when a row is missing.
#[must_use]
pub fn keys(id: &str) -> &'static str {
    find(id).map_or("?", |b| b.keys)
}

/// Pure mapping: `(modifiers, code)` → `Action`. No state access.
#[must_use]
pub fn from_key(key: KeyEvent) -> Option<Action> {
    use Action as A;
    let m = key.modifiers;
    let ctrl = m.contains(KeyModifiers::CONTROL);
    let alt = m.contains(KeyModifiers::ALT);
    let shift = m.contains(KeyModifiers::SHIFT);
    let none = m.is_empty();

    // Ctrl layer (safe operations) — checked first so Ctrl+letter never
    // falls through to type-to-filter.
    if ctrl && !alt {
        return match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => Some(A::Quit),
            KeyCode::Char('k') | KeyCode::Char('K') => Some(A::OpenPalette),
            KeyCode::Char('h') | KeyCode::Char('H') => Some(A::ToggleHelp),
            KeyCode::Char('c') | KeyCode::Char('C') => Some(A::CreateBranch),
            KeyCode::Char('f') | KeyCode::Char('F') => Some(A::OpenFilter),
            KeyCode::Char('d') | KeyCode::Char('D') => Some(A::SafeDelete),
            KeyCode::Char('p') | KeyCode::Char('P') => Some(A::Prune),
            KeyCode::Char('s') | KeyCode::Char('S') => Some(A::CycleSort),
            KeyCode::Char('i') | KeyCode::Char('I') => Some(A::AiAnalyze),
            _ => None,
        };
    }
    // Alt layer.
    if alt && !ctrl {
        return match key.code {
            KeyCode::Char('d') | KeyCode::Char('D') => Some(A::ToggleView),
            _ => None,
        };
    }
    // Shift-letter layer: crossterm delivers these as uppercase Char + SHIFT.
    // Accept both bare-uppercase (some terminals strip SHIFT) and explicit SHIFT.
    let shift_action = match key.code {
        KeyCode::Char('C') if shift || none => Some(A::OpenCommits),
        KeyCode::Char('R') if shift || none => Some(A::OpenRebase),
        KeyCode::Char('S') if shift || none => Some(A::OpenStashManager),
        _ => None,
    };
    if shift_action.is_some() {
        return shift_action;
    }
    // No-modifier layer.
    if none || (shift && matches!(key.code, KeyCode::BackTab)) {
        match key.code {
            KeyCode::Esc => Some(A::CloseOrBack),
            KeyCode::BackTab => Some(A::Switcher),
            KeyCode::Char('?') => Some(A::OpenHelp),
            KeyCode::Char(':') => Some(A::QuickActions),
            KeyCode::Char('!') => Some(A::Shell),
            KeyCode::Enter => Some(A::OpenManage),
            KeyCode::Char(' ') => Some(A::ToggleSelect),
            KeyCode::Up | KeyCode::Char('k') => Some(A::NavigateUp),
            KeyCode::Down | KeyCode::Char('j') => Some(A::NavigateDown),
            KeyCode::Right => Some(A::OpenInspector),
            KeyCode::Left => Some(A::OpenSidebar),
            _ => None,
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::crossterm::event::{KeyEventKind, KeyEventState};

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent {
            code,
            modifiers,
            kind: KeyEventKind::Press,
            state: KeyEventState::empty(),
        }
    }

    #[test]
    fn ctrl_layer_never_falls_through_to_filter() {
        assert_eq!(
            from_key(key(KeyCode::Char('d'), KeyModifiers::CONTROL)),
            Some(Action::SafeDelete)
        );
        assert_eq!(
            from_key(key(KeyCode::Char('q'), KeyModifiers::CONTROL)),
            Some(Action::Quit)
        );
        assert_eq!(
            from_key(key(KeyCode::Char('k'), KeyModifiers::CONTROL)),
            Some(Action::OpenPalette)
        );
    }

    #[test]
    fn alt_d_toggles_view_without_state() {
        assert_eq!(
            from_key(key(KeyCode::Char('d'), KeyModifiers::ALT)),
            Some(Action::ToggleView)
        );
    }

    #[test]
    fn shift_letters_accept_uppercase_with_or_without_flag() {
        // Some terminals strip SHIFT and deliver bare 'C'.
        assert_eq!(
            from_key(key(KeyCode::Char('C'), KeyModifiers::empty())),
            Some(Action::OpenCommits)
        );
        assert_eq!(
            from_key(key(KeyCode::Char('S'), KeyModifiers::SHIFT)),
            Some(Action::OpenStashManager)
        );
        assert_eq!(
            from_key(key(KeyCode::Char('R'), KeyModifiers::SHIFT)),
            Some(Action::OpenRebase)
        );
    }

    #[test]
    fn esc_question_and_backtab() {
        use KeyModifiers as M;
        assert_eq!(
            from_key(key(KeyCode::Esc, M::empty())),
            Some(Action::CloseOrBack)
        );
        assert_eq!(
            from_key(key(KeyCode::Char('?'), M::empty())),
            Some(Action::OpenHelp)
        );
        assert_eq!(
            from_key(key(KeyCode::BackTab, M::SHIFT)),
            Some(Action::Switcher)
        );
    }

    #[test]
    fn bindings_table_covers_all_actions() {
        for id in [
            "global.quit",
            "global.palette",
            "branches.safe_delete",
            "branches.create",
            "global.toggle_view",
        ] {
            assert!(BINDINGS.iter().any(|b| b.id == id), "missing {id}");
        }
        // ids unique
        let mut ids: Vec<_> = BINDINGS.iter().map(|b| b.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), BINDINGS.len());
    }

    #[test]
    fn help_referenced_ids_all_resolve_to_short_labels() {
        // Every id consumed by `help.rs` via `keys()` must exist and stay
        // display-safe (no `⚠` notes — those live in docs/KEYBINDINGS.md).
        for id in [
            "branches.type_filter",
            "branches.toggle_select",
            "branches.manage",
            "branches.prune",
            "branches.safe_delete",
            "branches.cycle_sort",
            "branches.create",
            "branches.filter",
            "ai.analyze",
            "global.palette",
            "global.help_toggle_ctrl",
            "global.quit",
            "global.switcher",
            "global.toggle_view",
            "global.close_back",
            "global.commits",
            "global.stash_mgr",
            "global.rebase",
            "diff.ai_autofix",
            "global.quick_actions",
            "global.shell",
        ] {
            let label = keys(id);
            assert_ne!(label, "?", "missing binding {id}");
            assert!(
                !label.contains('⚠'),
                "label for {id} must stay short, notes go in docs"
            );
        }
    }
}
