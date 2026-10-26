use crate::app::App;
use crate::git::conflicts::{self, ConflictChoice};
use crate::state::ui::{AppMode, ConflictResolverState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(
    app: &mut App,
    key: KeyEvent,
    mut state: ConflictResolverState,
    path: &str,
) -> Option<bool> {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
            Some(false)
        }
        KeyCode::Tab => {
            if !state.files.is_empty() {
                state.selected_file_idx = (state.selected_file_idx + 1) % state.files.len();
                let selected_file = &state.files[state.selected_file_idx];
                state.hunks = conflicts::parse_file_conflicts(path, selected_file);
                state.selected_hunk_idx = 0;
                if let AppMode::ConflictResolver(s) = app.ui.current_mode_mut() {
                    *s = state;
                }
            }
            Some(false)
        }
        KeyCode::Char('n') | KeyCode::Down | KeyCode::Char('j') => {
            if !state.hunks.is_empty() && state.selected_hunk_idx + 1 < state.hunks.len() {
                state.selected_hunk_idx += 1;
            } else if !state.files.is_empty() && state.selected_file_idx + 1 < state.files.len() {
                state.selected_file_idx += 1;
                let selected_file = &state.files[state.selected_file_idx];
                state.hunks = conflicts::parse_file_conflicts(path, selected_file);
                state.selected_hunk_idx = 0;
            }
            if let AppMode::ConflictResolver(s) = app.ui.current_mode_mut() {
                *s = state;
            }
            Some(false)
        }
        KeyCode::Char('p') | KeyCode::Up | KeyCode::Char('k') => {
            if state.selected_hunk_idx > 0 {
                state.selected_hunk_idx -= 1;
            } else if state.selected_file_idx > 0 {
                state.selected_file_idx -= 1;
                let selected_file = &state.files[state.selected_file_idx];
                state.hunks = conflicts::parse_file_conflicts(path, selected_file);
                state.selected_hunk_idx = state.hunks.len().saturating_sub(1);
            }
            if let AppMode::ConflictResolver(s) = app.ui.current_mode_mut() {
                *s = state;
            }
            Some(false)
        }
        KeyCode::Char('o') | KeyCode::Char('t') | KeyCode::Char('b') => {
            if state.files.is_empty() || state.hunks.is_empty() {
                return Some(false);
            }

            let choice = match key.code {
                KeyCode::Char('o') => ConflictChoice::Ours,
                KeyCode::Char('t') => ConflictChoice::Theirs,
                _ => ConflictChoice::Both,
            };

            let selected_file = state.files[state.selected_file_idx].clone();
            let hunk = &state.hunks[state.selected_hunk_idx.min(state.hunks.len() - 1)];

            match conflicts::resolve_hunk_in_file(path, &selected_file, hunk, choice) {
                Ok(fully_resolved) => {
                    let choice_label = match choice {
                        ConflictChoice::Ours => "Accepted Ours",
                        ConflictChoice::Theirs => "Accepted Theirs",
                        ConflictChoice::Both => "Accepted Both",
                    };
                    app.ui.notify(
                        crate::state::toast::ToastKind::Success,
                        format!("{choice_label} in {selected_file}"),
                        None,
                    );

                    // Re-parse remaining hunks in this file
                    state.hunks = conflicts::parse_file_conflicts(path, &selected_file);
                    if state.selected_hunk_idx >= state.hunks.len() {
                        state.selected_hunk_idx = state.hunks.len().saturating_sub(1);
                    }

                    if fully_resolved {
                        let remaining_files = crate::git::status::get_conflicting_files(path);
                        state.files = remaining_files.clone();
                        if state.files.is_empty() {
                            app.ui.notify(
                                crate::state::toast::ToastKind::Success,
                                "All conflicts resolved! Press 'c' to continue rebase/merge."
                                    .to_string(),
                                None,
                            );
                        } else {
                            if state.selected_file_idx >= state.files.len() {
                                state.selected_file_idx = 0;
                            }
                            let next_file = &state.files[state.selected_file_idx];
                            state.hunks = conflicts::parse_file_conflicts(path, next_file);
                            state.selected_hunk_idx = 0;
                        }
                    }

                    if let AppMode::ConflictResolver(s) = app.ui.current_mode_mut() {
                        *s = state;
                    }
                }
                Err(e) => {
                    app.ui.notify(
                        crate::state::toast::ToastKind::Error,
                        format!("Resolution failed: {e}"),
                        None,
                    );
                }
            }
            Some(false)
        }
        KeyCode::Char('c') => {
            if crate::git::status::is_rebasing(path) {
                match crate::actions::commands::continue_rebase(path) {
                    Ok(msg) => {
                        app.ui.pop_modal();
                        app.refresh_branches(path);
                        app.ui.notify(
                            crate::state::toast::ToastKind::Success,
                            format!("Rebase continued: {msg}"),
                            None,
                        );
                    }
                    Err(e) => {
                        app.ui.notify(
                            crate::state::toast::ToastKind::Error,
                            format!("Rebase continue failed: {e}"),
                            None,
                        );
                    }
                }
            } else if crate::git::status::is_merging(path) {
                match crate::git::commands::run_git(path, &["commit", "--no-edit"]) {
                    Ok(_) => {
                        app.ui.pop_modal();
                        app.refresh_branches(path);
                        app.ui.notify(
                            crate::state::toast::ToastKind::Success,
                            "Merge completed successfully!".to_string(),
                            None,
                        );
                    }
                    Err(e) => {
                        app.ui.notify(
                            crate::state::toast::ToastKind::Error,
                            format!("Merge commit failed: {e}"),
                            None,
                        );
                    }
                }
            } else {
                app.ui.notify(
                    crate::state::toast::ToastKind::Info,
                    "No rebase or merge in progress to continue.".to_string(),
                    None,
                );
            }
            Some(false)
        }
        _ => None,
    }
}
