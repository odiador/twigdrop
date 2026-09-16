use crate::app::App;
use crate::git;
use crate::state::ui::{
    AppMode, CommandAction, DatePickerField, DatePickerState, FilePanel, PreviewState,
    PrimaryMode, RebaseAction,
};
use crate::ui::animations::SnapAnimation;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_date_picker_keyboard(
    app: &mut App,
    key: KeyEvent,
    state: &mut DatePickerState,
    path: &str,
) -> bool {
    match key.code {
        KeyCode::Left | KeyCode::BackTab => {
            state.active_field = match state.active_field {
                DatePickerField::Year => DatePickerField::Minute,
                DatePickerField::Month => DatePickerField::Year,
                DatePickerField::Day => DatePickerField::Month,
                DatePickerField::Hour => DatePickerField::Day,
                DatePickerField::Minute => DatePickerField::Hour,
            };
        }
        KeyCode::Right | KeyCode::Tab => {
            state.active_field = match state.active_field {
                DatePickerField::Year => DatePickerField::Month,
                DatePickerField::Month => DatePickerField::Day,
                DatePickerField::Day => DatePickerField::Hour,
                DatePickerField::Hour => DatePickerField::Minute,
                DatePickerField::Minute => DatePickerField::Year,
            };
        }
        KeyCode::Up | KeyCode::Char('+') | KeyCode::Char('k') => match state.active_field {
            DatePickerField::Year => state.year -= 1,
            DatePickerField::Month => {
                state.month = if state.month == 1 {
                    12
                } else {
                    state.month - 1
                };
                let max_days = crate::utils::days_in_month(state.month, state.year);
                if state.day > max_days {
                    state.day = max_days;
                }
            }
            DatePickerField::Day => {
                let max_days = crate::utils::days_in_month(state.month, state.year);
                state.day = if state.day == 1 {
                    max_days
                } else {
                    state.day - 1
                };
            }
            DatePickerField::Hour => state.hour = if state.hour == 0 { 23 } else { state.hour - 1 },
            DatePickerField::Minute => {
                state.minute = if state.minute == 0 {
                    59
                } else {
                    state.minute - 1
                }
            }
        },
        KeyCode::Down | KeyCode::Char('-') | KeyCode::Char('j') => match state.active_field {
            DatePickerField::Year => state.year += 1,
            DatePickerField::Month => {
                state.month = (state.month % 12) + 1;
                let max_days = crate::utils::days_in_month(state.month, state.year);
                if state.day > max_days {
                    state.day = max_days;
                }
            }
            DatePickerField::Day => {
                let max_days = crate::utils::days_in_month(state.month, state.year);
                state.day = (state.day % max_days) + 1;
            }
            DatePickerField::Hour => state.hour = (state.hour + 1) % 24,
            DatePickerField::Minute => state.minute = (state.minute + 1) % 60,
        },
        KeyCode::Enter => {
            let new_date = format!(
                "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
                state.year, state.month, state.day, state.hour, state.minute, state.second
            );

            if let Some(AppMode::CommitAction(hash)) =
                app.ui.modal_stack.iter().rev().nth(1).map(|m| &m.mode)
            {
                let short_hash = hash.clone();
                let full_hash = crate::git::commands::run_git(path, &["rev-parse", &short_hash])
                    .unwrap_or(short_hash.clone());

                let exec_cmd = format!(
                    "if [ \"$(git rev-parse HEAD)\" = \"{}\" ]; then GIT_COMMITTER_DATE=\"{}\" git commit --amend --no-edit --date=\"{}\"; fi",
                    full_hash, new_date, new_date
                );

                let msg = match crate::git::commands::run_git(
                    path,
                    &[
                        "rebase",
                        "--autostash",
                        &format!("{}^", short_hash),
                        "--exec",
                        &exec_cmd,
                    ],
                ) {
                    Ok(m) => {
                        app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
                        format!("Date updated to {}.\n{}", new_date, m)
                    }
                    Err(e) => format!("Failed to update date: {}", e),
                };
                app.ui.pop_modal();
                app.ui.pop_modal();
                app.ui.push_modal(AppMode::Message(msg));
            } else {
                app.ui.pop_modal();
            }
            return false;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
            return false;
        }
        _ => {}
    }
    false
}

pub fn handle_search_keyboard(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Enter | KeyCode::Esc => {
            app.ui.pop_modal();
        }
        KeyCode::Char(c) => {
            app.ui.search_query.push(c);
            app.refresh_filtered_branches();
        }
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

pub fn handle_settings_keyboard(app: &mut App, key: KeyEvent) -> bool {
    if app.ui.settings_state.editing {
        match key.code {
            KeyCode::Enter => {
                match app.ui.settings_state.selected {
                    0 => app.config.ide_command = app.ui.settings_state.input.clone(),
                    1 => app.config.alternative_ide_command = app.ui.settings_state.input.clone(),
                    2 => app.config.ai_provider = app.ui.settings_state.input.clone(),
                    3 => {
                        app.config.current_provider_mut().model =
                            app.ui.settings_state.input.clone()
                    }
                    4 => {
                        app.config.current_provider_mut().api_key =
                            crate::utils::config::obfuscate(&app.ui.settings_state.input)
                    }
                    5 => {
                        app.config.current_provider_mut().url = app.ui.settings_state.input.clone()
                    }
                    7 => {
                        app.config.default_sidebar_width =
                            app.ui.settings_state.input.parse().unwrap_or(30)
                    }
                    _ => {}
                }
                app.ui.settings_state.editing = false;
                crate::utils::config::save_config(&app.config);
            }
            KeyCode::Esc => {
                app.ui.settings_state.editing = false;
            }
            KeyCode::Char(c) => {
                app.ui.settings_state.input.push(c);
            }
            KeyCode::Backspace => {
                app.ui.settings_state.input.pop();
            }
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
                    app.ui.settings_state.choice_idx =
                        app.ui.settings_state.choices.len().saturating_sub(1);
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
                if let Some(choice) = app
                    .ui
                    .settings_state
                    .choices
                    .get(app.ui.settings_state.choice_idx)
                {
                    match app.ui.settings_state.selected {
                        2 => app.config.ai_provider = choice.clone(),
                        3 => app.config.current_provider_mut().model = choice.clone(),
                        _ => {}
                    }
                }
                app.ui.settings_state.selecting = false;
                crate::utils::config::save_config(&app.config);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui.settings_state.selecting = false;
            }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.ui.settings_state.selected > 0 => {
            app.ui.settings_state.selected -= 1;
        }
        KeyCode::Down | KeyCode::Char('j') if app.ui.settings_state.selected < 8 => {
            app.ui.settings_state.selected += 1;
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            match app.ui.settings_state.selected {
                2 => {
                    app.ui.settings_state.selecting = true;
                    app.ui.settings_state.choices = crate::utils::config::PROVIDER_ARCHETYPES
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    app.ui.settings_state.choice_idx = app
                        .ui
                        .settings_state
                        .choices
                        .iter()
                        .position(|s| s == &app.config.ai_provider)
                        .unwrap_or(0);
                }
                3 => {
                    app.ui.settings_state.selecting = true;
                    app.ui.settings_state.choices = match app.config.ai_provider.as_str() {
                        "openai" => crate::utils::config::OPENAI_MODELS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "anthropic" => crate::utils::config::ANTHROPIC_MODELS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "google" => crate::utils::config::GOOGLE_MODELS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        _ => vec!["Loading models...".to_string()],
                    };
                    app.ui.settings_state.choice_idx = app
                        .ui
                        .settings_state
                        .choices
                        .iter()
                        .position(|s| s == &app.config.current_provider().model)
                        .unwrap_or(0);
                }
                6 => {
                    app.config.enable_animations = !app.config.enable_animations;
                    crate::utils::config::save_config(&app.config);
                }
                8 => {
                    crate::utils::config::save_config(&app.config);
                    app.ui.pop_modal();
                }
                _ => {
                    app.ui.settings_state.editing = true;
                    app.ui.settings_state.input = match app.ui.settings_state.selected {
                        0 => app.config.ide_command.clone(),
                        1 => app.config.alternative_ide_command.clone(),
                        4 => crate::utils::config::deobfuscate(
                            &app.config.current_provider().api_key,
                        ),
                        5 => app.config.current_provider().url.clone(),
                        7 => app.config.default_sidebar_width.to_string(),
                        _ => String::new(),
                    };
                }
            }
        }
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::BackTab => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    false
}

pub fn handle_filter_keyboard(app: &mut App, _key: KeyEvent) -> bool {
    match _key.code {
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
            app.ui.pop_modal();
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    false
}

pub fn handle_manage_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    const MANAGE_OPTIONS_COUNT: usize = 8;
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.ui.manage_selected = app.ui.manage_selected.saturating_sub(1);
        }
        KeyCode::Down | KeyCode::Char('j') if app.ui.manage_selected < MANAGE_OPTIONS_COUNT - 1 => {
            app.ui.manage_selected += 1;
        }
        KeyCode::Enter => {
            let branch = app
                .get_filtered_branches()
                .get(app.ui.selected_branch_idx)
                .cloned();
            if let Some(b) = branch {
                match app.ui.manage_selected {
                    0 => {
                        let msg = crate::git::commands::run_git(path, &["checkout", &b.name])
                            .unwrap_or_else(|e| e.to_string());
                        app.refresh_branches(path);
                        app.repo.current_branch = git::get_current_branch(path);
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    1 => {
                        let branch_name = b.name.clone();
                        let _ = app.ai_trigger_tx.try_send((
                            "analyze".to_string(),
                            path.to_string(),
                            branch_name.clone(),
                        ));
                        app.ai_state.ai_analysis = Some("Initializing AI analysis...".to_string());
                        app.repo.branch_info = git::get_branch_info(path, &branch_name);
                        app.ui.info_scroll = 0;

                        app.repo.diff_files = git::get_branch_diff_files(path, &branch_name);
                        app.ui.diff_file_selected = 0;
                        app.repo.diff_preview = None;
                        if !app.repo.diff_files.is_empty() {
                            let first_file = app.repo.diff_files[0].clone();
                            let diff_content =
                                git::get_branch_file_diff(path, &branch_name, &first_file);
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

                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Diff);
                    }
                    2 => {
                        let names = vec![b.name.clone()];
                        if b.status
                            .contains(&crate::models::BranchStatus::HasUniqueCommits)
                        {
                            app.ui.pop_modal();
                            app.ui.push_modal(AppMode::ConfirmDelete(names));
                        } else {
                            app.ui.snap_animation = Some(SnapAnimation::new(names));
                            app.ui.pop_modal();
                        }
                    }
                    3 => {
                        app.ui.pop_modal();
                        app.ui
                            .push_modal(AppMode::Message("Rename branch coming soon!".to_string()));
                    }
                    4 => {
                        let msg = crate::git::commands::run_git(
                            path,
                            &[
                                "stash",
                                "push",
                                "-m",
                                &format!("Stash from Twigdrop: {}", b.name),
                            ],
                        )
                        .unwrap_or_else(|e| e.to_string());
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                    5 => {
                        app.set_primary_mode(PrimaryMode::Commits);
                        app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
                        app.ui.selected_commit_idx = 0;
                        app.ui.pop_modal();
                        app.ui.track_history(AppMode::CommitsView);
                    }
                    6 => {
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Help);
                    }
                    _ => {
                        app.ui.pop_modal();
                    }
                }
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    false
}

pub fn handle_modal_keyboard(app: &mut App, key: KeyEvent, path: &str) -> Option<bool> {
    let current_mode = app.ui.current_mode().clone();
    match current_mode {
        AppMode::Switcher => {
            let modes = [
                AppMode::FilesView,
                AppMode::BranchesView,
                AppMode::CommitsView,
                AppMode::Help,
            ];
            match key.code {
                KeyCode::Enter => {
                    let selected_mode = modes[app.ui.switcher_index].clone();
                    app.ui.pop_modal();
                    app.ui.modal_stack.clear();
                    match selected_mode {
                        AppMode::BranchesView => {
                            app.set_primary_mode(PrimaryMode::Branches);
                            app.ui.track_history(AppMode::BranchesView);
                        }
                        AppMode::FilesView => {
                            app.set_primary_mode(PrimaryMode::Files);
                            app.ui.track_history(AppMode::FilesView);
                        }
                        AppMode::CommitsView => {
                            app.set_primary_mode(PrimaryMode::Commits);
                            app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
                            app.ui.selected_commit_idx = 0;
                            app.ui.track_history(AppMode::CommitsView);
                        }
                        AppMode::Help => {
                            app.ui.push_modal(AppMode::Help);
                        }
                        _ => {}
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.ui.pop_modal();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    app.ui.switcher_index = app.ui.switcher_index.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') if app.ui.switcher_index + 1 < modes.len() => {
                    app.ui.switcher_index += 1;
                }
                _ => {}
            }
            Some(false)
        }
        AppMode::Message(_) => {
            app.ui.pop_modal();
            Some(false)
        }
        AppMode::Settings => Some(handle_settings_keyboard(app, key)),
        AppMode::Search => Some(handle_search_keyboard(app, key)),
        AppMode::Filter => Some(handle_filter_keyboard(app, key)),
        AppMode::Manage => Some(handle_manage_keyboard(app, key, path)),
        AppMode::ConfirmDelete(names) => {
            match key.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => {
                    app.ui.snap_animation = Some(SnapAnimation::new(names));
                    app.ui.pop_modal();
                }
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                    app.ui.pop_modal();
                }
                _ => {}
            }
            Some(false)
        }
        AppMode::CommitsView => Some(super::commits::handle_commits_keyboard(app, key, path)),
        AppMode::DatePicker(mut state) => {
            let res = handle_date_picker_keyboard(app, key, &mut state, path);
            if let AppMode::DatePicker(current_state) = app.ui.current_mode_mut() {
                *current_state = state;
            }
            Some(res)
        }
        AppMode::CommitFiles(hash, files) => {
            Some(super::commits::handle_commit_files_keyboard(app, key, &hash, &files, path))
        }
        AppMode::CommitAction(hash) => {
            Some(super::commits::handle_commit_action_keyboard(app, key, &hash, path))
        }
        AppMode::Shell(input) => {
            match key.code {
                KeyCode::Enter => {
                    if !input.is_empty() {
                        let parts: Vec<&str> = input.split_whitespace().collect();
                        let msg = match crate::git::commands::run_git(path, &parts) {
                            Ok(m) => format!("$ {}\n{}", input, m),
                            Err(e) => format!("Error: {}", e),
                        };
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    } else {
                        app.ui.pop_modal();
                    }
                }
                KeyCode::Esc => {
                    app.ui.pop_modal();
                }
                KeyCode::Char(c) => {
                    if let AppMode::Shell(inp) = app.ui.current_mode_mut() {
                        inp.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if let AppMode::Shell(inp) = app.ui.current_mode_mut() {
                        inp.pop();
                    }
                }
                _ => {}
            }
            Some(false)
        }
        AppMode::MainMenu => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') if app.ui.main_menu_state.selected > 0 => {
                    app.ui.main_menu_state.selected -= 1;
                }
                KeyCode::Down | KeyCode::Char('j') if app.ui.main_menu_state.selected < 2 => {
                    app.ui.main_menu_state.selected += 1;
                }
                KeyCode::Enter => match app.ui.main_menu_state.selected {
                    0 => {
                        app.ui.push_modal(AppMode::Settings);
                        app.ui.settings_state.selected = 0;
                        app.ui.settings_state.editing = false;
                    }
                    1 => app.ui.push_modal(AppMode::Help),
                    2 => return Some(true),
                    _ => {}
                },
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.ui.pop_modal();
                }
                _ => {}
            }
            Some(false)
        }
        AppMode::QuickActions => {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') if app.ui.quick_actions_state.selected > 0 => {
                    app.ui.quick_actions_state.selected -= 1;
                }
                KeyCode::Down | KeyCode::Char('j')
                    if app.ui.quick_actions_state.selected
                        < app.ui.quick_actions_state.actions.len().saturating_sub(1) =>
                {
                    app.ui.quick_actions_state.selected += 1;
                }
                KeyCode::Enter => {
                    let action = app.ui.quick_actions_state.actions
                        [app.ui.quick_actions_state.selected]
                        .clone();
                    let parts: Vec<&str> = action.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0] == "git" {
                        let args = &parts[1..];
                        let msg = match crate::git::commands::run_git(path, args) {
                            Ok(m) => format!("> {}\n{}", action, m),
                            Err(e) => format!("Error: {}", e),
                        };
                        app.refresh_branches(path);
                        app.ui.pop_modal();
                        app.ui.push_modal(AppMode::Message(msg));
                    }
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    app.ui.pop_modal();
                }
                _ => {}
            }
            Some(false)
        }
        AppMode::InteractiveRebase => {
            if app.ui.rebase_state.editing {
                match key.code {
                    KeyCode::Enter => {
                        app.ui.rebase_state.editing = false;
                        let i = app.ui.rebase_state.selected;
                        app.ui.rebase_state.commits[i].new_message =
                            Some(app.ui.rebase_state.input.clone());
                        app.ui.rebase_state.commits[i].action = RebaseAction::Reword;
                    }
                    KeyCode::Esc => {
                        app.ui.rebase_state.editing = false;
                    }
                    KeyCode::Char(c) => {
                        app.ui.rebase_state.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.ui.rebase_state.input.pop();
                    }
                    _ => {}
                }
            } else {
                match key.code {
                    KeyCode::Up | KeyCode::Char('k') if app.ui.rebase_state.selected > 0 => {
                        app.ui.rebase_state.selected -= 1;
                    }
                    KeyCode::Down | KeyCode::Char('j')
                        if app.ui.rebase_state.selected
                            < app.ui.rebase_state.commits.len().saturating_sub(1) =>
                    {
                        app.ui.rebase_state.selected += 1;
                    }
                    KeyCode::Char('r') => {
                        app.ui.rebase_state.commits[app.ui.rebase_state.selected].action =
                            RebaseAction::Reword;
                    }
                    KeyCode::Char('d') => {
                        app.ui.rebase_state.commits[app.ui.rebase_state.selected].action =
                            RebaseAction::Drop;
                    }
                    KeyCode::Char('s') => {
                        app.ui.rebase_state.commits[app.ui.rebase_state.selected].action =
                            RebaseAction::Squash;
                    }
                    KeyCode::Char('p') => {
                        app.ui.rebase_state.commits[app.ui.rebase_state.selected].action =
                            RebaseAction::Pick;
                    }
                    KeyCode::Enter => {
                        app.ui.rebase_state.editing = true;
                        let i = app.ui.rebase_state.selected;
                        app.ui.rebase_state.input = app.ui.rebase_state.commits[i]
                            .new_message
                            .clone()
                            .unwrap_or_else(|| {
                                app.ui.rebase_state.commits[i].original_message.clone()
                            });
                    }
                    KeyCode::Char('A') => {
                        app.ui.rebase_state.ai_analyzing = true;
                        let i = app.ui.rebase_state.selected;
                        let hash = app.ui.rebase_state.commits[i].hash.clone();
                        let _ = app.ai_trigger_tx.try_send((
                            "rename_commit".to_string(),
                            path.to_string(),
                            hash,
                        ));
                    }
                    KeyCode::Char('E') => {
                        crate::actions::commands::execute_interactive_rebase(
                            path,
                            &app.ui.rebase_state.commits,
                        );
                        app.ui.pop_modal();
                        app.ui
                            .push_modal(AppMode::Message("Rebase executed.".to_string()));
                    }
                    KeyCode::Esc | KeyCode::Char('q') => {
                        app.ui.pop_modal();
                    }
                    _ => {}
                }
            }
            Some(false)
        }
        AppMode::CreateBranch(input) => {
            match key.code {
                KeyCode::Enter => {
                    if !input.is_empty() {
                        match crate::git::commands::run_git(path, &["checkout", "-b", &input]) {
                            Ok(_) => {
                                app.refresh_branches(path);
                                app.repo.current_branch = git::get_current_branch(path);
                                app.ui.pop_modal();
                                app.ui
                                    .push_modal(AppMode::Message(format!("Created {}", input)));
                            }
                            Err(e) => {
                                app.ui.pop_modal();
                                app.ui.push_modal(AppMode::Message(format!("Error: {}", e)));
                            }
                        }
                    } else {
                        app.ui.pop_modal();
                    }
                }
                KeyCode::Esc => {
                    app.ui.pop_modal();
                }
                KeyCode::Char(c) => {
                    if let AppMode::CreateBranch(inp) = app.ui.current_mode_mut() {
                        inp.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if let AppMode::CreateBranch(inp) = app.ui.current_mode_mut() {
                        inp.pop();
                    }
                }
                _ => {}
            }
            Some(false)
        }
        AppMode::CodePreview(mut state) => {
            if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') {
                app.ui.pop_modal();
                app.ui.active_panel = FilePanel::Directory;
            } else if app.ui.active_panel == FilePanel::Preview {
                let res = super::files::handle_preview_keyboard(&mut state, key.code);
                if let AppMode::CodePreview(s) = app.ui.current_mode_mut() {
                    *s = state;
                }
                return Some(res);
            }
            Some(false)
        }
        AppMode::CommandPalette(mut state) => {
            let visible_actions: Vec<_> = state
                .actions
                .iter()
                .filter(|(name, _)| {
                    state.query.is_empty()
                        || name.to_lowercase().contains(&state.query.to_lowercase())
                })
                .cloned()
                .collect();

            match key.code {
                KeyCode::Esc => {
                    app.ui.pop_modal();
                }
                KeyCode::Down if state.selected + 1 < visible_actions.len() => {
                    state.selected += 1;
                }
                KeyCode::Up if state.selected > 0 => {
                    state.selected -= 1;
                }
                KeyCode::Char(c) => {
                    state.query.push(c);
                    state.selected = 0;
                }
                KeyCode::Backspace => {
                    state.query.pop();
                    state.selected = 0;
                }
                KeyCode::Enter => {
                    if let Some((_, action)) = visible_actions.get(state.selected) {
                        let action = action.clone();
                        app.ui.pop_modal();

                        match action {
                            CommandAction::CreateBranch => {
                                app.ui.push_modal(AppMode::CreateBranch(String::new()));
                            }
                            CommandAction::CheckoutBranch => {
                                app.ui.push_modal(AppMode::Filter);
                                app.ui.filter_selected = 0;
                            }
                            CommandAction::OpenSettings => {
                                app.ui.push_modal(AppMode::Settings);
                            }
                            CommandAction::InteractiveRebase => {
                                app.ui.push_modal(AppMode::InteractiveRebase);
                                app.load_rebase_commits(path);
                            }
                            CommandAction::StashChanges => {
                                app.ui.push_modal(AppMode::Shell("git stash".to_string()));
                            }
                            CommandAction::PopStash => {
                                app.ui
                                    .push_modal(AppMode::Shell("git stash pop".to_string()));
                            }
                            CommandAction::Fetch => {
                                app.ui.push_modal(AppMode::Shell("git fetch".to_string()));
                            }
                            CommandAction::Pull => {
                                app.ui.push_modal(AppMode::Shell("git pull".to_string()));
                            }
                            CommandAction::Push => {
                                app.ui.push_modal(AppMode::Shell("git push".to_string()));
                            }
                            CommandAction::CommitChanges => {
                                app.ui
                                    .push_modal(AppMode::Shell("git commit -m \"\"".to_string()));
                            }
                        }
                    } else {
                        app.ui.pop_modal();
                    }
                    return Some(false);
                }
                _ => {}
            }
            if let AppMode::CommandPalette(s) = app.ui.current_mode_mut() {
                *s = state;
            }
            Some(false)
        }
        AppMode::Diff => {
            Some(super::diff::handle_diff_keyboard(app, key, path))
        }
        AppMode::Help if key.code == KeyCode::Esc || key.code == KeyCode::Char('q') => {
            app.ui.pop_modal();
            Some(false)
        }
        AppMode::StashDetail => {
            Some(super::stashes::handle_stashes_keyboard(app, key, path))
        }
        _ => None,
    }
}
