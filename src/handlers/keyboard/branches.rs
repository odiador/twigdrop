use crate::actions::prune_branches;
use crate::app::App;
use crate::git;
use crate::state::ui::{AppMode, PreviewState};
use crate::ui::animations::SnapAnimation;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_enter_or_selection(app: &mut App, path: &str) -> bool {
    if *app.ui.current_mode() == AppMode::Normal {
        if let Some(branch) = app
            .get_filtered_branches()
            .get(app.ui.selected_branch_idx)
            .cloned()
            && branch.name.starts_with('*')
        {
            let branch_name = branch.name.clone();
            app.repo.branch_info = git::get_branch_info(path, &branch_name);
            app.ui.info_scroll = 0;

            app.repo.diff_files = git::get_branch_diff_files(path, &branch_name);
            app.ui.diff_file_selected = 0;
            app.repo.diff_preview = None;
            if !app.repo.diff_files.is_empty() {
                let first_file = app.repo.diff_files[0].clone();
                let diff_content = git::get_branch_file_diff(path, &branch_name, &first_file);
                let mut preview = PreviewState {
                    file_path: first_file,
                    lines: diff_content.lines().map(|s| s.to_string()).collect(),
                    highlighted_lines: vec![],
                    cursor_y: 0,
                    scroll_y: 0,
                    selection_start: None,
                    selection_end: None,
                    line_diffs: std::collections::HashMap::new(),
                };
                app.update_diff_highlighting(&mut preview);
                app.repo.diff_preview = Some(preview);
            }

            app.ui.push_modal(AppMode::Diff);
            return false;
        }

        app.ui.push_modal(AppMode::Manage);
        app.ui.manage_selected = 0;
        return false;
    }
    false
}

pub fn handle_branches_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    match key.code {
        KeyCode::Char('p') => {
            let msg = prune_branches(path, &app.repo.branches, &app.repo.current_branch);
            app.refresh_branches(path);
            app.ui.push_modal(AppMode::Message(msg));
            false
        }
        KeyCode::Char('i') => {
            if let Some(branch) = app.get_filtered_branches().get(app.ui.selected_branch_idx) {
                if branch.name.starts_with('*') {
                    return false;
                }
                let branch_name = branch.name.clone();
                let _ = app.ai_trigger_tx.try_send((
                    "analyze".to_string(),
                    path.to_string(),
                    branch_name.clone(),
                ));
                app.ai_state.ai_analysis = Some("Initializing AI analysis...".to_string());
                app.repo.branch_info = git::get_branch_info(path, &branch_name);
                app.ui.info_scroll = 0;

                app.repo.diff_files = git::get_branch_diff_files(path, &branch_name);
                app.ui.diff_file_selected = 0;
                app.repo.diff_preview = None;
                if !app.repo.diff_files.is_empty() {
                    let first_file = app.repo.diff_files[0].clone();
                    let diff_content = git::get_branch_file_diff(path, &branch_name, &first_file);
                    let mut preview = PreviewState {
                        file_path: first_file,
                        lines: diff_content.lines().map(|s| s.to_string()).collect(),
                        highlighted_lines: vec![],
                        cursor_y: 0,
                        scroll_y: 0,
                        selection_start: None,
                        selection_end: None,
                        line_diffs: std::collections::HashMap::new(),
                    };
                    app.update_diff_highlighting(&mut preview);
                    app.repo.diff_preview = Some(preview);
                }

                app.ui.push_modal(AppMode::Diff);
            }
            false
        }
        KeyCode::Char(' ') => {
            if let Some(branch) = app.get_filtered_branches().get(app.ui.selected_branch_idx)
                && !branch.name.starts_with('*')
            {
                app.toggle_selection();
            }
            false
        }
        KeyCode::Char('D') if app.ui.shift_pressed => {
            if !app.ui.bulk_selected.is_empty() {
                let names: Vec<String> = app.ui.bulk_selected.iter().cloned().collect();
                app.ui.snap_animation = Some(SnapAnimation::new(names));
            }
            false
        }
        KeyCode::Char('f') => {
            app.ui.push_modal(AppMode::Filter);
            app.ui.filter_selected = 0;
            false
        }
        KeyCode::Char('/') => {
            app.ui.push_modal(AppMode::Search);
            app.ui.search_query.clear();
            app.refresh_filtered_branches();
            false
        }
        KeyCode::Char('m') | KeyCode::Enter => handle_enter_or_selection(app, path),
        KeyCode::Char('j') | KeyCode::Down => {
            app.next(path);
            false
        }
        KeyCode::Char('k') | KeyCode::Up => {
            app.previous(path);
            false
        }
        _ => false,
    }
}
