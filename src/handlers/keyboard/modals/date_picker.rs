use crate::app::App;
use crate::state::ui::{AppMode, DatePickerField, DatePickerState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(
    app: &mut App,
    key: KeyEvent,
    mut state: DatePickerState,
    path: &str,
) -> Option<bool> {
    let res = handle_date_picker_keyboard(app, key, &mut state, path);
    if let AppMode::DatePicker(current_state) = app.ui.current_mode_mut() {
        *current_state = state;
    }
    Some(res)
}

pub fn handle_date_picker_keyboard(
    app: &mut App,
    key: KeyEvent,
    state: &mut DatePickerState,
    path: &str,
) -> bool {
    match key.code {
        KeyCode::Left | KeyCode::BackTab => {
            state.active_field = match state.active_field {
                DatePickerField::Year => DatePickerField::Minute,
                DatePickerField::Month => DatePickerField::Year,
                DatePickerField::Day => DatePickerField::Month,
                DatePickerField::Hour => DatePickerField::Day,
                DatePickerField::Minute => DatePickerField::Hour,
            };
        }
        KeyCode::Right | KeyCode::Tab => {
            state.active_field = match state.active_field {
                DatePickerField::Year => DatePickerField::Month,
                DatePickerField::Month => DatePickerField::Day,
                DatePickerField::Day => DatePickerField::Hour,
                DatePickerField::Hour => DatePickerField::Minute,
                DatePickerField::Minute => DatePickerField::Year,
            };
        }
        KeyCode::Up | KeyCode::Char('+') | KeyCode::Char('k') => match state.active_field {
            DatePickerField::Year => state.year -= 1,
            DatePickerField::Month => {
                state.month = if state.month == 1 {
                    12
                } else {
                    state.month - 1
                };
                let max_days = crate::utils::days_in_month(state.month, state.year);
                if state.day > max_days {
                    state.day = max_days;
                }
            }
            DatePickerField::Day => {
                let max_days = crate::utils::days_in_month(state.month, state.year);
                state.day = if state.day == 1 {
                    max_days
                } else {
                    state.day - 1
                };
            }
            DatePickerField::Hour => state.hour = if state.hour == 0 { 23 } else { state.hour - 1 },
            DatePickerField::Minute => {
                state.minute = if state.minute == 0 {
                    59
                } else {
                    state.minute - 1
                }
            }
        },
        KeyCode::Down | KeyCode::Char('-') | KeyCode::Char('j') => match state.active_field {
            DatePickerField::Year => state.year += 1,
            DatePickerField::Month => {
                state.month = (state.month % 12) + 1;
                let max_days = crate::utils::days_in_month(state.month, state.year);
                if state.day > max_days {
                    state.day = max_days;
                }
            }
            DatePickerField::Day => {
                let max_days = crate::utils::days_in_month(state.month, state.year);
                state.day = (state.day % max_days) + 1;
            }
            DatePickerField::Hour => state.hour = (state.hour + 1) % 24,
            DatePickerField::Minute => state.minute = (state.minute + 1) % 60,
        },
        KeyCode::Enter => {
            let new_date = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                state.year, state.month, state.day, state.hour, state.minute, state.second
            );

            if let Some(AppMode::CommitAction(hash)) =
                app.ui.modal_stack.iter().rev().nth(1).map(|m| &m.mode)
            {
                let short_hash = hash.clone();
                let full_hash = crate::git::commands::run_git(path, &["rev-parse", &short_hash])
                    .unwrap_or(short_hash.clone());

                let exec_cmd = format!(
                    "if [ \"$(git rev-parse HEAD)\" = \"{full_hash}\" ]; then GIT_COMMITTER_DATE=\"{new_date}\" git commit --amend --no-edit --date=\"{new_date}\"; fi"
                );

                let msg = match crate::git::commands::run_git(
                    path,
                    &[
                        "rebase",
                        "--autostash",
                        &format!("{short_hash}^"),
                        "--exec",
                        &exec_cmd,
                    ],
                ) {
                    Ok(m) => {
                        app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
                        format!("Date updated to {new_date}.\n{m}")
                    }
                    Err(e) => format!("Failed to update date: {e}"),
                };
                app.ui.pop_modal();
                app.ui.pop_modal();
                app.ui.push_modal(AppMode::Message(msg));
            } else {
                app.ui.pop_modal();
            }
            return false;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
            return false;
        }
        _ => {}
    }
    false
}
