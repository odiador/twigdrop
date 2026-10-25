use crate::app::App;
use crate::state::ui::AppMode;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent, path: &str) -> Option<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.ui.quick_actions_state.selected > 0 => {
            app.ui.quick_actions_state.selected -= 1;
        }
        KeyCode::Down | KeyCode::Char('j')
            if app.ui.quick_actions_state.selected
                < app.ui.quick_actions_state.actions.len().saturating_sub(1) =>
        {
            app.ui.quick_actions_state.selected += 1;
        }
        KeyCode::Enter => {
            let action =
                app.ui.quick_actions_state.actions[app.ui.quick_actions_state.selected].clone();
            let parts: Vec<&str> = action.split_whitespace().collect();
            if parts.len() >= 2 && parts[0] == "git" {
                let args = &parts[1..];
                let msg = match crate::actions::commands::run_arbitrary_git(path, args) {
                    Ok(m) => format!("> {action}\n{m}"),
                    Err(e) => format!("Error: {e}"),
                };
                app.refresh_branches(path);
                app.ui.pop_modal();
                app.ui.push_modal(AppMode::Message(msg));
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    Some(false)
}
