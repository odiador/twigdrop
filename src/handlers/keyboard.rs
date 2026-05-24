use crate::actions::{apply_stash, prune_branches};
use crate::app::App;
use crate::state::ui::{AppMode, PrimaryMode, FilePanel, PreviewState, RebaseAction};
use crate::git;
use crate::ui::animations::SnapAnimation;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    app.ui.alt_pressed = key.modifiers.contains(KeyModifiers::ALT);
    app.ui.shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);

    // Global quit
    if key.code == KeyCode::Char('q') {
        let is_editing = match &app.ui.mode {
            AppMode::Settings => app.ui.settings_state.editing || app.ui.settings_state.selecting,
            AppMode::Search | AppMode::CreateBranch(_) | AppMode::Shell(_) => true,
            AppMode::InteractiveRebase => app.ui.rebase_state.editing,
            AppMode::CommitAction(_) => app.ui.settings_state.editing,
            _ => false,
        };
        if !is_editing {
            std::process::exit(0);
        }
    }

    if let AppMode::Message(_) = app.ui.mode {
        app.ui.mode = AppMode::Normal;
        return false;
    }

    if let AppMode::Settings = app.ui.mode {
        return handle_settings_keyboard(app, key);
    }

    if let AppMode::Search = app.ui.mode {
        return handle_search_keyboard(app, key);
    }

    if let AppMode::Filter = app.ui.mode {
        return handle_filter_keyboard(app, key);
    }

    if let AppMode::Manage = app.ui.mode {
        return handle_manage_keyboard(app, key, path);
    }

    if let AppMode::ConfirmDelete(names) = &app.ui.mode {
        let names = names.clone();
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                app.ui.snap_animation = Some(SnapAnimation::new(names));
                app.ui.mode = AppMode::Normal;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                app.ui.mode = AppMode::Normal;
            }
            _ => {}
        }
        return false;
    }

    if let AppMode::Commits = app.ui.mode {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if app.ui.selected_commit_idx > 0 => {
                app.ui.selected_commit_idx -= 1;
            }
            KeyCode::Down | KeyCode::Char('j') if app.ui.selected_commit_idx < app.repo.commits.len().saturating_sub(1) => {
                app.ui.selected_commit_idx += 1;
            }
            KeyCode::Enter => {
                if let Some(commit) = app.repo.commits.get(app.ui.selected_commit_idx) {
                    app.ui.mode = AppMode::CommitAction(commit.hash.clone());
                    // we can use settings_state to hold the selected action
                    app.ui.settings_state.selected = 0; 
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui.mode = AppMode::Normal;
            }
            _ => {}
        }
        return false;
    }

    if let AppMode::CommitAction(ref hash) = app.ui.mode {
        let hash = hash.clone();
        if app.ui.settings_state.editing {
            match key.code {
                KeyCode::Enter => {
                    let new_date = app.ui.settings_state.input.clone();
                    app.ui.settings_state.editing = false;
                    // Run rebase to change date
                    // For YOLO mode simplicity, let's use rebase --exec
                    let exec_cmd = format!("git commit --amend --no-edit --date=\"{}\"", new_date);
                    let msg = match crate::git::commands::run_git(path, &["rebase", &format!("{}^", hash), "--exec", &exec_cmd]) {
                        Ok(m) => format!("Date updated to {}.\n{}", new_date, m),
                        Err(e) => format!("Failed to update date: {}", e),
                    };
                    app.ui.mode = AppMode::Message(msg);
                }
                KeyCode::Esc => {
                    app.ui.settings_state.editing = false;
                }
                KeyCode::Char(c) => { app.ui.settings_state.input.push(c); }
                KeyCode::Backspace => { app.ui.settings_state.input.pop(); }
                _ => {}
            }
            return false;
        }

        match key.code {
            KeyCode::Up | KeyCode::Char('k') if app.ui.settings_state.selected > 0 => {
                app.ui.settings_state.selected -= 1;
            }
            KeyCode::Down | KeyCode::Char('j') if app.ui.settings_state.selected < 1 => {
                app.ui.settings_state.selected += 1;
            }
            KeyCode::Enter => {
                if app.ui.settings_state.selected == 0 {
                    app.ui.settings_state.editing = true;
                    app.ui.settings_state.input = "now".to_string();
                } else if app.ui.settings_state.selected == 1 {
                    let msg = crate::git::commands::run_git(path, &["commit", "--fixup", &hash]).unwrap_or_else(|e| e.to_string());
                    let _ = crate::git::commands::run_git(path, &["-c", "sequence.editor=:", "rebase", "-i", "--autosquash", &format!("{}^", hash)]);
                    app.ui.mode = AppMode::Message(format!("Amended to {}:\n{}", hash, msg));
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui.mode = AppMode::Commits;
            }
            _ => {}
        }
        return false;
    }

    if let AppMode::Shell(ref mut input) = app.ui.mode {
        match key.code {
            KeyCode::Enter => {
                let cmd = input.clone();
                if !cmd.is_empty() {
                    let msg = match crate::git::commands::run_git(path, &["-c", "alias.s=!", &cmd, "s"]) {
                        Ok(m) => format!("$ {}\n{}", cmd, m),
                        Err(e) => format!("Error executing {}: {}", cmd, e),
                    };
                    app.ui.mode = AppMode::Message(msg);
                } else {
                    app.ui.mode = AppMode::Normal;
                }
            }
            KeyCode::Esc => { app.ui.mode = AppMode::Normal; }
            KeyCode::Char(c) => { input.push(c); }
            KeyCode::Backspace => { input.pop(); }
            _ => {}
        }
        return false;
    }

    if let AppMode::MainMenu = app.ui.mode {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if app.ui.main_menu_state.selected > 0 => {
                app.ui.main_menu_state.selected -= 1;
            }
            KeyCode::Down | KeyCode::Char('j') if app.ui.main_menu_state.selected < 2 => {
                app.ui.main_menu_state.selected += 1;
            }
            KeyCode::Enter => {
                match app.ui.main_menu_state.selected {
                    0 => {
                        app.ui.mode = AppMode::Settings;
                        app.ui.settings_state.selected = 0;
                        app.ui.settings_state.editing = false;
                        app.ui.needs_clear = true;
                    }
                    1 => {
                        app.toggle_help();
                        app.ui.needs_clear = true;
                    }
                    2 => std::process::exit(0), // Quit
                    _ => {}
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui.mode = AppMode::Normal;
                app.ui.needs_clear = true;
            }
            _ => {}
        }
        return false;
    }

    if let AppMode::QuickActions = app.ui.mode {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if app.ui.quick_actions_state.selected > 0 => {
                app.ui.quick_actions_state.selected -= 1;
            }
            KeyCode::Down | KeyCode::Char('j') if app.ui.quick_actions_state.selected < app.ui.quick_actions_state.actions.len().saturating_sub(1) => {
                app.ui.quick_actions_state.selected += 1;
            }
            KeyCode::Enter => {
                let action = app.ui.quick_actions_state.actions[app.ui.quick_actions_state.selected].clone();
                // Simple parser for git commands
                let parts: Vec<&str> = action.split_whitespace().collect();
                if parts.len() >= 2 && parts[0] == "git" {
                    let args = &parts[1..];
                    let msg = match crate::git::commands::run_git(path, args) {
                        Ok(m) => format!("> {}\n{}", action, m),
                        Err(e) => format!("Error: {}", e),
                    };
                    app.refresh_branches(path);
                    app.ui.mode = AppMode::Message(msg);
                }
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui.mode = AppMode::Normal;
            }
            _ => {}
        }
        return false;
    }

    if let AppMode::InteractiveRebase = app.ui.mode {
        if app.ui.rebase_state.editing {
            match key.code {
                KeyCode::Enter => {
                    app.ui.rebase_state.editing = false;
                    let i = app.ui.rebase_state.selected;
                    app.ui.rebase_state.commits[i].new_message = Some(app.ui.rebase_state.input.clone());
                    app.ui.rebase_state.commits[i].action = RebaseAction::Reword;
                }
                KeyCode::Esc => { app.ui.rebase_state.editing = false; }
                KeyCode::Char(c) => { app.ui.rebase_state.input.push(c); }
                KeyCode::Backspace => { app.ui.rebase_state.input.pop(); }
                _ => {}
            }
        } else {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') if app.ui.rebase_state.selected > 0 => {
                    app.ui.rebase_state.selected -= 1;
                }
                KeyCode::Down | KeyCode::Char('j') if app.ui.rebase_state.selected < app.ui.rebase_state.commits.len().saturating_sub(1) => {
                    app.ui.rebase_state.selected += 1;
                }
                KeyCode::Char('r') => {
                    app.ui.rebase_state.commits[app.ui.rebase_state.selected].action = RebaseAction::Reword;
                }
                KeyCode::Char('d') => {
                    app.ui.rebase_state.commits[app.ui.rebase_state.selected].action = RebaseAction::Drop;
                }
                KeyCode::Char('s') => {
                    app.ui.rebase_state.commits[app.ui.rebase_state.selected].action = RebaseAction::Squash;
                }
                KeyCode::Char('p') => {
                    app.ui.rebase_state.commits[app.ui.rebase_state.selected].action = RebaseAction::Pick;
                }
                KeyCode::Enter => {
                    app.ui.rebase_state.editing = true;
                    let i = app.ui.rebase_state.selected;
                    app.ui.rebase_state.input = app.ui.rebase_state.commits[i].new_message.clone().unwrap_or_else(|| app.ui.rebase_state.commits[i].original_message.clone());
                }
                KeyCode::Char('A') => {
                    app.ui.rebase_state.ai_analyzing = true;
                    let i = app.ui.rebase_state.selected;
                    let hash = app.ui.rebase_state.commits[i].hash.clone();
                    let _ = app.ai_state.ai_trigger_tx.try_send(("rename_commit".to_string(), path.to_string(), hash));
                }
                KeyCode::Char('E') => {
                    // Execute rebase
                    crate::actions::commands::execute_interactive_rebase(path, &app.ui.rebase_state.commits);
                    app.ui.mode = AppMode::Message("Rebase script generated and executed.".to_string());
                }
                // When they press 'C', we could trigger the rebase execution here or in another key.
                KeyCode::Esc | KeyCode::Char('q') => { app.ui.mode = AppMode::Normal; }
                _ => {}
            }
        }
        return false;
    }

    if let AppMode::CreateBranch(ref mut input) = app.ui.mode {
        match key.code {
            KeyCode::Enter => {
                let name = input.clone();
                if !name.is_empty() {
                    match crate::git::commands::run_git(path, &["checkout", "-b", &name]) {
                        Ok(msg) => {
                            app.refresh_branches(path);
                            app.repo.current_branch = git::get_current_branch(path);
                            app.ui.mode = AppMode::Message(format!("Created and checked out: {}\n{}", name, msg));
                        }
                        Err(e) => app.ui.mode = AppMode::Message(format!("Error creating branch: {}", e)),
                    }
                } else {
                    app.ui.mode = AppMode::Normal;
                }
            }
            KeyCode::Esc => { app.ui.mode = AppMode::Normal; }
            KeyCode::Char(c) => { input.push(c); }
            KeyCode::Backspace => { input.pop(); }
            _ => {}
        }
        return false;
    }

    if let AppMode::CodePreview(ref mut state) = app.ui.mode {
        if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
            app.ui.needs_clear = true;
        } else if app.ui.active_panel == FilePanel::Preview {
            return handle_preview_keyboard(state, key.code);
        }
    }
    
    if matches!(app.ui.mode, AppMode::CodePreview(_)) && (key.code == KeyCode::Esc || key.code == KeyCode::Char('q')) {
        app.ui.mode = AppMode::Normal;
        app.ui.active_panel = FilePanel::Directory;
        return false;
    }

    match key.code {
        KeyCode::Char('q') => {
            if app.ui.mode != AppMode::Normal {
                if app.ui.mode == AppMode::Help {
                    app.refresh_branches(path);
                }
                app.ui.mode = AppMode::Normal;
                app.ui.needs_clear = true;
                false
            } else if app.ui.current_filter.is_some() {
                app.ui.current_filter = None;
                app.refresh_filtered_branches();
                app.ui.selected_branch_idx = 0;
                false
            } else {
                std::process::exit(0);
            }
        }
        KeyCode::Esc => {
            if app.ui.mode == AppMode::Normal {
                app.ui.mode = AppMode::MainMenu;
                app.ui.main_menu_state.selected = 0;
                false
            } else {
                app.ui.mode = AppMode::Normal;
                app.ui.needs_clear = true;
                false
            }
        }
        KeyCode::Tab => {
            if matches!(app.ui.mode, AppMode::CodePreview(_)) {
                app.ui.active_panel = match app.ui.active_panel {
                    FilePanel::Directory => FilePanel::Preview,
                    FilePanel::Preview => FilePanel::Directory,
                };
            } else if app.ui.mode == AppMode::Diff {
                app.ui.diff_panel = match app.ui.diff_panel {
                    FilePanel::Directory => FilePanel::Preview,
                    FilePanel::Preview => FilePanel::Directory,
                };
            }
            false
        }
        KeyCode::Char('?') => {
            app.toggle_help();
            false
        }
        KeyCode::Char('f') => {
            if app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Branches {
                app.ui.mode = AppMode::Filter;
                app.ui.filter_selected = 0;
            }
            false
        }
        KeyCode::Char('/') => {
            if app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Branches {
                app.ui.mode = AppMode::Search;
                app.ui.search_query.clear();
                app.refresh_filtered_branches();
            }
            false
        }
        KeyCode::Char('c') => {
            if app.ui.mode == AppMode::Normal {
                app.ui.mode = AppMode::CreateBranch(String::new());
            }
            false
        }
        KeyCode::Char('!') => {
            if app.ui.mode == AppMode::Normal {
                app.ui.mode = AppMode::Shell(String::new());
            }
            false
        }
        KeyCode::Char(':') => {
            if app.ui.mode == AppMode::Normal {
                app.ui.mode = AppMode::QuickActions;
                app.ui.quick_actions_state.selected = 0;
            }
            false
        }
        KeyCode::Char('[') => {
            if app.ui.primary_mode == PrimaryMode::Files && app.ui.sidebar_width > 10 {
                app.ui.sidebar_width -= 2;
                app.ui.needs_clear = true;
            }
            false
        }
        KeyCode::Char(']') => {
            if app.ui.primary_mode == PrimaryMode::Files && app.ui.sidebar_width < 90 {
                app.ui.sidebar_width += 2;
                app.ui.needs_clear = true;
            }
            false
        }
        KeyCode::Char('e') => {
            if app.ui.mode == AppMode::Normal {
                let target_path = if app.ui.alt_pressed
                    && app.ui.primary_mode == PrimaryMode::Files
                    && let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx)
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
            app.ui.show_terminal = !app.ui.show_terminal;
            false
        }
        KeyCode::Char('d') => {
            if app.ui.mode == AppMode::Normal || matches!(app.ui.mode, AppMode::CodePreview(_)) {
                app.toggle_primary_mode();
                if app.ui.primary_mode == PrimaryMode::Files {
                    app.load_file_tree(path);
                }
                if matches!(app.ui.mode, AppMode::CodePreview(_)) {
                    app.ui.mode = AppMode::Normal;
                    app.ui.active_panel = FilePanel::Directory;
                }
            }
            false
        }
        KeyCode::Left => {
            if app.ui.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
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
            if app.ui.primary_mode == PrimaryMode::Files
                && let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx)
                && entry.is_dir {
                    if !entry.is_open {
                        app.toggle_file_dir(path);
                    } else if app.ui.selected_file_idx + 1 < app.repo.file_tree.len() {
                        app.ui.selected_file_idx += 1;
                    }
            }
            false
        }
        KeyCode::Char('m') | KeyCode::Enter => {
            if app.ui.primary_mode == PrimaryMode::Files && key.code == KeyCode::Enter {
                if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                    if entry.is_dir {
                        app.toggle_file_dir(path);
                    } else {
                        let rel_path = entry.path.to_string_lossy().to_string();
                        if let Some(preview) = app.create_preview_state(path, &rel_path) {
                            app.ui.mode = AppMode::CodePreview(preview);
                        }
                    }
                }
                false
            } else {
                handle_enter_or_selection(app, path)
            }
        }
        KeyCode::Char('j') | KeyCode::Down => {
            match app.ui.mode {
                AppMode::CodePreview(ref mut state) if app.ui.active_panel == FilePanel::Preview => {
                    handle_preview_keyboard(state, KeyCode::Down);
                    false
                }
                AppMode::CodePreview(_) if app.ui.active_panel == FilePanel::Directory => {
                    app.next();
                    if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                        if !entry.is_dir {
                            let rel_path = entry.path.to_string_lossy().to_string();
                            if let Some(preview) = app.create_preview_state(path, &rel_path) {
                                app.ui.mode = AppMode::CodePreview(preview);
                            }
                        }
                    }
                    false
                }
                AppMode::Diff => { 
                    if app.ui.diff_panel == FilePanel::Directory {
                        if app.ui.diff_file_selected + 1 < app.repo.diff_files.len() {
                            app.ui.diff_file_selected += 1;
                            update_diff_preview(app, path);
                        }
                    } else if let Some(ref mut state) = app.repo.diff_preview {
                        handle_preview_keyboard(state, KeyCode::Down);
                    } else {
                        app.ui.info_scroll += 1; 
                    }
                    false 
                }
                AppMode::StashDetail => {
                    if app.ui.selected_stash_idx < app.repo.stashes.len().saturating_sub(1) {
                        app.ui.selected_stash_idx += 1;
                        app.load_stash_detail(path);
                    }
                    false
                }
                _ => { app.next(); false }
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            match app.ui.mode {
                AppMode::CodePreview(ref mut state) if app.ui.active_panel == FilePanel::Preview => {
                    handle_preview_keyboard(state, KeyCode::Up);
                    false
                }
                AppMode::CodePreview(_) if app.ui.active_panel == FilePanel::Directory => {
                    app.previous();
                    if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                        if !entry.is_dir {
                            let rel_path = entry.path.to_string_lossy().to_string();
                            if let Some(preview) = app.create_preview_state(path, &rel_path) {
                                app.ui.mode = AppMode::CodePreview(preview);
                            }
                        }
                    }
                    false
                }
                AppMode::Diff => { 
                    if app.ui.diff_panel == FilePanel::Directory {
                        app.ui.diff_file_selected = app.ui.diff_file_selected.saturating_sub(1);
                        update_diff_preview(app, path);
                    } else if let Some(ref mut state) = app.repo.diff_preview {
                        handle_preview_keyboard(state, KeyCode::Up);
                    } else {
                        app.ui.info_scroll = app.ui.info_scroll.saturating_sub(1); 
                    }
                    false 
                }
                AppMode::StashDetail => {
                    if app.ui.selected_stash_idx > 0 {
                        app.ui.selected_stash_idx -= 1;
                        app.load_stash_detail(path);
                    }
                    false
                }
                _ => { app.previous(); false }
            }
        }
        KeyCode::Char('g') if let AppMode::CodePreview(ref mut state) = app.ui.mode
            && app.ui.active_panel == FilePanel::Preview => {
                state.cursor_y = 0;
                state.scroll_y = 0;
                false
        }
        KeyCode::Char('G') if let AppMode::CodePreview(ref mut state) = app.ui.mode
            && app.ui.active_panel == FilePanel::Preview => {
                let line_count = state.lines.len();
                state.cursor_y = line_count.saturating_sub(1);
                state.scroll_y = state.cursor_y.saturating_sub(10);
                false
        }
        _ => handle_generic_actions(app, key, path)
    }
}

pub fn update_diff_preview(app: &mut App, path: &str) {
    if let Some(branch) = app.get_filtered_branches().get(app.ui.selected_branch_idx) {
        let branch_name = branch.name.clone();
        if let Some(file) = app.repo.diff_files.get(app.ui.diff_file_selected) {
            let diff_content = crate::git::get_branch_file_diff(path, &branch_name, file);
            let mut preview = PreviewState {
                file_path: file.clone(),
                lines: diff_content.lines().map(|s| s.to_string()).collect(),
                highlighted_lines: vec![],
                cursor_y: 0,
                scroll_y: 0,
                selection_start: None,
                selection_end: None,
                line_diffs: std::collections::HashMap::new(),
            };
            app.update_diff_highlighting(&mut preview);
            app.repo.diff_preview = Some(preview);
        }
    }
}

fn handle_generic_actions(app: &mut App, key: KeyEvent, path: &str) -> bool {
    match key.code {
        KeyCode::F(5) | KeyCode::Char('p') if app.ui.mode == AppMode::Normal => {
            let msg = prune_branches(path, &app.repo.branches, &app.repo.current_branch);
            app.refresh_branches(path);
            app.ui.mode = AppMode::Message(msg);
            false
        }
        KeyCode::Char('i') if app.ui.mode == AppMode::Normal => {
            if let Some(branch) = app.get_filtered_branches().get(app.ui.selected_branch_idx) {
                if branch.name.starts_with('*') { return false; }
                let branch_name = branch.name.clone();
                let _ = app.ai_state.ai_trigger_tx.try_send(("analyze".to_string(), path.to_string(), branch_name.clone()));
                app.ai_state.ai_analysis = Some("Initializing AI analysis...".to_string());
                app.repo.branch_info = git::get_branch_info(path, &branch_name);
                app.ui.info_scroll = 0;
                
                app.repo.diff_files = git::get_branch_diff_files(path, &branch_name);
                app.ui.diff_file_selected = 0;
                app.repo.diff_preview = None;
                if !app.repo.diff_files.is_empty() {
                    let first_file = app.repo.diff_files[0].clone();
                    let diff_content = git::get_branch_file_diff(path, &branch_name, &first_file);
                    let mut preview = PreviewState {
                        file_path: first_file,
                        lines: diff_content.lines().map(|s| s.to_string()).collect(),
                        highlighted_lines: vec![],
                        cursor_y: 0,
                        scroll_y: 0,
                        selection_start: None,
                        selection_end: None,
                        line_diffs: std::collections::HashMap::new(),
                    };
                    app.update_diff_highlighting(&mut preview);
                    app.repo.diff_preview = Some(preview);
                }
                
                app.ui.mode = AppMode::Diff;
            }
            false
        }
        KeyCode::Char(' ') if app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Branches => {
            if let Some(branch) = app.get_filtered_branches().get(app.ui.selected_branch_idx) {
                if !branch.name.starts_with('*') {
                    app.toggle_selection();
                }
            }
            false
        }
        KeyCode::F(8) | KeyCode::Char('D') if (key.code == KeyCode::F(8) || app.ui.shift_pressed) && app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Branches => {
            if !app.ui.bulk_selected.is_empty() {
                let names: Vec<String> = app.ui.bulk_selected.iter().cloned().collect();
                app.ui.snap_animation = Some(SnapAnimation::new(names));
            }
            false
        }
        KeyCode::Char('C') if app.ui.shift_pressed && app.ui.mode == AppMode::Normal => {
            app.repo.commits = crate::git::get_unpushed_commits(path);
            app.ui.selected_commit_idx = 0;
            app.ui.mode = AppMode::Commits;
            false
        }
        KeyCode::Char('R') if app.ui.shift_pressed && app.ui.mode == AppMode::Normal => {
            app.load_rebase_commits(path);
            app.ui.mode = AppMode::InteractiveRebase;
            false
        }
        KeyCode::Char('v') if app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Files => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                let target_path = if app.ui.alt_pressed { std::path::PathBuf::from(path).join(&entry.path) } else { std::path::PathBuf::from(path) };
                crate::utils::terminal::open_ide(&target_path, &app.config.ide_command);
            }
            false
        }
        KeyCode::Char('a') if app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Files => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                let target_path = if app.ui.alt_pressed { std::path::PathBuf::from(path).join(&entry.path) } else { std::path::PathBuf::from(path) };
                crate::utils::terminal::open_ide(&target_path, &app.config.alternative_ide_command);
            }
            false
        }
        KeyCode::Char('a') if app.ui.mode == AppMode::StashDetail => {
            if let Some(stash) = app.repo.stashes.get(app.ui.selected_stash_idx) {
                let msg = apply_stash(path, &stash.id);
                app.refresh_branches(path);
                app.ui.mode = AppMode::Message(msg);
            }
            false
        }
        KeyCode::Char('S') if app.ui.shift_pressed && app.ui.mode == AppMode::Normal => {
            app.load_stashes(path);
            app.load_stash_detail(path);
            app.ui.mode = AppMode::StashDetail;
            false
        }
        KeyCode::Char('t') if app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Files => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                let full_path = std::path::PathBuf::from(path).join(&entry.path);
                let dir = if entry.is_dir { full_path } else { full_path.parent().unwrap_or(&std::path::PathBuf::from(path)).to_path_buf() };
                crate::utils::terminal::open_terminal(&dir);
            }
            false
        }
        KeyCode::F(5) | KeyCode::Char('s') if app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Files => {
            if let Some(entry) = app.repo.file_tree.get(app.ui.selected_file_idx) {
                if !entry.is_dir {
                    let rel_path = entry.path.to_string_lossy().to_string().replace('\\', "/");
                    let result = if entry.status == crate::git::files::FileStatus::Staged {
                        crate::actions::commands::unstage_file(path, &rel_path)
                    } else {
                        crate::actions::commands::stage_file(path, &rel_path)
                    };

                    match result {
                        Ok(msg) => {
                            app.load_file_tree(path);
                            app.ui.mode = AppMode::Message(msg);
                        }
                        Err(e) => app.ui.mode = AppMode::Message(e),
                    }
                }
            }
            false
        }
        KeyCode::Char('F') if app.ui.shift_pressed && app.ui.mode == AppMode::Diff => {
             let branch = app.get_filtered_branches().get(app.ui.selected_branch_idx).copied();
             if let Some(crate::models::MergeStatus::Conflict(conflicts)) = branch.map(|b| &b.merge_status) {
                 for conflict in conflicts { let _ = app.ai_state.conflict_trigger_tx.try_send((path.to_string(), conflict.clone())); }
                 app.ui.mode = AppMode::Message("AI Resolving conflicts...".to_string());
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
        KeyCode::Enter | KeyCode::Esc => { app.ui.mode = AppMode::Normal; }
        KeyCode::Char(c) => { app.ui.search_query.push(c); app.refresh_filtered_branches(); }
        KeyCode::Backspace => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                app.ui.search_query.clear();
            } else {
                app.ui.search_query.pop();
            }
            app.refresh_filtered_branches();
        }
        _ => {}
    }
    false
}

fn handle_settings_keyboard(app: &mut App, key: KeyEvent) -> bool {
    if app.ui.settings_state.editing {
        match key.code {
            KeyCode::Enter => {
                match app.ui.settings_state.selected {
                    0 => app.config.ide_command = app.ui.settings_state.input.clone(),
                    1 => app.config.alternative_ide_command = app.ui.settings_state.input.clone(),
                    2 => app.config.ai_provider = app.ui.settings_state.input.clone(),
                    3 => app.config.current_provider_mut().model = app.ui.settings_state.input.clone(),
                    4 => app.config.current_provider_mut().api_key = crate::utils::config::obfuscate(&app.ui.settings_state.input),
                    5 => app.config.current_provider_mut().url = app.ui.settings_state.input.clone(),
                    7 => app.config.default_sidebar_width = app.ui.settings_state.input.parse().unwrap_or(30),
                    _ => {}
                }
                app.ui.settings_state.editing = false;
                crate::utils::config::save_config(&app.config);
            }
            KeyCode::Esc => { app.ui.settings_state.editing = false; }
            KeyCode::Char(c) => { app.ui.settings_state.input.push(c); }
            KeyCode::Backspace => { app.ui.settings_state.input.pop(); }
            _ => {}
        }
        return false;
    }

    if app.ui.settings_state.selecting {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if app.ui.settings_state.choice_idx > 0 {
                    app.ui.settings_state.choice_idx -= 1;
                } else {
                    app.ui.settings_state.choice_idx = app.ui.settings_state.choices.len().saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.ui.settings_state.choice_idx + 1 < app.ui.settings_state.choices.len() {
                    app.ui.settings_state.choice_idx += 1;
                } else {
                    app.ui.settings_state.choice_idx = 0;
                }
            }
            KeyCode::Enter => {
                if let Some(choice) = app.ui.settings_state.choices.get(app.ui.settings_state.choice_idx) {
                    match app.ui.settings_state.selected {
                        2 => app.config.ai_provider = choice.clone(),
                        3 => app.config.current_provider_mut().model = choice.clone(),
                        _ => {}
                    }
                }
                app.ui.settings_state.selecting = false;
                crate::utils::config::save_config(&app.config);
            }
            KeyCode::Esc | KeyCode::Char('q') => { app.ui.settings_state.selecting = false; }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.ui.settings_state.selected > 0 => { app.ui.settings_state.selected -= 1; }
        KeyCode::Down | KeyCode::Char('j') if app.ui.settings_state.selected < 8 => { app.ui.settings_state.selected += 1; }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            match app.ui.settings_state.selected {
                2 => { // AI Provider
                    app.ui.settings_state.selecting = true;
                    app.ui.settings_state.choices = crate::utils::config::PROVIDER_ARCHETYPES.iter().map(|s| s.to_string()).collect();
                    app.ui.settings_state.choice_idx = app.ui.settings_state.choices.iter().position(|s| s == &app.config.ai_provider).unwrap_or(0);
                }
                3 => { // AI Model
                    app.ui.settings_state.selecting = true;
                    app.ui.settings_state.choices = match app.config.ai_provider.as_str() {
                        "openai" => crate::utils::config::OPENAI_MODELS.iter().map(|s| s.to_string()).collect(),
                        "anthropic" => crate::utils::config::ANTHROPIC_MODELS.iter().map(|s| s.to_string()).collect(),
                        "google" => crate::utils::config::GOOGLE_MODELS.iter().map(|s| s.to_string()).collect(),
                        _ => vec!["Loading models...".to_string()],
                    };
                    app.ui.settings_state.choice_idx = app.ui.settings_state.choices.iter().position(|s| s == &app.config.current_provider().model).unwrap_or(0);
                }
                6 => { // Enable Animations
                    app.config.enable_animations = !app.config.enable_animations;
                    crate::utils::config::save_config(&app.config);
                }
                8 => { // Save and Exit
                    crate::utils::config::save_config(&app.config);
                    app.ui.mode = AppMode::Normal;
                }
                _ => {
                    app.ui.settings_state.editing = true;
                    app.ui.settings_state.input = match app.ui.settings_state.selected {
                        0 => app.config.ide_command.clone(),
                        1 => app.config.alternative_ide_command.clone(),
                        4 => crate::utils::config::deobfuscate(&app.config.current_provider().api_key),
                        5 => app.config.current_provider().url.clone(),
                        7 => app.config.default_sidebar_width.to_string(),
                        _ => String::new(),
                    };
                }
            }
        }
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::BackTab => { app.ui.mode = AppMode::Normal; }
        _ => {}
    }
    false
}

fn handle_manage_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    const MANAGE_OPTIONS_COUNT: usize = 7;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.ui.manage_selected = app.ui.manage_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') if app.ui.manage_selected < MANAGE_OPTIONS_COUNT - 1 => {
            app.ui.manage_selected += 1;
        }
        KeyCode::Enter => {
            let branch = app.get_filtered_branches().get(app.ui.selected_branch_idx).cloned();
            if let Some(b) = branch {
                match app.ui.manage_selected {
                    0 => {
                        let msg = crate::git::commands::run_git(path, &["checkout", &b.name]).unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.repo.current_branch = git::get_current_branch(path);
                        app.ui.mode = AppMode::Message(msg);
                    }
                    1 => {
                        let branch_name = b.name.clone();
                        let _ = app.ai_state.ai_trigger_tx.try_send(("analyze".to_string(), path.to_string(), branch_name.clone()));
                        app.ai_state.ai_analysis = Some("Initializing AI analysis...".to_string());
                        app.repo.branch_info = git::get_branch_info(path, &branch_name);
                        app.ui.info_scroll = 0;
                        
                        app.repo.diff_files = git::get_branch_diff_files(path, &branch_name);
                        app.ui.diff_file_selected = 0;
                        app.repo.diff_preview = None;
                        if !app.repo.diff_files.is_empty() {
                            let first_file = app.repo.diff_files[0].clone();
                            let diff_content = git::get_branch_file_diff(path, &branch_name, &first_file);
                            let mut preview = PreviewState {
                                file_path: first_file,
                                lines: diff_content.lines().map(|s| s.to_string()).collect(),
                                highlighted_lines: vec![],
                                cursor_y: 0,
                                scroll_y: 0,
                                selection_start: None,
                                selection_end: None,
                                line_diffs: std::collections::HashMap::new(),
                            };
                            app.update_diff_highlighting(&mut preview);
                            app.repo.diff_preview = Some(preview);
                        }
                        
                        app.ui.mode = AppMode::Diff;
                    }
                    2 => {
                        let names = vec![b.name.clone()];
                        if b.status.contains(&crate::models::BranchStatus::HasUniqueCommits) {
                            app.ui.mode = AppMode::ConfirmDelete(names);
                        } else {
                            app.ui.snap_animation = Some(SnapAnimation::new(names));
                            app.ui.mode = AppMode::Normal;
                        }
                    }
                    3 => { app.ui.mode = AppMode::Message("Rename branch coming soon!".to_string()); }
                    4 => { 
                        let msg = crate::git::commands::run_git(path, &["stash", "push", "-m", &format!("Stash from Twigdrop: {}", b.name)]).unwrap_or_else(|e| e.to_string());
                        app.ui.mode = AppMode::Message(msg);
                    }
                    5 => app.ui.mode = AppMode::Help,
                    _ => app.ui.mode = AppMode::Normal,
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.mode = AppMode::Normal;
        }
        _ => {}
    }
    false
}

fn handle_enter_or_selection(app: &mut App, path: &str) -> bool {
    if app.ui.mode == AppMode::Normal && app.ui.primary_mode == PrimaryMode::Branches {
        if let Some(branch) = app.get_filtered_branches().get(app.ui.selected_branch_idx).cloned() {
            if branch.name.starts_with('*') {
                // Pseudo branch: open Diff directly
                let branch_name = branch.name.clone();
                app.repo.branch_info = git::get_branch_info(path, &branch_name);
                app.ui.info_scroll = 0;
                
                app.repo.diff_files = git::get_branch_diff_files(path, &branch_name);
                app.ui.diff_file_selected = 0;
                app.repo.diff_preview = None;
                if !app.repo.diff_files.is_empty() {
                    let first_file = app.repo.diff_files[0].clone();
                    let diff_content = git::get_branch_file_diff(path, &branch_name, &first_file);
                    let mut preview = PreviewState {
                        file_path: first_file,
                        lines: diff_content.lines().map(|s| s.to_string()).collect(),
                        highlighted_lines: vec![],
                        cursor_y: 0,
                        scroll_y: 0,
                        selection_start: None,
                        selection_end: None,
                        line_diffs: std::collections::HashMap::new(),
                    };
                    app.update_diff_highlighting(&mut preview);
                    app.repo.diff_preview = Some(preview);
                }
                
                app.ui.mode = AppMode::Diff;
                return false;
            }
        }
        
        app.ui.mode = AppMode::Manage;
        app.ui.manage_selected = 0;
        return false;
    }
    false
}

fn handle_filter_keyboard(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.ui.filter_selected = app.ui.filter_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') if app.ui.filter_selected < 9 => {
            app.ui.filter_selected += 1;
        }
        KeyCode::Enter => {
            app.ui.current_filter = match app.ui.filter_selected {
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
            app.ui.mode = AppMode::Normal;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.mode = AppMode::Normal;
        }
        _ => {}
    }
    false
}
