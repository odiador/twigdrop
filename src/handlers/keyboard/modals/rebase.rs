use crate::app::App;
use crate::state::ui::{AppMode, RebaseAction};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent, path: &str) -> Option<bool> {
    if app.ui.rebase_state.editing {
        match key.code {
            KeyCode::Enter => {
                app.ui.rebase_state.editing = false;
                let i = app.ui.rebase_state.selected;
                app.ui.rebase_state.commits[i].new_message =
                    Some(app.ui.rebase_state.input.clone());
                app.ui.rebase_state.commits[i].action = RebaseAction::Reword;
            }
            KeyCode::Esc => {
                app.ui.rebase_state.editing = false;
            }
            KeyCode::Char(c) => {
                app.ui.rebase_state.input.push(c);
            }
            KeyCode::Backspace => {
                app.ui.rebase_state.input.pop();
            }
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if app.ui.rebase_state.selected > 0 => {
                app.ui.rebase_state.selected -= 1;
            }
            KeyCode::Down | KeyCode::Char('j')
                if app.ui.rebase_state.selected
                    < app.ui.rebase_state.commits.len().saturating_sub(1) =>
            {
                app.ui.rebase_state.selected += 1;
            }
            KeyCode::Char('r') => {
                app.ui.rebase_state.commits[app.ui.rebase_state.selected].action =
                    RebaseAction::Reword;
            }
            KeyCode::Char('d') => {
                app.ui.rebase_state.commits[app.ui.rebase_state.selected].action =
                    RebaseAction::Drop;
            }
            KeyCode::Char('s') => {
                app.ui.rebase_state.commits[app.ui.rebase_state.selected].action =
                    RebaseAction::Squash;
            }
            KeyCode::Char('p') => {
                app.ui.rebase_state.commits[app.ui.rebase_state.selected].action =
                    RebaseAction::Pick;
            }
            KeyCode::Enter => {
                app.ui.rebase_state.editing = true;
                let i = app.ui.rebase_state.selected;
                app.ui.rebase_state.input = app.ui.rebase_state.commits[i]
                    .new_message
                    .clone()
                    .unwrap_or_else(|| app.ui.rebase_state.commits[i].original_message.clone());
            }
            KeyCode::Char('A') => {
                app.ui.rebase_state.ai_analyzing = true;
                let i = app.ui.rebase_state.selected;
                let hash = app.ui.rebase_state.commits[i].hash.clone();
                let _ = app.ai_trigger_tx.try_send((
                    "rename_commit".to_string(),
                    path.to_string(),
                    hash,
                ));
            }
            KeyCode::Char('E') => {
                crate::actions::commands::execute_interactive_rebase(
                    path,
                    &app.ui.rebase_state.commits,
                );
                app.ui.pop_modal();
                app.ui
                    .push_modal(AppMode::Message("Rebase executed.".to_string()));
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui.pop_modal();
            }
            _ => {}
        }
    }
    Some(false)
}
