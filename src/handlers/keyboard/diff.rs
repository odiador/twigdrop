use crate::app::App;
use crate::state::ui::{AppMode, FilePanel, PreviewState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn update_diff_preview(app: &mut App, path: &str) {
    if let Some(branch) = app.get_filtered_branches().get(app.ui.selected_branch_idx) {
        let branch_name = branch.name.clone();
        if let Some(file) = app.repo.diff_files.get(app.ui.diff_file_selected) {
            let diff_content = crate::git::get_branch_file_diff(path, &branch_name, file);
            let mut preview = PreviewState {
                file_path: file.clone(),
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
    }
}

pub fn handle_diff_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    match key.code {
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
            false
        }
        KeyCode::Char('j') | KeyCode::Down => {
            if app.ui.diff_panel == FilePanel::Directory {
                if app.ui.diff_file_selected + 1 < app.repo.diff_files.len() {
                    app.ui.diff_file_selected += 1;
                    update_diff_preview(app, path);
                }
            } else if let Some(ref mut state) = app.repo.diff_preview {
                super::files::handle_preview_keyboard(state, KeyCode::Down);
            } else {
                app.ui.info_scroll += 1;
            }
            false
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.ui.diff_panel == FilePanel::Directory {
                app.ui.diff_file_selected = app.ui.diff_file_selected.saturating_sub(1);
                update_diff_preview(app, path);
            } else if let Some(ref mut state) = app.repo.diff_preview {
                super::files::handle_preview_keyboard(state, KeyCode::Up);
            } else {
                app.ui.info_scroll = app.ui.info_scroll.saturating_sub(1);
            }
            false
        }
        KeyCode::Char('F') if app.ui.shift_pressed => {
            let branch = app
                .get_filtered_branches()
                .get(app.ui.selected_branch_idx)
                .copied();
            if let Some(crate::models::MergeStatus::Conflict(conflicts)) =
                branch.map(|b| &b.merge_status)
            {
                for conflict in conflicts {
                    let _ = app
                        .conflict_trigger_tx
                        .try_send((path.to_string(), conflict.clone()));
                }
                app.ui.pop_modal();
                app.ui
                    .push_modal(AppMode::Message("AI Resolving conflicts...".to_string()));
            }
            false
        }
        _ => false,
    }
}
