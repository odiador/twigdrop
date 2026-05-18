use crate::actions::{apply_stash, checkout_branch, prune_branches};
use crate::app::{App, AppMode, PrimaryMode, PreviewState};
use crate::git;
use crate::models::BranchStatus;
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

    if let AppMode::CodePreview(ref mut state) = app.mode {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            app.needs_clear = true;
        } else {
            return handle_preview_keyboard(state, key);
        }
    }
    
    if let AppMode::CodePreview(_) = app.mode
         && (key.code == KeyCode::Esc || key.code == KeyCode::Char('q')) {
            app.mode = AppMode::Normal;
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
                true // Signal to quit
            }
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
            }
            false
        }
        KeyCode::Char(']') => {
            if app.primary_mode == PrimaryMode::Files && app.file_state.sidebar_width < 90 {
                app.file_state.sidebar_width += 2;
            }
            false
        }
        KeyCode::BackTab => {
            // Shift+Tab toggles settings
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
            if app.mode == AppMode::Normal {
                app.toggle_primary_mode();
                if app.primary_mode == PrimaryMode::Files {
                    app.load_file_tree(path);
                }
            }
            false
        }
        KeyCode::Char('e') => {
            if app.mode == AppMode::Normal {
                let target_path = if app.alt_pressed
                    && app.primary_mode == PrimaryMode::Files
                    && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected)
                {
                    std::path::PathBuf::from(path).join(&entry.path)
                } else {
                    std::path::PathBuf::from(path)
                };
                crate::utils::terminal::open_folder(&target_path);
            }
            false
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.show_terminal = !app.show_terminal;
            false
        }
        KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            let target_path = if app.alt_pressed
                && app.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected)
            {
                std::path::PathBuf::from(path).join(&entry.path)
            } else {
                std::path::PathBuf::from(path)
            };
            crate::utils::terminal::open_ide(&target_path, &app.config.ide_command);
            false
        }
        KeyCode::Char('S') if app.shift_pressed => {
            if app.mode == AppMode::Normal {
                app.load_stashes(path);
                app.load_stash_detail(path);
                app.mode = AppMode::StashDetail;
            }
            false
        }
        KeyCode::Char('F') if app.shift_pressed => {
            if app.mode == AppMode::Diff {
                // Trigger AI conflict resolution if conflicts exist
                let branch = app
                    .get_filtered_branches()
                    .get(app.branch_state.selected)
                    .copied();
                if let Some(crate::models::MergeStatus::Conflict(conflicts)) =
                    branch.map(|b| &b.merge_status)
                {
                    for conflict in conflicts {
                        let _ = app
                            .ai_state
                            .conflict_trigger_tx
                            .try_send((path.to_string(), conflict.clone()));
                    }
                    app.mode = AppMode::Message("AI Resolving conflicts...".to_string());
                }
            }
            false
        }
        KeyCode::Char('p') => {
            if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Branches {
                let msg = prune_branches(path, &app.branch_state.branches, &app.current_branch);
                app.refresh_branches(path);
                app.mode = AppMode::Message(msg);
            }
            false
        }
        KeyCode::Char('i') => {
            if app.mode == AppMode::Normal
                && app.primary_mode == PrimaryMode::Branches
                && let Some(branch) = app.get_filtered_branches().get(app.branch_state.selected)
            {
                let branch_name = branch.name.clone();
                let path_clone = path.to_string();
                let _ = app
                    .ai_state
                    .ai_trigger_tx
                    .try_send((path_clone, branch_name.clone()));
                app.ai_state.ai_analysis = Some("Initializing AI analysis...".to_string());

                // Switch to Diff mode to show the analysis panel
                let info = git::get_branch_info(path, &branch_name);
                app.branch_state.branch_info = info;
                app.branch_state.info_scroll = 0;
                app.mode = AppMode::Diff;
            }
            false
        }
        KeyCode::Char(' ') => {
            if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Branches {
                app.toggle_selection();
            }
            false
        }
        KeyCode::Char('D') if app.shift_pressed => {
            if app.mode == AppMode::Normal
                && app.primary_mode == PrimaryMode::Branches
                && !app.branch_state.bulk_selected.is_empty()
            {
                let names: Vec<String> = app.branch_state.bulk_selected.iter().cloned().collect();
                app.snap_animation = Some(SnapAnimation::new(names));
            }
            false
        }
        KeyCode::Right => {
            if app.mode == AppMode::Normal
                && app.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected)
                && entry.is_dir
            {
                if !entry.is_open {
                    app.toggle_file_dir(path);
                } else if app.file_state.file_selected + 1 < app.file_state.file_tree.len() {
                    let next_entry = &app.file_state.file_tree[app.file_state.file_selected + 1];
                    if next_entry.depth > entry.depth {
                        app.file_state.file_selected += 1;
                    }
                }
            }
            false
        }
        KeyCode::Left => {
            if app.mode == AppMode::Normal
                && app.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected)
            {
                if entry.is_dir && entry.is_open {
                    app.toggle_file_dir(path);
                } else if entry.depth > 0 {
                    let current_depth = entry.depth;
                    for i in (0..app.file_state.file_selected).rev() {
                        if app.file_state.file_tree[i].depth < current_depth {
                            app.file_state.file_selected = i;
                            break;
                        }
                    }
                }
            }
            false
        }
        KeyCode::Char('f') => {
            if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Branches {
                app.mode = AppMode::Filter;
                app.branch_state.filter_selected = 0;
            }
            false
        }
        KeyCode::Char('m') | KeyCode::Tab | KeyCode::Enter => {
            if app.mode == AppMode::Normal && app.primary_mode == PrimaryMode::Files {
                if let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected) {
                    if entry.is_dir {
                        app.toggle_file_dir(path);
                    } else {
                        // Open code preview
                        let full_path = std::path::Path::new(path).join(&entry.path);
                        if let Ok(content) = std::fs::read_to_string(&full_path) {
                            let file_path_str = entry.path.to_string_lossy().to_string();
                            let line_diffs = crate::git::get_line_diffs(path, &file_path_str);
                            app.mode = AppMode::CodePreview(PreviewState {
                                file_path: file_path_str,
                                content,
                                cursor_y: 0,
                                scroll_y: 0,
                                selection_start: None,
                                selection_end: None,
                                line_diffs,
                            });
                        }
                    }
                }
                false
            } else if app.mode == AppMode::StashDetail
                && let Some(stash) = app.stash_state.stashes.get(app.stash_state.stash_selected)
            {
                let msg = apply_stash(path, &stash.id);
                app.refresh_branches(path);
                app.mode = AppMode::Message(msg);
                false
            } else {
                handle_enter_or_selection(app, path)
            }
        }
        KeyCode::Char('v') => {
            if app.mode == AppMode::Normal
                && app.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected)
            {
                let target_path = if app.alt_pressed {
                    std::path::PathBuf::from(path).join(&entry.path)
                } else {
                    std::path::PathBuf::from(path)
                };
                crate::utils::terminal::open_ide(&target_path, &app.config.ide_command);
            }
            false
        }
        KeyCode::Char('t') => {
            if app.mode == AppMode::Normal
                && app.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected)
            {
                let full_path = std::path::PathBuf::from(path).join(&entry.path);
                let dir = if entry.is_dir {
                    full_path
                } else {
                    full_path
                        .parent()
                        .unwrap_or(&std::path::PathBuf::from(path))
                        .to_path_buf()
                };

                if app.alt_pressed {
                    crate::utils::terminal::open_terminal(&dir);
                } else {
                    app.mode = AppMode::Message(
                        "Inline TTY coming soon... (Use Alt+t for External)".to_string(),
                    );
                }
            }
            false
        }
        KeyCode::Char('a') => {
            if app.mode == AppMode::Normal
                && app.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.file_state.file_tree.get(app.file_state.file_selected)
                {
                    let target_path = if app.alt_pressed {
                        std::path::PathBuf::from(path).join(&entry.path)
                    } else {
                        std::path::PathBuf::from(path)
                    };
                    crate::utils::terminal::open_ide(
                        &target_path,
                        &app.config.alternative_ide_command,
                    );
                } else if app.mode == AppMode::StashDetail
                && let Some(stash) = app.stash_state.stashes.get(app.stash_state.stash_selected)
            {
                let msg = apply_stash(path, &stash.id);
                app.refresh_branches(path);
                app.mode = AppMode::Message(msg);
            }
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match app.mode {
                AppMode::Diff => app.branch_state.info_scroll += 1,
                AppMode::StashDetail => {
                    if app.stash_state.stash_selected
                        < app.stash_state.stashes.len().saturating_sub(1)
                    {
                        app.stash_state.stash_selected += 1;
                        app.load_stash_detail(path);
                    }
                }
                AppMode::Manage => {
                    if app.branch_state.manage_selected < 4 {
                        app.branch_state.manage_selected += 1;
                    }
                }
                AppMode::Filter => {
                    if app.branch_state.filter_selected < 9 {
                        app.branch_state.filter_selected += 1;
                    }
                }
                _ => app.next(),
            }
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match app.mode {
                AppMode::Diff => {
                    app.branch_state.info_scroll = app.branch_state.info_scroll.saturating_sub(1)
                }
                AppMode::StashDetail => {
                    if app.stash_state.stash_selected > 0 {
                        app.stash_state.stash_selected -= 1;
                        app.load_stash_detail(path);
                    }
                }
                AppMode::Manage => {
                    if app.branch_state.manage_selected > 0 {
                        app.branch_state.manage_selected -= 1;
                    }
                }
                AppMode::Filter => {
                    if app.branch_state.filter_selected > 0 {
                        app.branch_state.filter_selected -= 1;
                    }
                }
                _ => app.previous(),
            }
            false
        }
        KeyCode::Char(c) if c.is_ascii_digit() => {
            let digit = c.to_digit(10).unwrap() as usize;
            match app.mode {
                AppMode::Filter if digit < 10 => {
                    app.branch_state.filter_selected = digit;
                    handle_enter_or_selection(app, path);
                }
                AppMode::Manage if (1..=5).contains(&digit) => {
                    app.branch_state.manage_selected = digit - 1;
                    handle_enter_or_selection(app, path);
                }
                _ => {}
            }
            false
        }
        _ => false,
    }
}

fn handle_preview_keyboard(state: &mut PreviewState, key: KeyEvent) -> bool {
    let line_count = state.content.lines().count();
    match key.code {
        KeyCode::Char('j') | KeyCode::Down
            if state.cursor_y < line_count.saturating_sub(1) => {
                state.cursor_y += 1;
                if state.cursor_y >= state.scroll_y + 10 {
                    state.scroll_y += 1;
                }
        }
        KeyCode::Char('k') | KeyCode::Up
            if state.cursor_y > 0 => {
                state.cursor_y -= 1;
                if state.cursor_y < state.scroll_y && state.scroll_y > 0 {
                    state.scroll_y -= 1;
                }
        }
        KeyCode::Char('g') => {
            state.cursor_y = 0;
            state.scroll_y = 0;
        }
        KeyCode::Char('G') => {
            state.cursor_y = line_count.saturating_sub(1);
            state.scroll_y = state.cursor_y.saturating_sub(10);
        }
        _ => {}
    }
    false
}

fn handle_search_keyboard(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => {
            app.mode = AppMode::Normal;
        }
        KeyCode::Char(c) => {
            app.branch_state.search_query.push(c);
            app.refresh_filtered_branches();
        }
        KeyCode::Backspace => {
            app.branch_state.search_query.pop();
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
            KeyCode::Esc => {
                app.settings_state.editing = false;
            }
            KeyCode::Char(c) => {
                app.settings_state.input.push(c);
            }
            KeyCode::Backspace => {
                app.settings_state.input.pop();
            }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.settings_state.selected > 0 => {
            app.settings_state.selected -= 1;
        }
        KeyCode::Down | KeyCode::Char('j') if app.settings_state.selected < 2 => {
            app.settings_state.selected += 1;
        }
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
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::BackTab => {
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
    false
}

fn handle_enter_or_selection(app: &mut App, path: &str) -> bool {
    match app.mode {
        AppMode::Normal
            if app.primary_mode == PrimaryMode::Branches
                && !app.get_filtered_branches().is_empty() =>
        {
            app.mode = AppMode::Manage;
            app.branch_state.manage_selected = 0;
        }
        AppMode::Manage => {
            let branch_name = {
                let filtered = app.get_filtered_branches();
                filtered
                    .get(app.branch_state.selected)
                    .map(|b| b.name.clone())
            };

            if let Some(name) = branch_name {
                match app.branch_state.manage_selected {
                    0 => {
                        let msg = checkout_branch(path, &name);
                        app.refresh_branches(path);
                        app.current_branch = git::get_current_branch(path);
                        app.mode = AppMode::Message(msg);
                    }
                    1 => {
                        let mut info = git::get_branch_info(path, &name);
                        let branch = app
                            .get_filtered_branches()
                            .get(app.branch_state.selected)
                            .copied();
                        if let Some(crate::models::MergeStatus::SafeLimit(safe, total)) =
                            branch.map(|b| &b.merge_status)
                        {
                            info = format!(
                                "--- CONFLICT DETECTED ---\nSafe commits: {}/{}\n\n{}",
                                safe, total, info
                            );
                        } else if let Some(crate::models::MergeStatus::Conflict(conflicts)) =
                            branch.map(|b| &b.merge_status)
                        {
                            let files: Vec<String> =
                                conflicts.iter().map(|c| c.file_path.clone()).collect();
                            info = format!(
                                "--- CONFLICTS FOUND IN {} FILES ---\n[Shift+F] to resolve with AI\n\nFiles:\n{}\n\n{}",
                                conflicts.len(),
                                files.join("\n"),
                                info
                            );
                        }
                        app.branch_state.branch_info = info;
                        app.branch_state.info_scroll = 0;
                        app.mode = AppMode::Diff;
                    }
                    2 => {
                        // Individual delete also triggers snap if we want consistency, 
                        // but let's just do it for bulk for now as requested.
                        // Or we can just start the snap for this one branch.
                        app.snap_animation = Some(SnapAnimation::new(vec![name]));
                        app.mode = AppMode::Normal;
                    }
                    3 => app.mode = AppMode::Help,
                    _ => app.mode = AppMode::Normal,
                }
            } else {
                app.mode = AppMode::Normal;
            }
        }
        AppMode::Filter => {
            app.branch_state.current_filter = match app.branch_state.filter_selected {
                0 => None,
                1 => Some(BranchStatus::Merged),
                2 => Some(BranchStatus::Local),
                3 => Some(BranchStatus::Stashed),
                4 => Some(BranchStatus::Gone),
                5 => Some(BranchStatus::Ahead),
                6 => Some(BranchStatus::Behind),
                7 => Some(BranchStatus::HasUniqueCommits),
                8 => Some(BranchStatus::RemoteTracked),
                9 => Some(BranchStatus::RemoteUntracked),
                _ => None,
            };
            app.refresh_filtered_branches();
            app.branch_state.selected = 0;
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
    false
}
