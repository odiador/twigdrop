use crate::app::App;
use crate::git;
use crate::state::ui::{AppMode, PreviewState};
use crate::ui::animations::SnapAnimation;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent, path: &str) -> Option<bool> {
    Some(handle_manage_keyboard(app, key, path))
}

pub fn handle_manage_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    const MANAGE_OPTIONS_COUNT: usize = 11;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.ui.manage_selected = app.ui.manage_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') if app.ui.manage_selected < MANAGE_OPTIONS_COUNT - 1 => {
            app.ui.manage_selected += 1;
        }
        KeyCode::Enter => {
            let branch = app
                .get_filtered_branches()
                .get(app.ui.selected_branch_idx)
                .cloned();
            if let Some(b) = branch {
                match app.ui.manage_selected {
                    0 => {
                        let msg = crate::actions::commands::checkout_branch(path, &b.name)
                            .unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.repo.current_branch = git::get_current_branch(path);
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    1 => {
                        let branch_name = b.name.clone();
                        let _ = app.ai_trigger_tx.try_send((
                            "analyze".to_string(),
                            path.to_string(),
                            branch_name.clone(),
                        ));
                        app.ai_state.ai_analysis = Some("Initializing AI analysis...".to_string());
                        app.mark_ai_dispatched();
                        app.repo.branch_info = git::get_branch_info(path, &branch_name);
                        app.ui.info_scroll = 0;

                        app.repo.diff_files = git::get_branch_diff_files(path, &branch_name);
                        app.ui.diff_file_selected = 0;
                        app.repo.diff_preview = None;
                        if !app.repo.diff_files.is_empty() {
                            let first_file = app.repo.diff_files[0].clone();
                            let diff_content =
                                git::get_branch_file_diff(path, &branch_name, &first_file);
                            let mut preview = PreviewState {
                                file_path: first_file,
                                lines: diff_content.lines().map(|s| s.to_string()).collect(),
                                highlighted_lines: vec![],
                                cursor_y: 0,
                                scroll_y: 0,
                                selection_start: None,
                                selection_end: None,
                                line_diffs: std::collections::HashMap::new(),
                                line_blames: std::collections::HashMap::new(),
                                show_blame: false,
                            };
                            app.update_diff_highlighting(&mut preview);
                            app.repo.diff_preview = Some(preview);
                        }

                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Diff);
                    }
                    2 => {
                        if app.config.is_protected(&b.name) {
                            let clean = b.name.trim().trim_start_matches('*').trim().to_string();
                            app.ui.pop_modal();
                            app.ui.push_modal(AppMode::Message(format!(
                                "Cannot delete protected branch: {clean}"
                            )));
                            return false;
                        }
                        let names = vec![b.name.clone()];
                        if b.status
                            .contains(&crate::models::BranchStatus::HasUniqueCommits)
                        {
                            app.ui.pop_modal();
                            app.ui.push_modal(AppMode::ConfirmDelete(names));
                        } else {
                            app.ui.snap_animation = Some(SnapAnimation::new(names));
                            app.ui.pop_modal();
                        }
                    }
                    3 => {
                        let old = b.name.clone();
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::RenameBranch(old.clone(), old));
                        app.ui.manage_selected = 0;
                    }
                    4 => {
                        let msg = crate::actions::commands::push_branch(path, &b.name, false)
                            .unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    5 => {
                        let msg = crate::actions::commands::push_branch(path, &b.name, true)
                            .unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    6 => {
                        let msg = crate::actions::commands::merge_branch(path, &b.name)
                            .unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.repo.current_branch = git::get_current_branch(path);
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    7 => {
                        let msg = match arboard::Clipboard::new()
                            .and_then(|mut cb| cb.set_text(b.name.clone()))
                        {
                            Ok(()) => format!("Copied '{}' to clipboard", b.name),
                            Err(e) => format!("Clipboard unavailable: {e}"),
                        };
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    8 => {
                        let msg = crate::actions::commands::push_stash(
                            path,
                            &format!("Stash from Twigdrop: {}", b.name),
                        )
                        .unwrap_or_else(|e| e.to_string());
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    9 => {
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Help);
                    }
                    _ => {
                        app.ui.pop_modal();
                    }
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
