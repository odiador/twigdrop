use ratatui::crossterm::event::{MouseEvent, MouseEventKind, MouseButton};
use crate::app::App;

pub fn handle_mouse(app: &mut App, event: MouseEvent) {
    if let MouseEventKind::Down(MouseButton::Left) = event.kind {
        let row = event.row as usize;
        let height = event.column; // Wait, event.row is row, event.column is column. 
        // We don't have the terminal height here easily from the event itself (it's just coordinates).

        // List area starts at row 9 (after 8-row header and 1-row border)
        let list_top = 9;
        if row < list_top { return; }

        let relative_row = row - list_top;
        let start = app.list_start_index;
        let branches_len = app.branches.len();
        
        let show_top_dots = start > 0;
        
        if show_top_dots && relative_row == 0 {
            app.previous();
            return;
        }

        let branch_offset = if show_top_dots { relative_row - 1 } else { relative_row };
        let target_idx = start + branch_offset;

        if target_idx < branches_len {
            // Check if we clicked on a valid branch or a scroll indicator at the bottom
            // Since we don't know the list height here, we can't easily detect the bottom dots.
            // But usually, clicking a branch is what we want.
            // If the user clicks "too low", target_idx will be > branches_len anyway.
            
            app.selected = target_idx;
        } else if target_idx == branches_len && branches_len > 0 {
            // Probably clicked on the bottom dots
            app.next();
        }
    }
}
