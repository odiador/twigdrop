use crate::app::App;
use crate::state::ui::AppMode;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent, path: &str, input: String) -> Option<bool> {
    match key.code {
        KeyCode::Enter => {
            if !input.is_empty() {
                let parts: Vec<&str> = input.split_whitespace().collect();
                let msg = match crate::actions::commands::run_arbitrary_git(path, &parts) {
                    Ok(m) => format!("$ {input}\n{m}"),
                    Err(e) => format!("Error: {e}"),
                };
                app.ui.pop_modal();
                app.ui.push_modal(AppMode::Message(msg));
            } else {
                app.ui.pop_modal();
            }
        }
        KeyCode::Esc => {
            app.ui.pop_modal();
        }
        KeyCode::Char(c) => {
            if let AppMode::Shell(inp) = app.ui.current_mode_mut() {
                inp.push(c);
            }
        }
        KeyCode::Backspace => {
            if let AppMode::Shell(inp) = app.ui.current_mode_mut() {
                inp.pop();
            }
        }
        _ => {}
    }
    Some(false)
}
