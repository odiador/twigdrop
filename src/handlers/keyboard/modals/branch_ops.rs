use crate::app::App;
use crate::git;
use crate::state::ui::AppMode;
use crate::ui::animations::SnapAnimation;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_confirm_delete(app: &mut App, key: KeyEvent, names: Vec<String>) -> Option<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            app.ui.snap_animation = Some(SnapAnimation::new(names));
            app.ui.pop_modal();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    Some(false)
}

pub fn handle_create_branch(
    app: &mut App,
    key: KeyEvent,
    path: &str,
    input: String,
) -> Option<bool> {
    match key.code {
        KeyCode::Enter => {
            if !input.is_empty() {
                match crate::git::commands::run_git(path, &["checkout", "-b", &input]) {
                    Ok(_) => {
                        app.refresh_branches(path);
                        app.repo.current_branch = git::get_current_branch(path);
                        app.ui.pop_modal();
                        app.ui
                            .push_modal(AppMode::Message(format!("Created {input}")));
                    }
                    Err(e) => {
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(format!("Error: {e}")));
                    }
                }
            } else {
                app.ui.pop_modal();
            }
        }
        KeyCode::Esc => {
            app.ui.pop_modal();
        }
        KeyCode::Char(c) => {
            if let AppMode::CreateBranch(inp) = app.ui.current_mode_mut() {
                inp.push(c);
            }
        }
        KeyCode::Backspace => {
            if let AppMode::CreateBranch(inp) = app.ui.current_mode_mut() {
                inp.pop();
            }
        }
        _ => {}
    }
    Some(false)
}

pub fn handle_rename_branch(
    app: &mut App,
    key: KeyEvent,
    path: &str,
    old: String,
    input: String,
) -> Option<bool> {
    match key.code {
        KeyCode::Enter => {
            if !input.is_empty() && input != old {
                match crate::git::commands::run_git(path, &["branch", "-m", &old, &input]) {
                    Ok(_) => {
                        app.refresh_branches(path);
                        app.repo.current_branch = git::get_current_branch(path);
                        app.ui.pop_modal();
                        app.ui
                            .push_modal(AppMode::Message(format!("Renamed {old} to {input}")));
                    }
                    Err(e) => {
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(format!("Error: {e}")));
                    }
                }
            } else {
                app.ui.pop_modal();
            }
        }
        KeyCode::Esc => {
            app.ui.pop_modal();
        }
        KeyCode::Char(c) => {
            if let AppMode::RenameBranch(_, inp) = app.ui.current_mode_mut() {
                inp.push(c);
            }
        }
        KeyCode::Backspace => {
            if let AppMode::RenameBranch(_, inp) = app.ui.current_mode_mut() {
                inp.pop();
            }
        }
        _ => {}
    }
    Some(false)
}
