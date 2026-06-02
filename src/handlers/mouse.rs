use crate::app::App;
use crate::state::ui::{AppMode, FilePanel, PreviewState, PrimaryMode};
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
            let cur = app.ui.current_mode().clone();
            if cur != AppMode::Normal
                && !matches!(cur, AppMode::CodePreview(_))
                && cur != AppMode::Diff
                && handle_modal_click(app, row, col, term_rows as usize, term_cols as usize, path)
            {
                return;
            }

            if *app.ui.current_mode() == AppMode::Diff {
                let v_start = (term_rows as f32 * 0.05) as usize;
                let v_end = (term_rows as f32 * 0.95) as usize;
                let h_start = (term_cols as f32 * 0.05) as usize;
                let h_end = (term_cols as f32 * 0.95) as usize;

                if row < v_start || row > v_end || col < h_start || col > h_end {
                    app.ui.pop_modal();
                    return;
                }

                let sidebar_width = ((h_end - h_start) as f32 * 0.25) as usize;
                if col < h_start + sidebar_width {
                    app.ui.diff_panel = FilePanel::Directory;
                    let list_v_start = v_start + 1;
                    if row > list_v_start {
                        let idx = row - list_v_start - 1;
                        if idx < app.repo.diff_files.len() {
                            app.ui.diff_file_selected = idx;
                            crate::handlers::keyboard::update_diff_preview(app, path);
                        }
                    }
                } else {
                    app.ui.diff_panel = FilePanel::Preview;
                }
                return;
            }

            if matches!(app.ui.current_mode(), AppMode::CodePreview(_)) {
                let sidebar_width_px =
                    (term_cols as f32 * app.ui.sidebar_width as f32 / 100.0) as usize;
                if col >= sidebar_width_px {
                    app.ui.active_panel = FilePanel::Preview;
                    if let AppMode::CodePreview(state) = app.ui.current_mode_mut() {
                        handle_preview_click(state, row, col, sidebar_width_px);
                    }
                    return;
                } else {
                    app.ui.active_panel = FilePanel::Directory;
                }
            }

            match app.ui.primary_mode {
                PrimaryMode::Branches => handle_list_click(app, row),
                PrimaryMode::Files => handle_directory_click(app, row, path, col),
                PrimaryMode::Commits => {
                    if row > 0 {
                        app.ui.selected_commit_idx = row - 1;
                    }
                }
                PrimaryMode::Stashes => {
                    if row > 0 {
                        app.ui.selected_stash_idx = row - 1;
                        app.load_stash_detail(path);
                    }
                }
            }
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            if matches!(app.ui.current_mode(), AppMode::CodePreview(_)) {
                let sidebar_width_px =
                    (term_cols as f32 * app.ui.sidebar_width as f32 / 100.0) as usize;
                if col >= sidebar_width_px
                    && let AppMode::CodePreview(state) = app.ui.current_mode_mut()
                {
                    handle_preview_drag(state, row, term_rows as usize);
                }
            }
        }
        MouseEventKind::Up(MouseButton::Left) => {
            if matches!(app.ui.current_mode(), AppMode::CodePreview(_)) {
                let (sel_start, sel_end, lines) =
                    if let AppMode::CodePreview(state) = app.ui.current_mode_mut() {
                        (
                            state.selection_start,
                            state.selection_end,
                            state.lines.clone(),
                        )
                    } else {
                        (None, None, vec![])
                    };

                if let (Some(start), Some(end)) = (sel_start, sel_end)
                    && start != end
                {
                    let min_y = start.min(end);
                    let max_y = start.max(end);
                    let mut selected_text = String::new();
                    for i in min_y..=max_y {
                        if let Some(line) = lines.get(i) {
                            selected_text.push_str(line);
                            selected_text.push('\n');
                        }
                    }
                    if let Ok(mut clipboard) = arboard::Clipboard::new() {
                        let _ = clipboard.set_text(selected_text);
                    }
                }
            }
        }
        MouseEventKind::ScrollUp => {
            if *app.ui.current_mode() == AppMode::Diff {
                if app.ui.diff_panel == FilePanel::Directory {
                    app.ui.diff_file_selected = app.ui.diff_file_selected.saturating_sub(1);
                    crate::handlers::keyboard::update_diff_preview(app, path);
                } else if let Some(ref mut state) = app.repo.diff_preview
                    && state.scroll_y > 0
                {
                    state.scroll_y -= 1;
                }
            } else if matches!(app.ui.current_mode(), AppMode::CodePreview(_)) {
                if let AppMode::CodePreview(state) = app.ui.current_mode_mut()
                    && state.scroll_y > 0
                {
                    state.scroll_y -= 1;
                }
            } else {
                app.previous();
            }
        }
        MouseEventKind::ScrollDown => {
            if *app.ui.current_mode() == AppMode::Diff {
                if app.ui.diff_panel == FilePanel::Directory {
                    if app.ui.diff_file_selected + 1 < app.repo.diff_files.len() {
                        app.ui.diff_file_selected += 1;
                        crate::handlers::keyboard::update_diff_preview(app, path);
                    }
                } else if let Some(ref mut state) = app.repo.diff_preview {
                    let line_count = state.lines.len();
                    if state.scroll_y < line_count.saturating_sub(1) {
                        state.scroll_y += 1;
                    }
                }
            } else if matches!(app.ui.current_mode(), AppMode::CodePreview(_)) {
                if let AppMode::CodePreview(state) = app.ui.current_mode_mut() {
                    let line_count = state.lines.len();
                    if state.scroll_y < line_count.saturating_sub(1) {
                        state.scroll_y += 1;
                    }
                }
            } else {
                app.next();
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

fn handle_preview_drag(state: &mut PreviewState, row: usize, term_rows: usize) {
    if row == 0 {
        if state.scroll_y > 0 {
            state.scroll_y -= 1;
            state.cursor_y = state.scroll_y;
            state.selection_end = Some(state.cursor_y);
        }
    } else if row >= term_rows.saturating_sub(3) {
        let line_count = state.lines.len();
        if state.scroll_y < line_count.saturating_sub(1) {
            state.scroll_y += 1;
            state.cursor_y = state.scroll_y + row.saturating_sub(1);
            state.selection_end = Some(state.cursor_y);
        }
    } else {
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
    let cur = app.ui.current_mode().clone();

    // Modal areas based on percentages in screens.rs
    let (v_start_pct, v_size_pct) = match &cur {
        AppMode::Filter => (0.20, 0.60),
        AppMode::Manage | AppMode::Message(_) | AppMode::ConfirmDelete(_) => (0.30, 0.40),
        AppMode::Settings => (0.25, 0.50),
        AppMode::Help => (0.0, 1.0),
        _ => (0.30, 0.40),
    };

    let h_start_pct = match &cur {
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
        app.ui.pop_modal();
        return true;
    }

    // Click outside to close
    if row < min_row || row > max_row || col < min_col || col > max_col {
        app.ui.pop_modal();
        return true;
    }

    // Click inside handling
    let now = Instant::now();
    if cur == AppMode::Manage || cur == AppMode::Filter || cur == AppMode::Settings {
        if row > min_row {
            let option_idx = row - min_row - 1;

            let (is_double, target_option) = if let Some(last_opt) = app.ui.last_click_row
                && last_opt == option_idx
                && now.duration_since(app.ui.last_click_time).as_millis() < 500
            {
                (true, option_idx)
            } else {
                (false, option_idx)
            };

            if cur == AppMode::Manage && target_option < 7 {
                app.ui.manage_selected = target_option;
                if is_double {
                    use ratatui::crossterm::event::{
                        KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers,
                    };
                    let enter_event = KeyEvent {
                        code: KeyCode::Enter,
                        modifiers: KeyModifiers::empty(),
                        kind: KeyEventKind::Press,
                        state: KeyEventState::empty(),
                    };
                    crate::handlers::keyboard::handle_keyboard(app, enter_event, path);
                }
            } else if cur == AppMode::Filter && target_option < 10 {
                app.ui.filter_selected = target_option;
                if is_double {
                    app.ui.current_filter = match target_option {
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
                    app.ui.pop_modal();
                }
            } else if cur == AppMode::Settings && target_option < 9 {
                app.ui.settings_state.selected = target_option;
                if target_option == 8 {
                    crate::utils::config::save_config(&app.config);
                    app.ui.pop_modal();
                } else if target_option == 6 {
                    if is_double || !app.ui.settings_state.editing {
                        app.config.enable_animations = !app.config.enable_animations;
                        crate::utils::config::save_config(&app.config);
                    }
                } else if is_double || !app.ui.settings_state.editing {
                    app.ui.settings_state.editing = true;
                    app.ui.settings_state.input = match option_idx {
                        0 => app.config.ide_command.clone(),
                        1 => app.config.alternative_ide_command.clone(),
                        2 => app.config.ai_provider.clone(),
                        3 => app.config.current_provider().model.clone(),
                        4 => crate::utils::config::deobfuscate(
                            &app.config.current_provider().api_key,
                        ),
                        5 => app.config.current_provider().url.clone(),
                        7 => app.config.default_sidebar_width.to_string(),
                        _ => String::new(),
                    };
                }
            }

            app.ui.last_click_row = Some(option_idx);
            app.ui.last_click_time = now;
        }
    } else if let AppMode::ConfirmDelete(names) = cur {
        if row >= max_row.saturating_sub(3) {
            let mid_col = min_col + (max_col - min_col) / 2;
            if col < mid_col {
                app.ui.snap_animation = Some(SnapAnimation::new(names));
            }
            app.ui.pop_modal();
        }
    } else if matches!(cur, AppMode::Message(_)) {
        app.ui.pop_modal();
    }

    true
}

fn handle_list_click(app: &mut App, row: usize) {
    let list_top = 0;
    if row < list_top + 3 {
        return;
    }

    let relative_row = row - list_top - 3;
    let start = app.ui.list_start_index;
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

    if target_idx < app.repo.file_tree.len() {
        let entry = &app.repo.file_tree[target_idx];
        let chevron_start = 1 + entry.depth * 2;
        let chevron_end = chevron_start + 2;

        if entry.is_dir && col >= chevron_start && col <= chevron_end {
            app.ui.selected_file_idx = target_idx;
            app.toggle_file_dir(path);
            return;
        }

        let now = Instant::now();
        if let Some(last_row) = app.ui.last_click_row
            && last_row == target_idx
            && now.duration_since(app.ui.last_click_time).as_millis() < 500
        {
            app.ui.selected_file_idx = target_idx;

            if app.repo.file_tree[target_idx].is_dir {
                app.toggle_file_dir(path);
            } else {
                let rel_path = app.repo.file_tree[target_idx]
                    .path
                    .to_string_lossy()
                    .to_string();
                if let Some(preview) = app.create_preview_state(path, &rel_path) {
                    app.ui.push_modal(AppMode::CodePreview(preview));
                }
            }
            app.ui.last_click_row = None;
            return;
        }
        app.ui.selected_file_idx = target_idx;
        app.ui.last_click_row = Some(target_idx);
        app.ui.last_click_time = now;
    }
}

fn process_double_click(app: &mut App, target_idx: usize, double_click_mode: AppMode) {
    let now = Instant::now();
    if let Some(last_row) = app.ui.last_click_row
        && last_row == target_idx
        && now.duration_since(app.ui.last_click_time).as_millis() < 500
    {
        app.ui.selected_branch_idx = target_idx;
        if let AppMode::Manage = double_click_mode {
            app.ui.manage_selected = 0;
        }
        app.ui.push_modal(double_click_mode);
        app.ui.last_click_row = None;
        return;
    }

    app.ui.selected_branch_idx = target_idx;
    app.ui.last_click_row = Some(target_idx);
    app.ui.last_click_time = now;
}
