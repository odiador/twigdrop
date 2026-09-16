use crate::app::App;
use crate::state::ui::{AppMode, PreviewState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_preview_keyboard(state: &mut PreviewState, code: KeyCode) -> bool {
    let line_count = state.lines.len();
    match code {
        KeyCode::Down if state.cursor_y < line_count.saturating_sub(1) => {
            state.cursor_y += 1;
            if state.cursor_y >= state.scroll_y + 20 {
                state.scroll_y += 1;
            }
        }
        KeyCode::Up if state.cursor_y > 0 => {
            state.cursor_y -= 1;
            if state.cursor_y < state.scroll_y {
                state.scroll_y = state.cursor_y;
            }
        }
        _ => {}
    }
    false
}

pub fn handle_files_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    match key.code {
        KeyCode::Char('[') if app.ui.sidebar_width > 10 => {
            app.ui.sidebar_width -= 2;
            app.ui.needs_clear = true;
            false
        }
        KeyCode::Char(']') if app.ui.sidebar_width < 90 => {
            app.ui.sidebar_width += 2;
            app.ui.needs_clear = true;
            false
        }
        KeyCode::Char('e') => {
            let target_path = if app.ui.alt_pressed
                && let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx)
            {
                std::path::PathBuf::from(path).join(&entry.path)
            } else {
                std::path::PathBuf::from(path)
            };
            crate::utils::terminal::open_folder(&target_path);
            false
        }
        KeyCode::Char('v') => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                let target_path = if app.ui.alt_pressed {
                    std::path::PathBuf::from(path).join(&entry.path)
                } else {
                    std::path::PathBuf::from(path)
                };
                crate::utils::terminal::open_ide(&target_path, &app.config.ide_command);
            }
            false
        }
        KeyCode::Char('a') => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                let target_path = if app.ui.alt_pressed {
                    std::path::PathBuf::from(path).join(&entry.path)
                } else {
                    std::path::PathBuf::from(path)
                };
                crate::utils::terminal::open_ide(&target_path, &app.config.alternative_ide_command);
            }
            false
        }
        KeyCode::Char('t') => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                let full_path = std::path::PathBuf::from(path).join(&entry.path);
                let dir = if entry.is_dir {
                    full_path
                } else {
                    full_path
                        .parent()
                        .unwrap_or(&std::path::PathBuf::from(path))
                        .to_path_buf()
                };
                crate::utils::terminal::open_terminal(&dir);
            }
            false
        }
        KeyCode::Char('s') => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx)
                && !entry.is_dir
            {
                let rel_path = entry.path.to_string_lossy().to_string().replace('\\', "/");
                let result = if entry.status == crate::git::files::FileStatus::Staged {
                    crate::actions::commands::unstage_file(path, &rel_path)
                } else {
                    crate::actions::commands::stage_file(path, &rel_path)
                };

                match result {
                    Ok(msg) => {
                        app.load_file_tree(path);
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    Err(e) => {
                        app.ui.push_modal(AppMode::Message(e));
                    }
                }
            }
            false
        }
        KeyCode::Left => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                if entry.is_dir && entry.is_open {
                    app.toggle_file_dir(path);
                } else if entry.depth > 0 {
                    let current_depth = entry.depth;
                    let mut i = app.ui.selected_file_idx;
                    while i > 0 {
                        i -= 1;
                        if app.repo.file_tree[i].depth < current_depth {
                            app.ui.selected_file_idx = i;
                            break;
                        }
                    }
                }
            }
            false
        }
        KeyCode::Right => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx)
                && entry.is_dir
            {
                if !entry.is_open {
                    app.toggle_file_dir(path);
                } else if app.ui.selected_file_idx + 1 < app.repo.file_tree.len() {
                    app.ui.selected_file_idx += 1;
                }
            }
            false
        }
        KeyCode::Enter => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                if entry.is_dir {
                    app.toggle_file_dir(path);
                } else {
                    let rel_path = entry.path.to_string_lossy().to_string();
                    if let Some(preview) = app.create_preview_state(path, &rel_path) {
                        app.ui.push_modal(AppMode::CodePreview(preview));
                    }
                }
            }
            false
        }
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
