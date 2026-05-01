use ratatui::crossterm::event::{KeyCode, KeyEvent};
use crate::app::{App, AppMode};
use crate::git;
use crate::models::BranchStatus;
use crate::actions::{delete_branch, checkout_branch};

pub fn handle_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    if let AppMode::Message(_) = app.mode {
        app.mode = AppMode::Normal;
        return false;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.mode != AppMode::Normal {
                app.mode = AppMode::Normal;
                false
            } else if app.current_filter.is_some() {
                app.current_filter = None;
                app.refresh_filtered_branches();
                app.selected = 0;
                false
            } else {
                true // Signal to quit
            }
        }
        KeyCode::Char('h') => {
            app.toggle_help();
            false
        }
        KeyCode::Char('f') => {
            if app.mode == AppMode::Normal {
                app.mode = AppMode::Filter;
                app.filter_selected = 0;
            }
            false
        }
        KeyCode::Char('m') | KeyCode::Tab | KeyCode::Enter => {
            handle_enter_or_selection(app, path)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match app.mode {
                AppMode::Diff => app.info_scroll += 1,
                AppMode::Manage => { if app.manage_selected < 4 { app.manage_selected += 1; } }
                AppMode::Filter => { if app.filter_selected < 9 { app.filter_selected += 1; } }
                _ => app.next(),
            }
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match app.mode {
                AppMode::Diff => app.info_scroll = app.info_scroll.saturating_sub(1),
                AppMode::Manage => { if app.manage_selected > 0 { app.manage_selected -= 1; } }
                AppMode::Filter => { if app.filter_selected > 0 { app.filter_selected -= 1; } }
                _ => app.previous(),
            }
            false
        }
        KeyCode::Char(c) if c.is_digit(10) => {
            let digit = c.to_digit(10).unwrap() as usize;
            match app.mode {
                AppMode::Filter => {
                    if digit < 10 {
                        app.filter_selected = digit;
                        handle_enter_or_selection(app, path);
                    }
                }
                AppMode::Manage => {
                    if digit >= 1 && digit <= 5 {
                        app.manage_selected = digit - 1;
                        handle_enter_or_selection(app, path);
                    }
                }
                _ => {}
            }
            false
        }
        _ => false
    }
}

fn handle_enter_or_selection(app: &mut App, path: &str) -> bool {
    match app.mode {
        AppMode::Normal => {
            if !app.get_filtered_branches().is_empty() {
                app.mode = AppMode::Manage;
                app.manage_selected = 0;
            }
        }
        AppMode::Manage => {
            let branch_name = {
                let filtered = app.get_filtered_branches();
                filtered.get(app.selected).map(|b| b.name.clone())
            };

            if let Some(name) = branch_name {
                match app.manage_selected {
                    0 => { // Checkout
                        let msg = checkout_branch(path, &name);
                        app.branches = git::build_branches(path);
                        app.refresh_filtered_branches();
                        app.current_branch = git::get_current_branch(path);
                        app.mode = AppMode::Message(msg);
                    }
                    1 => { // Diff
                        app.branch_info = git::get_branch_info(path, &name);
                        app.info_scroll = 0;
                        app.mode = AppMode::Diff;
                    }
                    2 => { // Delete
                        let msg = delete_branch(path, &name);
                        app.branches = git::build_branches(path);
                        app.refresh_filtered_branches();
                        app.current_branch = git::get_current_branch(path);
                        app.mode = AppMode::Message(msg);
                    }
                    3 => app.mode = AppMode::Help,
                    _ => app.mode = AppMode::Normal,
                }
            } else {
                app.mode = AppMode::Normal;
            }
        }
        AppMode::Filter => {
            app.current_filter = match app.filter_selected {
                0 => None,
                1 => Some(BranchStatus::Merged),
                2 => Some(BranchStatus::Local),
                3 => Some(BranchStatus::Stashed),
                4 => Some(BranchStatus::Gone),
                5 => Some(BranchStatus::Ahead),
                6 => Some(BranchStatus::Behind),
                7 => Some(BranchStatus::HasUniqueCommits),
                8 => Some(BranchStatus::RemoteTracked),
                9 => Some(BranchStatus::RemoteUntracked),
                _ => None,
            };
            app.refresh_filtered_branches();
            app.selected = 0;
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
    false
}
