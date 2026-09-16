use crate::actions::apply_stash;
use crate::app::App;
use crate::state::ui::AppMode;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_stashes_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.ui.selected_stash_idx < app.repo.stashes.len().saturating_sub(1) {
                app.ui.selected_stash_idx += 1;
                app.load_stash_detail(path);
            }
            false
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.ui.selected_stash_idx > 0 {
                app.ui.selected_stash_idx -= 1;
                app.load_stash_detail(path);
            }
            false
        }
        KeyCode::Char('a') | KeyCode::Enter => {
            if let Some(stash) = app.repo.stashes.get(app.ui.selected_stash_idx) {
                let msg = apply_stash(path, &stash.id);
                app.refresh_branches(path);
                if *app.ui.current_mode() == AppMode::StashDetail {
                    app.ui.pop_modal();
                }
                app.ui.push_modal(AppMode::Message(msg));
            }
            false
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            if *app.ui.current_mode() == AppMode::StashDetail {
                app.ui.pop_modal();
            }
            false
        }
        _ => false,
    }
}
