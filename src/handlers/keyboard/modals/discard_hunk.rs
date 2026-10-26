use crate::app::App;
use crate::state::ui::{AppMode, ConfirmDiscardHunkState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(
    app: &mut App,
    key: KeyEvent,
    state: ConfirmDiscardHunkState,
    path: &str,
) -> Option<bool> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
            let hunk = crate::git::hunks::DiffHunk {
                hunk_idx: state.hunk_idx,
                old_start: 0,
                old_lines: 0,
                new_start: state.new_start,
                new_lines: state.new_lines,
                header: String::new(),
                raw_patch: state.raw_patch,
                is_staged: false,
            };

            match crate::git::hunks::discard_hunk(path, &hunk) {
                Ok(()) => {
                    app.ui.pop_modal();
                    app.load_file_tree(path);
                    if let Some(new_preview) = app.create_preview_state(path, &state.file_path)
                        && let AppMode::CodePreview(cur) = app.ui.current_mode_mut()
                    {
                        cur.lines = new_preview.lines;
                        cur.highlighted_lines = new_preview.highlighted_lines;
                        cur.line_diffs = new_preview.line_diffs;
                        cur.line_blames = new_preview.line_blames;
                        cur.cursor_y = cur.cursor_y.min(cur.lines.len().saturating_sub(1));
                    }
                    app.ui.notify(
                        crate::state::toast::ToastKind::Success,
                        format!(
                            "Discarded hunk #{} in {}",
                            state.hunk_idx + 1,
                            state.file_path
                        ),
                        None,
                    );
                }
                Err(e) => {
                    app.ui.pop_modal();
                    app.ui.notify(
                        crate::state::toast::ToastKind::Error,
                        format!("Discard failed: {e}"),
                        None,
                    );
                }
            }
            Some(false)
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.ui.pop_modal();
            Some(false)
        }
        _ => Some(false),
    }
}
