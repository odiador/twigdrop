use crate::app::{App, AppMode, PrimaryMode, PreviewState};
use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::crossterm::terminal;
use std::time::Instant;

pub fn handle_mouse(app: &mut App, event: MouseEvent, path: &str) {
    let (term_cols, term_rows) = terminal::size().unwrap_or((100, 40));
    let row = event.row as usize;
    let col = event.column as usize;

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if app.mode != AppMode::Normal
                && handle_modal_click(app, row, col, term_rows as usize, term_cols as usize)
            {
                return;
            }

            if let AppMode::CodePreview(ref mut state) = app.mode {
                let sidebar_width = (term_cols as f32 * app.file_state.sidebar_width as f32 / 100.0) as usize;
                if col >= sidebar_width {
                    handle_preview_click(state, row, col, sidebar_width);
                    return;
                }
            }

            match app.primary_mode {
                PrimaryMode::Branches => handle_list_click(app, row),
                PrimaryMode::Files => handle_directory_click(app, row, path),
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if let AppMode::CodePreview(ref mut state) = app.mode {
                let sidebar_width = (term_cols as f32 * app.file_state.sidebar_width as f32 / 100.0) as usize;
                if col >= sidebar_width {
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
                state.scroll_y += 1;
            } else if app.primary_mode == PrimaryMode::Files {
                app.file_state.file_scroll += 1;
            }
        }
        _ => {}
    }
}

fn handle_preview_click(state: &mut PreviewState, row: usize, _col: usize, _sidebar_width: usize) {
    // block starts at y=0, header at y=1, content at y=2
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
) -> bool {
    // Modal areas based on percentages in screens.rs
    let (v_start_pct, v_size_pct) = match app.mode {
        AppMode::Filter => (0.20, 0.60),
        AppMode::Manage | AppMode::Message(_) => (0.30, 0.40),
        AppMode::Settings => (0.25, 0.50),
        AppMode::Help => (0.0, 1.0), // Help is full screen
        _ => (0.30, 0.40),
    };

    let h_start_pct = match app.mode {
        AppMode::Filter | AppMode::Manage => 0.30,
        AppMode::Settings => 0.20,
        AppMode::Message(_) | AppMode::Help => 0.15,
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
    if app.mode == AppMode::Manage || app.mode == AppMode::Filter || app.mode == AppMode::Settings {
        if row > min_row {
            let option_idx = row - min_row - 1;
            if app.mode == AppMode::Manage && option_idx < 5 {
                app.branch_state.manage_selected = option_idx;
            } else if app.mode == AppMode::Filter && option_idx < 10 {
                app.branch_state.filter_selected = option_idx;
            } else if app.mode == AppMode::Settings && option_idx < 3 {
                app.settings_state.selected = option_idx;
                if option_idx == 2 {
                    // Click on Save and Exit
                    crate::utils::config::save_config(&app.config);
                    app.mode = AppMode::Normal;
                } else {
                    app.settings_state.editing = true;
                    app.settings_state.input = match option_idx {
                        0 => app.config.ide_command.clone(),
                        1 => app.config.alternative_ide_command.clone(),
                        _ => String::new(),
                    };
                }
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

fn handle_directory_click(app: &mut App, row: usize, path: &str) {
    // List inside Block starts at y+1
    let list_top = 0;
    if row < list_top + 1 {
        return;
    }

    let target_idx = row - list_top - 1;

    if target_idx < app.file_state.file_tree.len() {
        let now = Instant::now();
        if let Some(last_row) = app.last_click_row
            && last_row == target_idx
            && now.duration_since(app.last_click_time).as_millis() < 500
        {
            app.file_state.file_selected = target_idx;
            app.toggle_file_dir(path);
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
