use crate::app::App;
use crate::state::ui::{AppMode, FilePanel, PreviewState};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle(app: &mut App, key: KeyEvent, mut state: PreviewState) -> Option<bool> {
    if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
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
