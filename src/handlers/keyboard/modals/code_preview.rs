use crate::app::App;
use crate::state::ui::{AppMode, FilePanel, PreviewState};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle(app: &mut App, key: KeyEvent, mut state: PreviewState) -> Option<bool> {
    if key.code == KeyCode::Char('Z') {
        app.ui.maximized_panel = !app.ui.maximized_panel;
        let msg = if app.ui.maximized_panel {
            "Panel maximized [Z: restore]"
        } else {
            "Panel restored"
        };
        app.ui
            .notify(crate::state::toast::ToastKind::Info, msg.to_string(), None);
        return Some(false);
    }

    if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
        if app.ui.maximized_panel {
            app.ui.maximized_panel = false;
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "Panel restored".to_string(),
                None,
            );
            return Some(false);
        }
        app.ui.pop_modal();
        app.ui.active_panel = FilePanel::Directory;
        Some(false)
    } else if key.code == KeyCode::Char('b') {
        state.show_blame = !state.show_blame;
        let msg = if state.show_blame {
            "Inline Blame: ON"
        } else {
            "Inline Blame: OFF"
        };
        app.ui
            .notify(crate::state::toast::ToastKind::Info, msg.to_string(), None);
        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
            *s = state;
        }
        Some(false)
    } else if key.code == KeyCode::Char('n') {
        if let Some(target) = state.jump_hunk(true) {
            app.ui.active_panel = FilePanel::Preview;
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                format!("Jumped to change at line {}", target + 1),
                None,
            );
        } else {
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "No changes in file".to_string(),
                None,
            );
        }
        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
            *s = state;
        }
        Some(false)
    } else if key.code == KeyCode::Char('N') {
        if let Some(target) = state.jump_hunk(false) {
            app.ui.active_panel = FilePanel::Preview;
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                format!("Jumped to change at line {}", target + 1),
                None,
            );
        } else {
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "No changes in file".to_string(),
                None,
            );
        }
        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
            *s = state;
        }
        Some(false)
    } else if key.code == KeyCode::Tab
        || key.code == KeyCode::BackTab
        || key.modifiers.contains(KeyModifiers::ALT)
        || key.modifiers.contains(KeyModifiers::CONTROL)
    {
        None
    } else if app.ui.active_panel == FilePanel::Preview {
        let res = super::super::files::handle_preview_keyboard(&mut state, key.code);
        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
            *s = state;
        }
        Some(res)
    } else {
        None
    }
}
