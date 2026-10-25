use crate::app::App;
use crate::state::ui::AppMode;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Option<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.ui.main_menu_state.selected > 0 => {
            app.ui.main_menu_state.selected -= 1;
        }
        KeyCode::Down | KeyCode::Char('j') if app.ui.main_menu_state.selected < 2 => {
            app.ui.main_menu_state.selected += 1;
        }
        KeyCode::Enter => match app.ui.main_menu_state.selected {
            0 => {
                app.ui.push_modal(AppMode::Settings);
                app.ui.settings_state.selected = 0;
                app.ui.settings_state.editing = false;
            }
            1 => app.ui.push_modal(AppMode::Help),
            2 => return Some(true),
            _ => {}
        },
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    Some(false)
}
