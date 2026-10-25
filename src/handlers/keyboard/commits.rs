use crate::app::App;
use crate::git::files::FileEntry;
use crate::state::ui::{AppMode, DatePickerField, DatePickerState, PrimaryMode};
use chrono::Datelike;
use chrono::Timelike;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_commits_keyboard(app: &mut App, key: KeyEvent, _path: &str) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.ui.selected_commit_idx > 0 => {
            app.ui.selected_commit_idx -= 1;
            false
        }
        KeyCode::Down | KeyCode::Char('j')
            if app.ui.selected_commit_idx < app.repo.commit_tree.len().saturating_sub(1) =>
        {
            app.ui.selected_commit_idx += 1;
            false
        }
        KeyCode::Enter => {
            if let Some(commit) = app.repo.commit_tree.get(app.ui.selected_commit_idx)
                && !commit.hash.is_empty()
            {
                app.ui
                    .push_modal(AppMode::CommitAction(commit.hash.clone()));
                app.ui.settings_state.selected = 0;
            }
            false
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.primary_mode = PrimaryMode::Branches;
            false
        }
        _ => false,
    }
}

pub fn handle_commit_action_keyboard(app: &mut App, key: KeyEvent, hash: &str, path: &str) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.ui.settings_state.selected > 0 => {
            app.ui.settings_state.selected -= 1;
        }
        KeyCode::Down | KeyCode::Char('j') if app.ui.settings_state.selected < 4 => {
            app.ui.settings_state.selected += 1;
        }
        KeyCode::Enter => {
            if app.ui.settings_state.selected == 0 {
                let date_str =
                    crate::actions::commands::resolve_commit_date(path, hash).unwrap_or_default();

                let datetime = chrono::DateTime::parse_from_rfc3339(date_str.trim())
                    .map(|dt| dt.with_timezone(&chrono::Local))
                    .unwrap_or_else(|_| chrono::Local::now());

                app.ui.push_modal(AppMode::DatePicker(DatePickerState {
                    year: datetime.year(),
                    month: datetime.month(),
                    day: datetime.day(),
                    hour: datetime.hour(),
                    minute: datetime.minute(),
                    second: datetime.second(),
                    active_field: DatePickerField::Day,
                }));
            } else if app.ui.settings_state.selected == 1 {
                let msg = crate::actions::commands::fixup_commit(path, hash)
                    .unwrap_or_else(|e| e.to_string());
                app.ui.pop_modal();
                app.ui.push_modal(AppMode::Message(msg));
            } else if app.ui.settings_state.selected == 2 {
                let files = crate::git::files::get_commit_files(path, hash);
                app.ui.selected_commit_file_idx = 0;
                app.ui
                    .push_modal(AppMode::CommitFiles(hash.to_string(), files));
            } else if app.ui.settings_state.selected == 3 {
                let short_hash = hash.to_string();
                let msg = match crate::actions::commands::squash_into_parent(path, &short_hash) {
                    Ok(m) => {
                        app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
                        m
                    }
                    Err(e) => format!("Failed to squash: {e}"),
                };
                app.ui.pop_modal();
                app.ui.push_modal(AppMode::Message(msg));
            } else if app.ui.settings_state.selected == 4 {
                app.load_rebase_commits_from_hash(path, hash);
                app.ui.pop_modal();
                if app.ui.rebase_state.commits.is_empty() {
                    app.ui
                        .push_modal(AppMode::Message("No commits to rebase.".to_string()));
                } else {
                    app.ui.push_modal(AppMode::InteractiveRebase);
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    false
}

pub fn handle_commit_files_keyboard(
    app: &mut App,
    key: KeyEvent,
    hash: &str,
    files: &[FileEntry],
    path: &str,
) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.ui.selected_commit_file_idx > 0 => {
            app.ui.selected_commit_file_idx -= 1;
        }
        KeyCode::Down | KeyCode::Char('j')
            if app.ui.selected_commit_file_idx < files.len().saturating_sub(1) =>
        {
            app.ui.selected_commit_file_idx += 1;
        }
        KeyCode::Char('u') => {
            if let Some(entry) = files.get(app.ui.selected_commit_file_idx) {
                let file_path = entry.path.to_string_lossy().to_string();
                let msg = match crate::actions::commands::move_file_changes_forward(
                    path, hash, &file_path,
                ) {
                    Ok(m) => m,
                    Err(e) => format!("Error: {e}"),
                };
                app.ui.pop_modal(); // pop CommitFiles
                app.ui.pop_modal(); // pop CommitAction
                app.ui.push_modal(AppMode::Message(msg));
                app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    false
}
