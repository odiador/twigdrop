use ratatui::crossterm::event::{KeyCode, KeyEvent};
use crate::app::{App, AppMode};
use crate::git;
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
            } else {
                true // Signal to quit
            }
        }
        KeyCode::Char('h') => {
            app.toggle_help();
            false
        }
        KeyCode::Char('m') | KeyCode::Tab | KeyCode::Enter => {
            handle_enter_or_manage(app, path)
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if app.mode == AppMode::Diff {
                app.info_scroll += 1;
            } else if app.mode == AppMode::Manage {
                if app.manage_selected < 3 { app.manage_selected += 1; }
            } else {
                app.next();
            }
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if app.mode == AppMode::Diff {
                app.info_scroll = app.info_scroll.saturating_sub(1);
            } else if app.mode == AppMode::Manage {
                if app.manage_selected > 0 { app.manage_selected -= 1; }
            } else {
                app.previous();
            }
            false
        }
        KeyCode::Char(' ') => {
            if app.mode == AppMode::Normal { app.toggle(); }
            false
        }
        KeyCode::Char('d') => {
            if app.mode == AppMode::Normal {
                let mut output = String::new();
                for (i, b) in app.branches.iter().enumerate() {
                    if app.marked[i] {
                        let msg = delete_branch(path, &b.name);
                        output.push_str(&format!("{}\n", msg));
                    }
                }
                app.branches = git::build_branches(path);
                app.marked = vec![false; app.branches.len()];
                if !output.is_empty() { app.mode = AppMode::Message(output); }
            }
            false
        }
        _ => false
    }
}

fn handle_enter_or_manage(app: &mut App, path: &str) -> bool {
    if app.mode == AppMode::Normal {
        if !app.branches.is_empty() {
            app.mode = AppMode::Manage;
            app.manage_selected = 0;
        }
    } else if app.mode == AppMode::Manage {
        match app.manage_selected {
            0 => { // Checkout
                let b = &app.branches[app.selected];
                let msg = checkout_branch(path, &b.name);
                app.branches = git::build_branches(path);
                app.mode = AppMode::Message(msg);
            }
            1 => { // Diff
                app.mode = AppMode::Diff;
                app.branch_info = git::get_branch_info(path, &app.branches[app.selected].name);
                app.info_scroll = 0;
            }
            2 => app.mode = AppMode::Help,
            _ => app.mode = AppMode::Normal,
        }
    }
    false
}
