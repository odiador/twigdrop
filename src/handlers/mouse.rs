use crate::app::{App, AppMode, PrimaryMode, PreviewState, FilePanel};
use crate::ui::animations::SnapAnimation;
use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::crossterm::terminal;
use std::time::Instant;

pub fn handle_mouse(app: &mut App, event: MouseEvent, path: &str) {
    let (term_cols, term_rows) = terminal::size().unwrap_or((100, 40));
    let row = event.row as usize;
    let col = event.column as usize;

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if app.mode != AppMode::Normal && !matches!(app.mode, AppMode::CodePreview(_))
                && handle_modal_click(app, row, col, term_rows as usize, term_cols as usize, path)
            {
                return;
            }

            if let AppMode::CodePreview(ref mut state) = app.mode {
                let sidebar_width_px = (term_cols as f32 * app.file_state.sidebar_width as f32 / 100.0) as usize;
                if col >= sidebar_width_px {
                    app.file_state.active_panel = FilePanel::Preview;
                    handle_preview_click(state, row, col, sidebar_width_px);
                    return;
                } else {
                    app.file_state.active_panel = FilePanel::Directory;
                }
            }

            match app.primary_mode {
                PrimaryMode::Branches => handle_list_click(app, row),
                PrimaryMode::Files => handle_directory_click(app, row, path, col),
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if let AppMode::CodePreview(ref mut state) = app.mode {
                let sidebar_width_px = (term_cols as f32 * app.file_state.sidebar_width as f32 / 100.0) as usize;
                if col >= sidebar_width_px {
                    handle_preview_drag(state, row);
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if let AppMode::CodePreview(ref mut state) = app.mode {
                if state.scroll_y > 0 {
                    state.scroll_y -= 1;
                }
            } else if app.primary_mode == PrimaryMode::Files && app.file_state.file_scroll > 0 {
                app.file_state.file_scroll -= 1;
            }
        }
        MouseEventKind::ScrollDown => {
            if let AppMode::CodePreview(ref mut state) = app.mode {
                let line_count = state.lines.len();
                if state.scroll_y < line_count.saturating_sub(1) {
                    state.scroll_y += 1;
                }
            } else if app.primary_mode == PrimaryMode::Files {
                app.file_state.file_scroll += 1;
            }
        }
        _ => {}
    }
}

fn handle_preview_click(state: &mut PreviewState, row: usize, _col: usize, _sidebar_width: usize) {
    if row >= 1 {
        let relative_row = row - 1;
        state.cursor_y = state.scroll_y + relative_row;
        state.selection_start = Some(state.cursor_y);
        state.selection_end = Some(state.cursor_y);
    }
}

fn handle_preview_drag(state: &mut PreviewState, row: usize) {
    if row >= 1 {
        let relative_row = row - 1;
        state.selection_end = Some(state.scroll_y + relative_row);
        state.cursor_y = state.scroll_y + relative_row;
    }
}

fn handle_modal_click(
    app: &mut App,
    row: usize,
    col: usize,
    term_rows: usize,
    term_cols: usize,
    path: &str,
) -> bool {
    // Modal areas based on percentages in screens.rs
    let (v_start_pct, v_size_pct) = match app.mode {
        AppMode::Filter => (0.20, 0.60),
        AppMode::Manage | AppMode::Message(_) | AppMode::ConfirmDelete(_) => (0.30, 0.40),
        AppMode::Settings => (0.25, 0.50),
        AppMode::Help => (0.0, 1.0), // Help is full screen
        _ => (0.30, 0.40),
    };

    let h_start_pct = match app.mode {
        AppMode::Filter | AppMode::Manage => 0.30,
        AppMode::Settings => 0.20,
        AppMode::Message(_) | AppMode::Help | AppMode::ConfirmDelete(_) => 0.15,
        _ => 0.30,
    };

    let min_row = (term_rows as f32 * v_start_pct) as usize;
    let max_row = min_row + (term_rows as f32 * v_size_pct) as usize;
    let min_col = (term_cols as f32 * h_start_pct) as usize;
    let max_col = term_cols.saturating_sub(min_col);

    // [X] detection (Top right corner of the modal)
    if row == min_row && col > max_col.saturating_sub(6) && col <= max_col {
        app.mode = AppMode::Normal;
        return true;
    }

    // Click outside to close
    if row < min_row || row > max_row || col < min_col || col > max_col {
        app.mode = AppMode::Normal;
        return true;
    }

    // Click inside handling
    let now = Instant::now();
    if app.mode == AppMode::Manage || app.mode == AppMode::Filter || app.mode == AppMode::Settings {
        if row > min_row {
            let option_idx = row - min_row - 1;
            
            let (is_double, target_option) = if let Some(last_opt) = app.last_click_row 
                && last_opt == option_idx
                && now.duration_since(app.last_click_time).as_millis() < 500 {
                    (true, option_idx)
                } else {
                    (false, option_idx)
                };

            if app.mode == AppMode::Manage && target_option < 7 {
                app.branch_state.manage_selected = target_option;
                if is_double {
                    use ratatui::crossterm::event::{KeyEvent, KeyCode, KeyEventKind, KeyEventState, KeyModifiers};
                    let enter_event = KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::empty(),
                        kind: KeyEventKind::Press,
                        state: KeyEventState::empty(),
                    };
                    crate::handlers::keyboard::handle_keyboard(app, enter_event, path);
                }
            } else if app.mode == AppMode::Filter && target_option < 10 {
                app.branch_state.filter_selected = target_option;
                if is_double {
                    app.branch_state.current_filter = match target_option {
                        1 => Some(crate::models::BranchStatus::Merged),
                        2 => Some(crate::models::BranchStatus::Local),
                        3 => Some(crate::models::BranchStatus::Stashed),
                        4 => Some(crate::models::BranchStatus::Gone),
                        5 => Some(crate::models::BranchStatus::Ahead),
                        6 => Some(crate::models::BranchStatus::Behind),
                        7 => Some(crate::models::BranchStatus::HasUniqueCommits),
                        8 => Some(crate::models::BranchStatus::RemoteTracked),
                        9 => Some(crate::models::BranchStatus::RemoteUntracked),
                        _ => None,
                    };
                    app.refresh_filtered_branches();
                    app.mode = AppMode::Normal;
                }
            } else if app.mode == AppMode::Settings && target_option < 3 {
                app.settings_state.selected = target_option;
                if target_option == 2 {
                    crate::utils::config::save_config(&app.config);
                    app.mode = AppMode::Normal;
                } else if is_double || !app.settings_state.editing {
                    app.settings_state.editing = true;
                    app.settings_state.input = match target_option {
                        0 => app.config.ide_command.clone(),
                        1 => app.config.alternative_ide_command.clone(),
                        _ => String::new(),
                    };
                }
            }
            
            app.last_click_row = Some(option_idx);
            app.last_click_time = now;
        }
    } else if let AppMode::ConfirmDelete(names) = &app.mode {
        let names = names.clone();
        if row >= max_row.saturating_sub(3) {
            let mid_col = min_col + (max_col - min_col) / 2;
            if col < mid_col {
                app.snap_animation = Some(SnapAnimation::new(names));
                app.mode = AppMode::Normal;
            } else {
                app.mode = AppMode::Normal;
            }
        }
    } else if let AppMode::Message(_) = app.mode {
        app.mode = AppMode::Normal;
    }

    true
}

fn handle_list_click(app: &mut App, row: usize) {
    let list_top = 0;
    if row < list_top + 3 {
        return;
    }

    let relative_row = row - list_top - 3;
    let start = app.branch_state.list_start_index;
    let filtered_branches = app.get_filtered_branches();
    let branches_len = filtered_branches.len();

    let target_idx = start + relative_row;

    if target_idx < branches_len {
        process_double_click(app, target_idx, AppMode::Manage);
    }
}

fn handle_directory_click(app: &mut App, row: usize, path: &str, col: usize) {
    let list_top = 0;
    if row < list_top + 1 || col == 0 {
        return;
    }

    let target_idx = row - list_top - 1;

    if target_idx < app.file_state.file_tree.len() {
        let entry = &app.file_state.file_tree[target_idx];
        let chevron_start = 1 + entry.depth * 2;
        let chevron_end = chevron_start + 2;
        
        if entry.is_dir && col >= chevron_start && col <= chevron_end {
            app.file_state.file_selected = target_idx;
            app.toggle_file_dir(path);
            return;
        }

        let now = Instant::now();
        if let Some(last_row) = app.last_click_row
            && last_row == target_idx
            && now.duration_since(app.last_click_time).as_millis() < 500
        {
            app.file_state.file_selected = target_idx;
            
            if app.file_state.file_tree[target_idx].is_dir {
                app.toggle_file_dir(path);
            } else {
                let rel_path = app.file_state.file_tree[target_idx].path.to_string_lossy().to_string();
                if let Some(preview) = app.create_preview_state(path, &rel_path) {
                    app.mode = AppMode::CodePreview(preview);
                }
            }
            app.last_click_row = None;
            return;
        }
        app.file_state.file_selected = target_idx;
        app.last_click_row = Some(target_idx);
        app.last_click_time = now;
    }
}

fn process_double_click(app: &mut App, target_idx: usize, double_click_mode: AppMode) {
    let now = Instant::now();
    if let Some(last_row) = app.last_click_row
        && last_row == target_idx
        && now.duration_since(app.last_click_time).as_millis() < 500
    {
        app.branch_state.selected = target_idx;
        app.mode = double_click_mode;
        if let AppMode::Manage = app.mode {
            app.branch_state.manage_selected = 0;
        }
        app.last_click_row = None;
        return;
    }

    app.branch_state.selected = target_idx;
    app.last_click_row = Some(target_idx);
    app.last_click_time = now;
}
