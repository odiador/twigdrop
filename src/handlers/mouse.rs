use ratatui::crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use ratatui::crossterm::terminal;
use std::time::Instant;
use crate::app::{App, AppMode};

pub fn handle_mouse(app: &mut App, event: MouseEvent) {
    if let MouseEventKind::Down(MouseButton::Left) = event.kind {
        let (term_cols, term_rows) = terminal::size().unwrap_or((100, 40));
        let row = event.row as usize;
        let col = event.column as usize;

        if app.mode != AppMode::Normal {
            // Modal areas based on percentages in screens.rs
            let (v_start_pct, v_size_pct) = match app.mode {
                AppMode::Filter => (0.20, 0.60),
                AppMode::Manage | AppMode::Message(_) => (0.30, 0.40),
                AppMode::Help => (0.0, 1.0), // Help is full screen
                _ => (0.30, 0.40),
            };

            let h_start_pct = match app.mode {
                AppMode::Filter | AppMode::Manage => 0.30,
                AppMode::Message(_) | AppMode::Help => 0.15,
                _ => 0.30,
            };

            let min_row = (term_rows as f32 * v_start_pct) as usize;
            let max_row = min_row + (term_rows as f32 * v_size_pct) as usize;
            let min_col = (term_cols as f32 * h_start_pct) as usize;
            let max_col = (term_cols as usize).saturating_sub(min_col);

            // [X] detection (Top right corner of the modal)
            if row == min_row && col > max_col.saturating_sub(6) && col <= max_col {
                app.mode = AppMode::Normal;
                return;
            }

            // Click outside to close
            if row < min_row || row > max_row || col < min_col || col > max_col {
                app.mode = AppMode::Normal;
                return;
            }

            // Click inside handling
            if app.mode == AppMode::Manage || app.mode == AppMode::Filter {
                // The list inside the block starts at min_row + 1
                if row > min_row {
                    let option_idx = row - min_row - 1;
                    if app.mode == AppMode::Manage && option_idx < 5 {
                        app.manage_selected = option_idx;
                    } else if app.mode == AppMode::Filter && option_idx < 10 {
                        app.filter_selected = option_idx;
                    }
                }
            } else if let AppMode::Message(_) = app.mode {
                app.mode = AppMode::Normal;
            }
            
            return;
        }

        // List area starts at row 9
        let list_top = 9;
        if row < list_top { return; }

        let relative_row = row - list_top;
        let start = app.list_start_index;
        let filtered_branches = app.get_filtered_branches();
        let branches_len = filtered_branches.len();
        
        let show_top_dots = start > 0;
        
        if show_top_dots && relative_row == 0 {
            app.previous();
            return;
        }

        let branch_offset = if show_top_dots { relative_row - 1 } else { relative_row };
        let target_idx = start + branch_offset;

        if target_idx < branches_len {
            let now = Instant::now();
            if let Some(last_row) = app.last_click_row {
                if last_row == target_idx && now.duration_since(app.last_click_time).as_millis() < 500 {
                    app.selected = target_idx;
                    app.mode = AppMode::Manage;
                    app.manage_selected = 0;
                    app.last_click_row = None;
                    return;
                }
            }
            
            app.selected = target_idx;
            app.last_click_row = Some(target_idx);
            app.last_click_time = now;
        } else if target_idx == branches_len && branches_len > 0 {
            app.next();
        }
    }
}
