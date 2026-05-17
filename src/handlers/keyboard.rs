use crate::actions::{
    apply_stash, bulk_delete_branches, checkout_branch, delete_branch, prune_branches,
};
use crate::app::{App, AppMode};
use crate::git;
use crate::models::BranchStatus;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    app.alt_pressed = key
        .modifiers
        .contains(ratatui::crossterm::event::KeyModifiers::ALT);

    if let AppMode::Message(_) = app.mode {
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
            } else if app.current_filter.is_some() {
                app.current_filter = None;
                app.refresh_filtered_branches();
                app.selected = 0;
                false
            } else {
                true // Signal to quit
            }
        }
        KeyCode::Char('h') => {
            app.toggle_help();
            false
        }
        KeyCode::Char('b')
            if key
                .modifiers
                .contains(ratatui::crossterm::event::KeyModifiers::CONTROL) =>
        {
            if app.mode == AppMode::Normal {
                app.load_file_tree(path);
                app.mode = AppMode::DirectorySearcher;
            }
            false
        }
        KeyCode::Char('S') => {
            if app.mode == AppMode::Normal {
                app.load_stashes(path);
                app.load_stash_detail(path);
                app.mode = AppMode::StashDetail;
            }
            false
        }
        KeyCode::Char('p') => {
            if app.mode == AppMode::Normal {
                let msg = prune_branches(path, &app.branches, &app.current_branch);
                app.refresh_branches(path);
                app.mode = AppMode::Message(msg);
            }
            false
        }
        KeyCode::Char('i') => {
            if app.mode == AppMode::Normal
                && let Some(branch) = app.get_filtered_branches().get(app.selected)
            {
                let branch_name = branch.name.clone();
                let path_clone = path.to_string();
                let _ = app
                    .ai_trigger_tx
                    .try_send((path_clone, branch_name.clone()));
                app.ai_analysis = Some("Initializing AI analysis...".to_string());

                // Switch to Diff mode to show the analysis panel
                let info = git::get_branch_info(path, &branch_name);
                app.branch_info = info;
                app.info_scroll = 0;
                app.mode = AppMode::Diff;
            }
            false
        }
        KeyCode::Char(' ') | KeyCode::Char('d') => {
            if app.mode == AppMode::Normal {
                app.toggle_selection();
            }
            false
        }
        KeyCode::Char('D') => {
            if app.mode == AppMode::Normal && !app.bulk_selected.is_empty() {
                let names: Vec<String> = app.bulk_selected.iter().cloned().collect();
                let msg = bulk_delete_branches(path, &names);
                app.bulk_selected.clear();
                app.refresh_branches(path);
                app.current_branch = git::get_current_branch(path);
                app.mode = AppMode::Message(msg);
            }
            false
        }
        KeyCode::Right => {
            if app.mode == AppMode::DirectorySearcher
                && let Some(entry) = app.file_tree.get(app.file_selected)
                && entry.is_dir
                && !entry.is_open
            {
                app.toggle_file_dir(path);
            }
            false
        }
        KeyCode::Left => {
            if app.mode == AppMode::DirectorySearcher
                && let Some(entry) = app.file_tree.get(app.file_selected)
            {
                if entry.is_dir && entry.is_open {
                    // Close if open
                    app.toggle_file_dir(path);
                } else if entry.depth > 0 {
                    // Just move focus to parent
                    let current_depth = entry.depth;
                    for i in (0..app.file_selected).rev() {
                        if app.file_tree[i].depth < current_depth {
                            app.file_selected = i;
                            break;
                        }
                    }
                }
            }
            false
        }
        KeyCode::Char('f') => {
            if app.mode == AppMode::Normal {
                app.mode = AppMode::Filter;
                app.filter_selected = 0;
            }
            false
        }
        KeyCode::Char('m') | KeyCode::Tab | KeyCode::Enter => {
            if app.mode == AppMode::DirectorySearcher {
                app.toggle_file_dir(path);
                false
            } else if app.mode == AppMode::StashDetail {
                app.load_stash_detail(path);
                false
            } else {
                handle_enter_or_selection(app, path)
            }
        }
        KeyCode::Char('v') => {
            if app.mode == AppMode::DirectorySearcher
                && let Some(entry) = app.file_tree.get(app.file_selected)
            {
                let full_path = std::path::PathBuf::from(path).join(&entry.path);
                let _ = std::process::Command::new("code").arg(full_path).spawn();
            }
            false
        }
        KeyCode::Char('t') => {
            if app.mode == AppMode::DirectorySearcher
                && let Some(entry) = app.file_tree.get(app.file_selected)
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
                crate::utils::terminal::open_terminal(&dir);
            }
            false
        }
        KeyCode::Char('a') => {
            if app.mode == AppMode::DirectorySearcher
                && let Some(entry) = app.file_tree.get(app.file_selected)
            {
                let full_path = std::path::PathBuf::from(path).join(&entry.path);
                let _ = std::process::Command::new("antigravity")
                    .arg(full_path)
                    .spawn();
            } else if app.mode == AppMode::StashDetail
                && let Some(stash) = app.stashes.get(app.stash_selected)
            {
                let msg = apply_stash(path, &stash.id);
                app.refresh_branches(path);
                app.mode = AppMode::Message(msg);
            }
            false
        }
        KeyCode::Down | KeyCode::Char('j') => {
            match app.mode {
                AppMode::Diff => app.info_scroll += 1,
                AppMode::DirectorySearcher => {
                    if app.file_selected < app.file_tree.len().saturating_sub(1) {
                        app.file_selected += 1;
                    }
                }
                AppMode::StashDetail => {
                    if app.stash_selected < app.stashes.len().saturating_sub(1) {
                        app.stash_selected += 1;
                        app.load_stash_detail(path);
                    }
                }
                AppMode::Manage => {
                    if app.manage_selected < 4 {
                        app.manage_selected += 1;
                    }
                }
                AppMode::Filter => {
                    if app.filter_selected < 9 {
                        app.filter_selected += 1;
                    }
                }
                _ => app.next(),
            }
            false
        }
        KeyCode::Up | KeyCode::Char('k') => {
            match app.mode {
                AppMode::Diff => app.info_scroll = app.info_scroll.saturating_sub(1),
                AppMode::DirectorySearcher => {
                    if app.file_selected > 0 {
                        app.file_selected -= 1;
                    }
                }
                AppMode::StashDetail => {
                    if app.stash_selected > 0 {
                        app.stash_selected -= 1;
                        app.load_stash_detail(path);
                    }
                }
                AppMode::Manage => {
                    if app.manage_selected > 0 {
                        app.manage_selected -= 1;
                    }
                }
                AppMode::Filter => {
                    if app.filter_selected > 0 {
                        app.filter_selected -= 1;
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
                    app.filter_selected = digit;
                    handle_enter_or_selection(app, path);
                }
                AppMode::Manage if (1..=5).contains(&digit) => {
                    app.manage_selected = digit - 1;
                    handle_enter_or_selection(app, path);
                }
                _ => {}
            }
            false
        }
        _ => false,
    }
}

fn handle_enter_or_selection(app: &mut App, path: &str) -> bool {
    match app.mode {
        AppMode::Normal if !app.get_filtered_branches().is_empty() => {
            app.mode = AppMode::Manage;
            app.manage_selected = 0;
        }
        AppMode::Manage => {
            let branch_name = {
                let filtered = app.get_filtered_branches();
                filtered.get(app.selected).map(|b| b.name.clone())
            };

            if let Some(name) = branch_name {
                match app.manage_selected {
                    0 => {
                        // Checkout
                        let msg = checkout_branch(path, &name);
                        app.refresh_branches(path);
                        app.current_branch = git::get_current_branch(path);
                        app.mode = AppMode::Message(msg);
                    }
                    1 => {
                        // Diff
                        let mut info = git::get_branch_info(path, &name);
                        let branch = app.get_filtered_branches().get(app.selected).copied();
                        if let Some(crate::models::MergeStatus::SafeLimit(safe, total)) =
                            branch.map(|b| &b.merge_status)
                        {
                            info = format!(
                                "--- CONFLICT DETECTED ---\nSafe commits: {}/{}\n\n{}",
                                safe, total, info
                            );
                        }
                        app.branch_info = info;
                        app.info_scroll = 0;
                        app.mode = AppMode::Diff;
                    }
                    2 => {
                        // Delete
                        let msg = delete_branch(path, &name);
                        app.refresh_branches(path);
                        app.current_branch = git::get_current_branch(path);
                        app.mode = AppMode::Message(msg);
                    }
                    3 => app.mode = AppMode::Help,
                    _ => app.mode = AppMode::Normal,
                }
            } else {
                app.mode = AppMode::Normal;
            }
        }
        AppMode::Filter => {
            app.current_filter = match app.filter_selected {
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
            app.selected = 0;
            app.mode = AppMode::Normal;
        }
        _ => {}
    }
    false
}
