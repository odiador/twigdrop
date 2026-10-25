use crate::app::App;
use crate::state::ui::{AppMode, CommandAction, CommandPaletteState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(
    app: &mut App,
    key: KeyEvent,
    path: &str,
    mut state: CommandPaletteState,
) -> Option<bool> {
    let visible_actions: Vec<_> = state
        .actions
        .iter()
        .filter(|(name, _)| {
            state.query.is_empty() || name.to_lowercase().contains(&state.query.to_lowercase())
        })
        .cloned()
        .collect();

    match key.code {
        KeyCode::Esc => {
            app.ui.pop_modal();
        }
        KeyCode::Down if state.selected + 1 < visible_actions.len() => {
            state.selected += 1;
        }
        KeyCode::Up if state.selected > 0 => {
            state.selected -= 1;
        }
        KeyCode::Char(c) => {
            state.query.push(c);
            state.selected = 0;
        }
        KeyCode::Backspace => {
            state.query.pop();
            state.selected = 0;
        }
        KeyCode::Enter => {
            if let Some((_, action)) = visible_actions.get(state.selected) {
                let action = action.clone();
                app.ui.pop_modal();

                match action {
                    CommandAction::CreateBranch => {
                        app.ui.push_modal(AppMode::CreateBranch(String::new()));
                    }
                    CommandAction::CheckoutBranch => {
                        app.ui.push_modal(AppMode::Filter);
                        app.ui.filter_selected = 0;
                    }
                    CommandAction::OpenSettings => {
                        app.ui.push_modal(AppMode::Settings);
                    }
                    CommandAction::InteractiveRebase => {
                        app.ui.push_modal(AppMode::InteractiveRebase);
                        app.load_rebase_commits(path);
                    }
                    CommandAction::StashChanges => {
                        app.ui.push_modal(AppMode::Shell("git stash".to_string()));
                    }
                    CommandAction::PopStash => {
                        app.ui
                            .push_modal(AppMode::Shell("git stash pop".to_string()));
                    }
                    CommandAction::Fetch => {
                        app.ui.push_modal(AppMode::Shell("git fetch".to_string()));
                    }
                    CommandAction::Pull => {
                        app.ui.push_modal(AppMode::Shell("git pull".to_string()));
                    }
                    CommandAction::Push => {
                        app.ui.push_modal(AppMode::Shell("git push".to_string()));
                    }
                    CommandAction::CommitChanges => {
                        app.ui
                            .push_modal(AppMode::Shell("git commit -m \"\"".to_string()));
                    }
                    CommandAction::ContinueRebase => {
                        let msg = match crate::actions::commands::continue_rebase(path) {
                            Ok(m) => m,
                            Err(e) => {
                                let files = crate::git::status::get_conflicting_files(path);
                                if files.is_empty() {
                                    e.to_string()
                                } else {
                                    format!(
                                        "{e}\n\n{}:\n{}",
                                        app.locale().modals.conflicting_files,
                                        files.join("\n")
                                    )
                                }
                            }
                        };
                        app.refresh_branches(path);
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    CommandAction::AbortRebase => {
                        let msg = crate::actions::commands::abort_rebase(path)
                            .unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    CommandAction::AbortMerge => {
                        let msg = crate::actions::commands::abort_merge(path)
                            .unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                }
            } else {
                app.ui.pop_modal();
            }
            return Some(false);
        }
        _ => {}
    }
    if let AppMode::CommandPalette(s) = app.ui.current_mode_mut() {
        *s = state;
    }
    Some(false)
}
