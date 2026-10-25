use crate::app::App;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle(app: &mut App, key: KeyEvent) -> Option<bool> {
    if key.code == KeyCode::Esc
        || key.code == KeyCode::Char('q')
        || (key.modifiers.contains(KeyModifiers::CONTROL)
            && (key.code == KeyCode::Char('h') || key.code == KeyCode::Char('H')))
        || key.code == KeyCode::Char('?')
    {
        app.ui.pop_modal();
    }
    Some(false)
}
