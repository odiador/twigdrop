use crate::app::App;
use crate::state::ui::{AppMode, PrimaryMode, SwitchTarget, View};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent, path: &str) -> Option<bool> {
    match key.code {
        KeyCode::Enter => {
            let order = app.ui.switcher_order();
            let target = order
                .get(app.ui.switcher_index)
                .copied()
                .unwrap_or(SwitchTarget::View(View::Branches));
            app.ui.pop_modal();
            app.ui.modal_stack.clear();
            match target {
                SwitchTarget::View(View::Branches) => {
                    app.set_primary_mode(PrimaryMode::Branches);
                    app.ui.track_history(View::Branches);
                }
                SwitchTarget::View(View::Files) => {
                    app.set_primary_mode(PrimaryMode::Files);
                    app.load_file_tree(path);
                    app.ui.track_history(View::Files);
                }
                SwitchTarget::View(View::Commits) => {
                    app.set_primary_mode(PrimaryMode::Commits);
                    app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
                    app.ui.selected_commit_idx = 0;
                    app.ui.track_history(View::Commits);
                }
                SwitchTarget::View(View::Stashes) => {
                    app.set_primary_mode(PrimaryMode::Stashes);
                    app.load_stashes(path);
                    app.load_stash_detail(path);
                    app.ui.track_history(View::Stashes);
                }
                SwitchTarget::Help => {
                    app.ui.push_modal(AppMode::Help);
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.ui.switcher_index = app.ui.switcher_index.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j')
            if app.ui.switcher_index + 1 < app.ui.switcher_order().len() =>
        {
            app.ui.switcher_index += 1;
        }
        _ => {}
    }
    Some(false)
}
