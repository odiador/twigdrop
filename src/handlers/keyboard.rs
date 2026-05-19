use crate::actions::{apply_stash, prune_branches};
use crate::app::{App, AppMode, PrimaryMode, PreviewState, FilePanel};
use crate::git;
use crate::ui::animations::SnapAnimation;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    app.alt_pressed = key.modifiers.contains(KeyModifiers::ALT);
    app.shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);

    if let AppMode::Message(_) = app.mode {
        app.mode = AppMode::Normal;
        return false;
    }

    if let AppMode::Settings = app.mode {
        return handle_settings_keyboard(app, key);
    }

    if let AppMode::Search = app.mode {
        return handle_search_keyboard(app, key);
    }

    if let AppMode::Manage = app.mode {
        return handle_manage_keyboard(app, key, path);
    }

    if let AppMode::ConfirmDelete(names) = &app.mode {
        let names = names.clone();
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                app.snap_animation = Some(SnapAnimation::new(names));
                app.mode = AppMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.mode = AppMode::Normal;
            }
            _ => {}
        }
        return false;
    }

    if let AppMode::CodePreview(ref mut state) = app.mode {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            app.needs_clear = true;
        } else if app.file_state.active_panel == FilePanel::Preview {
            return handle_preview_keyboard(state, key.code);
        }
    }
    
    if matches!(app.mode, AppMode::CodePreview(_)) && (key.code == KeyCode::Esc || key.code == KeyCode::Char('q')) {
        app.mode = AppMode::Normal;
        app.file_state.active_panel = FilePanel::Directory;
        return false;
    }

    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            if app.mode != AppMode::Normal {
                if app.mode == AppMode::Help {
                    app.refresh_branches(path);
                }
                app.mode = AppMode::Normal;
                app.needs_clear = true;
                false
            } else if app.branch_state.current_filter.is_some() {
                app.branch_state.current_filter = None;
                app.refresh_filtered_branches();
                app.branch_state.selected = 0;
                false
            } else {
                true 
            }
        }
        KeyCode::Tab => {
            if matches!(app.mode, AppMode::CodePreview(_)) {
                app.file_state.active_panel = match app.file_state.active_panel {
                    FilePanel::Directory => FilePanel::Preview,
                    FilePanel::Preview => FilePanel::Directory,
                };
            }
            false
        }
        KeyCode::Char('/') => {
            if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Branches {
                app.mode = AppMode::Search;
                app.branch_state.search_query.clear();
                app.refresh_filtered_branches();
            }
            false
        }
        KeyCode::Char('[') => {
            if app.primary_mode == PrimaryMode::Files && app.file_state.sidebar_width > 10 {
                app.file_state.sidebar_width -= 2;
                app.needs_clear = true;
            }
            false
        }
        KeyCode::Char(']') => {
            if app.primary_mode == PrimaryMode::Files && app.file_state.sidebar_width < 90 {
                app.file_state.sidebar_width += 2;
                app.needs_clear = true;
            }
            false
        }
        KeyCode::BackTab => {
            app.mode = AppMode::Settings;
            app.settings_state.selected = 0;
            app.settings_state.editing = false;
            false
        }
        KeyCode::Char('h') => {
            app.toggle_help();
            false
        }
        KeyCode::Char('d') => {
            if app.mode == AppMode::Normal || matches!(app.mode, AppMode::CodePreview(_)) {
                app.toggle_primary_mode();
                if app.primary_mode == PrimaryMode::Files {
                    app.load_file_tree(path);
                }
                if matches!(app.mode, AppMode::CodePreview(_)) {
                    app.mode = AppMode::Normal;
                    app.file_state.active_panel = FilePanel::Directory;
                }
            }
            false
        }
        KeyCode::Left => {
            if app.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected) {
                    if entry.is_dir && entry.is_open {
                        app.toggle_file_dir(path);
                    } else if entry.depth > 0 {
                        let current_depth = entry.depth;
                        let mut i = app.file_state.file_selected;
                        while i > 0 {
                            i -= 1;
                            if app.file_state.file_tree[i].depth < current_depth {
                                app.file_state.file_selected = i;
                                break;
                            }
                        }
                    }
            }
            false
        }
        KeyCode::Right => {
            if app.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected)
                && entry.is_dir {
                    if !entry.is_open {
                        app.toggle_file_dir(path);
                    } else if app.file_state.file_selected + 1 < app.file_state.file_tree.len() {
                        app.file_state.file_selected += 1;
                    }
            }
            false
        }
        KeyCode::Char('m') | KeyCode::Enter => {
            if app.primary_mode == PrimaryMode::Files {
                if let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected) {
                    if entry.is_dir {
                        app.toggle_file_dir(path);
                    } else {
                        let rel_path = entry.path.to_string_lossy().to_string();
                        if let Some(preview) = app.create_preview_state(path, &rel_path) {
                            app.mode = AppMode::CodePreview(preview);
                        }
                    }
                }
                false
            } else {
                handle_enter_or_selection(app)
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            match app.mode {
                AppMode::CodePreview(ref mut state) if app.file_state.active_panel == FilePanel::Preview => {
                    handle_preview_keyboard(state, KeyCode::Down);
                    false
                }
                AppMode::Diff => { app.branch_state.info_scroll += 1; false }
                AppMode::StashDetail => {
                    if app.stash_state.stash_selected < app.stash_state.stashes.len().saturating_sub(1) {
                        app.stash_state.stash_selected += 1;
                        app.load_stash_detail(path);
                    }
                    false
                }
                _ => { app.next(); false }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            match app.mode {
                AppMode::CodePreview(ref mut state) if app.file_state.active_panel == FilePanel::Preview => {
                    handle_preview_keyboard(state, KeyCode::Up);
                    false
                }
                AppMode::Diff => { app.branch_state.info_scroll = app.branch_state.info_scroll.saturating_sub(1); false }
                AppMode::StashDetail => {
                    if app.stash_state.stash_selected > 0 {
                        app.stash_state.stash_selected -= 1;
                        app.load_stash_detail(path);
                    }
                    false
                }
                _ => { app.previous(); false }
            }
        }
        KeyCode::Char('g') if let AppMode::CodePreview(ref mut state) = app.mode
            && app.file_state.active_panel == FilePanel::Preview => {
                state.cursor_y = 0;
                state.scroll_y = 0;
                false
        }
        KeyCode::Char('G') if let AppMode::CodePreview(ref mut state) = app.mode
            && app.file_state.active_panel == FilePanel::Preview => {
                let line_count = state.lines.len();
                state.cursor_y = line_count.saturating_sub(1);
                state.scroll_y = state.cursor_y.saturating_sub(10);
                false
        }
        _ => handle_generic_actions(app, key, path)
    }
}

fn handle_generic_actions(app: &mut App, key: KeyEvent, path: &str) -> bool {
    match key.code {
        KeyCode::Char('p') if app.mode == AppMode::Normal => {
            let msg = prune_branches(path, &app.branch_state.branches, &app.current_branch);
            app.refresh_branches(path);
            app.mode = AppMode::Message(msg);
            false
        }
        KeyCode::Char('i') if app.mode == AppMode::Normal => {
            if let Some(branch) = app.get_filtered_branches().get(app.branch_state.selected) {
                let branch_name = branch.name.clone();
                let _ = app.ai_state.ai_trigger_tx.try_send((path.to_string(), branch_name.clone()));
                app.ai_state.ai_analysis = Some("Initializing AI analysis...".to_string());
                app.branch_state.branch_info = git::get_branch_info(path, &branch_name);
                app.branch_state.info_scroll = 0;
                app.mode = AppMode::Diff;
            }
            false
        }
        KeyCode::Char(' ') if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Branches => {
            app.toggle_selection();
            false
        }
        KeyCode::Char('D') if app.shift_pressed && app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Branches => {
            if !app.branch_state.bulk_selected.is_empty() {
                let names: Vec<String> = app.branch_state.bulk_selected.iter().cloned().collect();
                app.snap_animation = Some(SnapAnimation::new(names));
            }
            false
        }
        KeyCode::Char('v') if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Files => {
            if let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected) {
                let target_path = if app.alt_pressed { std::path::PathBuf::from(path).join(&entry.path) } else { std::path::PathBuf::from(path) };
                crate::utils::terminal::open_ide(&target_path, &app.config.ide_command);
            }
            false
        }
        KeyCode::Char('a') if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Files => {
            if let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected) {
                let target_path = if app.alt_pressed { std::path::PathBuf::from(path).join(&entry.path) } else { std::path::PathBuf::from(path) };
                crate::utils::terminal::open_ide(&target_path, &app.config.alternative_ide_command);
            }
            false
        }
        KeyCode::Char('a') if app.mode == AppMode::StashDetail => {
            if let Some(stash) = app.stash_state.stashes.get(app.stash_state.stash_selected) {
                let msg = apply_stash(path, &stash.id);
                app.refresh_branches(path);
                app.mode = AppMode::Message(msg);
            }
            false
        }
        KeyCode::Char('S') if app.shift_pressed && app.mode == AppMode::Normal => {
            app.load_stashes(path);
            app.load_stash_detail(path);
            app.mode = AppMode::StashDetail;
            false
        }
        KeyCode::Char('t') if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Files => {
            if let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected) {
                let full_path = std::path::PathBuf::from(path).join(&entry.path);
                let dir = if entry.is_dir { full_path } else { full_path.parent().unwrap_or(&std::path::PathBuf::from(path)).to_path_buf() };
                crate::utils::terminal::open_terminal(&dir);
            }
            false
        }
        KeyCode::Char('F') if app.shift_pressed && app.mode == AppMode::Diff => {
             let branch = app.get_filtered_branches().get(app.branch_state.selected).copied();
             if let Some(crate::models::MergeStatus::Conflict(conflicts)) = branch.map(|b| &b.merge_status) {
                 for conflict in conflicts { let _ = app.ai_state.conflict_trigger_tx.try_send((path.to_string(), conflict.clone())); }
                 app.mode = AppMode::Message("AI Resolving conflicts...".to_string());
             }
             false
        }
        _ => false
    }
}

fn handle_preview_keyboard(state: &mut PreviewState, code: KeyCode) -> bool {
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

fn handle_search_keyboard(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => { app.mode = AppMode::Normal; }
        KeyCode::Char(c) => { app.branch_state.search_query.push(c); app.refresh_filtered_branches(); }
        KeyCode::Backspace => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                app.branch_state.search_query.clear();
            } else {
                app.branch_state.search_query.pop();
            }
            app.refresh_filtered_branches();
        }
        _ => {}
    }
    false
}

fn handle_settings_keyboard(app: &mut App, key: KeyEvent) -> bool {
    if app.settings_state.editing {
        match key.code {
            KeyCode::Enter => {
                match app.settings_state.selected {
                    0 => app.config.ide_command = app.settings_state.input.clone(),
                    1 => app.config.alternative_ide_command = app.settings_state.input.clone(),
                    _ => {}
                }
                app.settings_state.editing = false;
                crate::utils::config::save_config(&app.config);
            }
            KeyCode::Esc => { app.settings_state.editing = false; }
            KeyCode::Char(c) => { app.settings_state.input.push(c); }
            KeyCode::Backspace => { app.settings_state.input.pop(); }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.settings_state.selected > 0 => { app.settings_state.selected -= 1; }
        KeyCode::Down | KeyCode::Char('j') if app.settings_state.selected < 2 => { app.settings_state.selected += 1; }
        KeyCode::Enter => {
            if app.settings_state.selected == 2 {
                crate::utils::config::save_config(&app.config);
                app.mode = AppMode::Normal;
            } else {
                app.settings_state.editing = true;
                app.settings_state.input = match app.settings_state.selected {
                    0 => app.config.ide_command.clone(),
                    1 => app.config.alternative_ide_command.clone(),
                    _ => String::new(),
                };
            }
        }
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::BackTab => { app.mode = AppMode::Normal; }
        _ => {}
    }
    false
}

fn handle_manage_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    const MANAGE_OPTIONS_COUNT: usize = 7;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.branch_state.manage_selected = app.branch_state.manage_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') if app.branch_state.manage_selected < MANAGE_OPTIONS_COUNT - 1 => {
            app.branch_state.manage_selected += 1;
        }
        KeyCode::Enter => {
            let branch = app.get_filtered_branches().get(app.branch_state.selected).cloned();
            if let Some(b) = branch {
                match app.branch_state.manage_selected {
                    0 => {
                        let msg = crate::git::commands::run_git(path, &["checkout", &b.name]).unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.current_branch = git::get_current_branch(path);
                        app.mode = AppMode::Message(msg);
                    }
                    1 => {
                        let branch_name = b.name.clone();
                        let _ = app.ai_state.ai_trigger_tx.try_send((path.to_string(), branch_name.clone()));
                        app.ai_state.ai_analysis = Some("Initializing AI analysis...".to_string());
                        app.branch_state.branch_info = git::get_branch_info(path, &branch_name);
                        app.branch_state.info_scroll = 0;
                        app.mode = AppMode::Diff;
                    }
                    2 => {
                        let names = vec![b.name.clone()];
                        if b.status.contains(&crate::models::BranchStatus::HasUniqueCommits) {
                            app.mode = AppMode::ConfirmDelete(names);
                        } else {
                            app.snap_animation = Some(SnapAnimation::new(names));
                            app.mode = AppMode::Normal;
                        }
                    }
                    3 => { app.mode = AppMode::Message("Rename branch coming soon!".to_string()); }
                    4 => { 
                        let msg = crate::git::commands::run_git(path, &["stash", "push", "-m", &format!("Stash from Twigdrop: {}", b.name)]).unwrap_or_else(|e| e.to_string());
                        app.mode = AppMode::Message(msg);
                    }
                    5 => app.mode = AppMode::Help,
                    _ => app.mode = AppMode::Normal,
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
    false
}

fn handle_enter_or_selection(app: &mut App) -> bool {
    if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Branches {
        app.mode = AppMode::Manage;
        app.branch_state.manage_selected = 0;
        return false;
    }
    false
}
