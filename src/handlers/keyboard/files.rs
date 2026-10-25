use crate::app::App;
use crate::state::ui::{AppMode, PreviewState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_preview_keyboard(state: &mut PreviewState, code: KeyCode) -> bool {
    let line_count = state.lines.len();
    match code {
        KeyCode::Down | KeyCode::Char('j') if state.cursor_y < line_count.saturating_sub(1) => {
            state.cursor_y += 1;
            if state.cursor_y >= state.scroll_y + 20 {
                state.scroll_y += 1;
            }
        }
        KeyCode::Up | KeyCode::Char('k') if state.cursor_y > 0 => {
            state.cursor_y -= 1;
            if state.cursor_y < state.scroll_y {
                state.scroll_y = state.cursor_y;
            }
        }
        KeyCode::PageDown => {
            state.cursor_y = (state.cursor_y + 20).min(line_count.saturating_sub(1));
            state.scroll_y = (state.scroll_y + 20).min(line_count.saturating_sub(1));
        }
        KeyCode::PageUp => {
            state.cursor_y = state.cursor_y.saturating_sub(20);
            state.scroll_y = state.scroll_y.saturating_sub(20);
        }
        KeyCode::Home => {
            state.cursor_y = 0;
            state.scroll_y = 0;
        }
        KeyCode::End => {
            state.cursor_y = line_count.saturating_sub(1);
            state.scroll_y = line_count.saturating_sub(20);
        }
        _ => {}
    }
    false
}

pub fn handle_files_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    if app.ui.files_search_active {
        match key.code {
            KeyCode::Esc => {
                app.ui.files_search_active = false;
            }
            KeyCode::Enter => {
                app.ui.files_search_active = false;
            }
            KeyCode::Backspace => {
                app.ui.files_search_query.pop();
                app.refresh_filtered_files();
            }
            KeyCode::Char(c) => {
                app.ui.files_search_query.push(c);
                app.refresh_filtered_files();
            }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Char('/') => {
            app.ui.files_search_active = true;
            false
        }
        KeyCode::Char('.') | KeyCode::Char('H') => {
            app.ui.show_hidden_files = !app.ui.show_hidden_files;
            app.load_file_tree(path);
            let label = if app.ui.show_hidden_files {
                "Hidden files: Shown"
            } else {
                "Hidden files: Hidden"
            };
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                label.to_string(),
                None,
            );
            false
        }
        KeyCode::Char('c') | KeyCode::Char('m') => {
            app.ui.files_changed_only = !app.ui.files_changed_only;
            app.load_file_tree(path);
            let label = if app.ui.files_changed_only {
                "Changed files only"
            } else {
                "All files"
            };
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                format!("Filter: {label}"),
                None,
            );
            false
        }
        KeyCode::Esc => {
            if !app.ui.files_search_query.is_empty() {
                app.ui.files_search_query.clear();
                app.refresh_filtered_files();
            } else if app.ui.files_changed_only {
                app.ui.files_changed_only = false;
                app.load_file_tree(path);
                app.ui.notify(
                    crate::state::toast::ToastKind::Info,
                    "Filter: All files".to_string(),
                    None,
                );
            } else {
                app.ui.primary_mode = crate::state::ui::PrimaryMode::Branches;
            }
            false
        }
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
            let alt = key
                .modifiers
                .contains(ratatui::crossterm::event::KeyModifiers::ALT);
            let target_path = if alt && let Some(entry) = app.selected_file() {
                std::path::PathBuf::from(path).join(&entry.path)
            } else {
                std::path::PathBuf::from(path)
            };
            crate::utils::terminal::open_folder(&target_path);
            false
        }
        KeyCode::Char('v') => {
            if let Some(entry) = app.selected_file() {
                let alt = key
                    .modifiers
                    .contains(ratatui::crossterm::event::KeyModifiers::ALT);
                let target_path = if alt {
                    std::path::PathBuf::from(path).join(&entry.path)
                } else {
                    std::path::PathBuf::from(path)
                };
                crate::utils::terminal::open_ide(&target_path, &app.config.ide_command);
            }
            false
        }
        KeyCode::Char('a') => {
            if let Some(entry) = app.selected_file() {
                let alt = key
                    .modifiers
                    .contains(ratatui::crossterm::event::KeyModifiers::ALT);
                let target_path = if alt {
                    std::path::PathBuf::from(path).join(&entry.path)
                } else {
                    std::path::PathBuf::from(path)
                };
                crate::utils::terminal::open_ide(&target_path, &app.config.alternative_ide_command);
            }
            false
        }
        KeyCode::Char('t') => {
            if let Some(entry) = app.selected_file() {
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
        KeyCode::Char(' ') | KeyCode::Char('s') => {
            if let Some(entry) = app.selected_file()
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
                        app.ui
                            .notify(crate::state::toast::ToastKind::Success, msg, None);
                    }
                    Err(e) => {
                        app.ui
                            .notify(crate::state::toast::ToastKind::Error, e, None);
                    }
                }
            }
            false
        }
        KeyCode::Left => {
            if let Some(entry) = app.selected_file().cloned() {
                if entry.is_dir && entry.is_open {
                    app.toggle_file_dir(path);
                } else if entry.depth > 0 {
                    let current_depth = entry.depth;
                    let mut i = app.ui.selected_file_idx;
                    while i > 0 {
                        i -= 1;
                        if let Some(&tree_idx) = app.ui.filtered_file_indices.get(i)
                            && app.repo.file_tree[tree_idx].depth < current_depth
                        {
                            app.ui.selected_file_idx = i;
                            break;
                        }
                    }
                }
            }
            false
        }
        KeyCode::Right => {
            if let Some(entry) = app.selected_file().cloned()
                && entry.is_dir
            {
                if !entry.is_open {
                    app.toggle_file_dir(path);
                } else if app.ui.selected_file_idx + 1 < app.ui.filtered_file_indices.len() {
                    app.ui.selected_file_idx += 1;
                }
            }
            false
        }
        KeyCode::Enter => {
            if let Some(entry) = app.selected_file().cloned() {
                if entry.is_dir {
                    app.toggle_file_dir(path);
                } else {
                    let rel_path = entry.path.to_string_lossy().to_string();
                    if let Some(preview) = app.create_preview_state(path, &rel_path) {
                        if matches!(app.ui.current_mode(), AppMode::CodePreview(_)) {
                            app.ui.pop_modal();
                        }
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
