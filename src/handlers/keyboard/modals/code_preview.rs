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
