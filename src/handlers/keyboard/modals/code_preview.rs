use crate::app::App;
use crate::state::ui::{AppMode, FilePanel, PreviewState};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle(app: &mut App, key: KeyEvent, mut state: PreviewState, path: &str) -> Option<bool> {
    if key.code == KeyCode::Char('Z') {
        app.ui.maximized_panel = !app.ui.maximized_panel;
        let msg = if app.ui.maximized_panel {
            "Panel maximized [Z: restore]"
        } else {
            "Panel restored"
        };
        app.ui
            .notify(crate::state::toast::ToastKind::Info, msg.to_string(), None);
        return Some(false);
    }

    if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
        if app.ui.maximized_panel {
            app.ui.maximized_panel = false;
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "Panel restored".to_string(),
                None,
            );
            return Some(false);
        }
        app.ui.pop_modal();
        app.ui.active_panel = FilePanel::Directory;
        Some(false)
    } else if key.code == KeyCode::Char('b') {
        state.show_blame = !state.show_blame;
        let msg = if state.show_blame {
            "Inline Blame: ON"
        } else {
            "Inline Blame: OFF"
        };
        app.ui
            .notify(crate::state::toast::ToastKind::Info, msg.to_string(), None);
        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
            *s = state;
        }
        Some(false)
    } else if key.code == KeyCode::Char('n') {
        if let Some(target) = state.jump_hunk(true) {
            app.ui.active_panel = FilePanel::Preview;
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                format!("Jumped to change at line {}", target + 1),
                None,
            );
        } else {
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "No changes in file".to_string(),
                None,
            );
        }
        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
            *s = state;
        }
        Some(false)
    } else if key.code == KeyCode::Char('N') {
        if let Some(target) = state.jump_hunk(false) {
            app.ui.active_panel = FilePanel::Preview;
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                format!("Jumped to change at line {}", target + 1),
                None,
            );
        } else {
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "No changes in file".to_string(),
                None,
            );
        }
        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
            *s = state;
        }
        Some(false)
    } else if key.code == KeyCode::Char('B') {
        if let Some(blame) = state.line_blames.get(&state.cursor_y) {
            let commit_hash = &blame.commit;
            let (stats, diff) = crate::git::commands::get_commit_details(path, commit_hash);
            let msg = format!(
                "Commit: {}\nAuthor: {}\nDate:   {}\n\nChanged files & stats:\n{}\n\nDiff excerpt:\n{}",
                commit_hash,
                blame.author,
                blame.date,
                stats.trim(),
                diff.lines().take(30).collect::<Vec<_>>().join("\n")
            );
            app.ui.push_modal(AppMode::Message(msg));
        } else {
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "No blame info for current line".to_string(),
                None,
            );
        }
        Some(false)
    } else if (key.code == KeyCode::Char(' ') || key.code == KeyCode::Char('s'))
        && app.ui.active_panel == FilePanel::Preview
    {
        let hunks = crate::git::hunks::get_file_hunks(path, &state.file_path);
        if hunks.is_empty() {
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "No diff hunks in this file".to_string(),
                None,
            );
            return Some(false);
        }

        if let Some(hunk) = crate::git::hunks::find_hunk_at_line(&hunks, state.cursor_y + 1) {
            let hunk_label = format!(
                "#{} (lines {}-{})",
                hunk.hunk_idx + 1,
                hunk.new_start,
                hunk.new_start + hunk.new_lines.max(1).saturating_sub(1)
            );
            let result = if hunk.is_staged {
                crate::git::hunks::unstage_hunk(path, hunk)
                    .map(|()| format!("✓ Unstaged hunk {hunk_label}"))
            } else {
                crate::git::hunks::stage_hunk(path, hunk)
                    .map(|()| format!("✓ Staged hunk {hunk_label}"))
            };

            match result {
                Ok(msg) => {
                    app.load_file_tree(path);
                    if let Some(new_preview) = app.create_preview_state(path, &state.file_path) {
                        state.lines = new_preview.lines;
                        state.highlighted_lines = new_preview.highlighted_lines;
                        state.line_diffs = new_preview.line_diffs;
                        state.line_blames = new_preview.line_blames;
                        state.cursor_y = state.cursor_y.min(state.lines.len().saturating_sub(1));
                        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
                            *s = state;
                        }
                    }
                    app.ui
                        .notify(crate::state::toast::ToastKind::Success, msg, None);
                }
                Err(e) => {
                    app.ui.notify(
                        crate::state::toast::ToastKind::Error,
                        format!("Hunk action failed: {e}"),
                        None,
                    );
                }
            }
        }
        Some(false)
    } else if key.code == KeyCode::Char('d') && app.ui.active_panel == FilePanel::Preview {
        let hunks = crate::git::hunks::get_file_hunks(path, &state.file_path);
        if hunks.is_empty() {
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "No diff hunks in this file".to_string(),
                None,
            );
            return Some(false);
        }

        if let Some(hunk) = crate::git::hunks::find_hunk_at_line(&hunks, state.cursor_y + 1) {
            if hunk.is_staged {
                app.ui.notify(
                    crate::state::toast::ToastKind::Info,
                    "Hunk is staged; press [Space] to unstage before discarding".to_string(),
                    None,
                );
            } else {
                app.ui.push_modal(AppMode::ConfirmDiscardHunk(
                    crate::state::ui::ConfirmDiscardHunkState {
                        file_path: state.file_path.clone(),
                        hunk_idx: hunk.hunk_idx,
                        new_start: hunk.new_start,
                        new_lines: hunk.new_lines,
                        raw_patch: hunk.raw_patch.clone(),
                    },
                ));
            }
        }
        Some(false)
    } else if key.code == KeyCode::Tab
        || key.code == KeyCode::BackTab
        || key.modifiers.contains(KeyModifiers::ALT)
        || key.modifiers.contains(KeyModifiers::CONTROL)
    {
        None
    } else if app.ui.active_panel == FilePanel::Preview {
        let res = super::super::files::handle_preview_keyboard(&mut state, key.code);
        if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
            *s = state;
        }
        Some(res)
    } else {
        None
    }
}
