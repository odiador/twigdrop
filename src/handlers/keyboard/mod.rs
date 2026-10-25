pub mod branches;
pub mod commits;
pub mod diff;
pub mod files;
pub mod modals;
pub mod stashes;

pub use diff::update_diff_preview;

use crate::app::App;
use crate::state::ui::{
    AppMode, CommandAction, CommandPaletteState, FilePanel, PrimaryMode, SwitchTarget, View,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn handle_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    // NOTE: modifiers are matched per-event below (see `key.modifiers`).
    // Do NOT store them in UiState (unreliable in tmux/SSH; see docs/ARCHITECTURE.md §4).

    // Global quit (Ctrl+Q)
    if (key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q'))
        && key.modifiers.contains(KeyModifiers::CONTROL)
        && key.kind == KeyEventKind::Press
    {
        return true;
    }

    // Shift+Tab App Switcher
    if key.code == KeyCode::BackTab && key.kind == KeyEventKind::Press {
        if *app.ui.current_mode() != AppMode::Switcher {
            app.ui.push_modal(AppMode::Switcher);
            let current = SwitchTarget::View(View::from_primary(app.ui.primary_mode));
            app.ui.switcher_index = app
                .ui
                .switcher_order()
                .iter()
                .position(|t| *t == current)
                .unwrap_or(0);
        }
        return false;
    }

    if key.kind == KeyEventKind::Release {
        return false;
    }

    // 1. Check if an active modal handles this key
    if let Some(res) = modals::handle_modal_keyboard(app, key, path) {
        return res;
    }

    // 2. Global Shortcuts across normal views
    match key.code {
        KeyCode::Char('k') | KeyCode::Char('K')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            let loc = app.locale();
            // In-progress rebase/merge actions go first so they are easy to reach.
            let mut actions = Vec::new();
            if crate::git::status::is_rebasing(path) {
                actions.push((
                    loc.modals.cmd_continue_rebase.to_string(),
                    CommandAction::ContinueRebase,
                ));
                actions.push((
                    loc.modals.cmd_abort_rebase.to_string(),
                    CommandAction::AbortRebase,
                ));
            }
            if crate::git::status::is_merging(path) {
                actions.push((
                    loc.modals.cmd_abort_merge.to_string(),
                    CommandAction::AbortMerge,
                ));
            }
            app.ui
                .push_modal(AppMode::CommandPalette(CommandPaletteState {
                    query: String::new(),
                    selected: 0,
                    actions: actions
                        .into_iter()
                        .chain([
                            (
                                loc.modals.cmd_checkout.to_string(),
                                CommandAction::CheckoutBranch,
                            ),
                            (
                                loc.modals.cmd_create_branch.to_string(),
                                CommandAction::CreateBranch,
                            ),
                            (
                                loc.modals.cmd_settings.to_string(),
                                CommandAction::OpenSettings,
                            ),
                            (
                                loc.modals.cmd_rebase.to_string(),
                                CommandAction::InteractiveRebase,
                            ),
                            (
                                loc.modals.cmd_stash.to_string(),
                                CommandAction::StashChanges,
                            ),
                            (
                                loc.modals.cmd_pop_stash.to_string(),
                                CommandAction::PopStash,
                            ),
                            (loc.modals.cmd_fetch.to_string(), CommandAction::Fetch),
                            (loc.modals.cmd_pull.to_string(), CommandAction::Pull),
                            (loc.modals.cmd_push.to_string(), CommandAction::Push),
                            (
                                loc.modals.cmd_commit.to_string(),
                                CommandAction::CommitChanges,
                            ),
                        ])
                        .collect(),
                }));
            return false;
        }
        KeyCode::Esc => {
            if !app.ui.search_query.is_empty() {
                app.ui.search_query.clear();
                app.refresh_filtered_branches();
                app.ui.selected_branch_idx = 0;
                return false;
            }
            if !app.ui.files_search_query.is_empty() {
                app.ui.files_search_query.clear();
                app.refresh_filtered_files();
                return false;
            }
            if app.ui.files_changed_only && app.ui.primary_mode == PrimaryMode::Files {
                app.ui.files_changed_only = false;
                app.load_file_tree(path);
                app.ui.notify(
                    crate::state::toast::ToastKind::Info,
                    "Filter: All files".to_string(),
                    None,
                );
                return false;
            }
            if app.ui.branches_active_only && app.ui.primary_mode == PrimaryMode::Branches {
                app.ui.branches_active_only = false;
                app.refresh_filtered_branches();
                app.ui.notify(
                    crate::state::toast::ToastKind::Info,
                    "Filter: All branches".to_string(),
                    None,
                );
                return false;
            }
            if app.ui.commit_panel == crate::state::ui::CommitPanel::Files
                && app.ui.primary_mode == PrimaryMode::Commits
            {
                app.ui.commit_panel = crate::state::ui::CommitPanel::Tree;
                return false;
            }
            if app.ui.show_inspector_drawer || app.ui.show_nav_sidebar {
                app.ui.show_inspector_drawer = false;
                app.ui.show_nav_sidebar = false;
                return false;
            }
            if *app.ui.current_mode() == AppMode::MainMenu {
                app.ui.pop_modal();
                return false;
            }
            app.ui.push_modal(AppMode::MainMenu);
            app.ui.main_menu_state.selected = 0;
            return false;
        }
        KeyCode::Tab => {
            let mode = app.ui.current_mode().clone();
            if let AppMode::CodePreview(_) = mode {
                app.ui.active_panel = match app.ui.active_panel {
                    FilePanel::Directory => FilePanel::Preview,
                    FilePanel::Preview => FilePanel::Directory,
                };
            } else if mode == AppMode::Diff {
                app.ui.diff_panel = match app.ui.diff_panel {
                    FilePanel::Directory => FilePanel::Preview,
                    FilePanel::Preview => FilePanel::Directory,
                };
            } else if app.ui.primary_mode == PrimaryMode::Commits {
                app.ui.commit_panel = match app.ui.commit_panel {
                    crate::state::ui::CommitPanel::Tree => crate::state::ui::CommitPanel::Files,
                    crate::state::ui::CommitPanel::Files => crate::state::ui::CommitPanel::Tree,
                };
            } else if app.ui.primary_mode == PrimaryMode::Branches {
                app.ui.branches_active_only = !app.ui.branches_active_only;
                app.refresh_filtered_branches();
                let label = if app.ui.branches_active_only {
                    "Active & divergent branches only"
                } else {
                    "All branches"
                };
                app.ui.notify(
                    crate::state::toast::ToastKind::Info,
                    format!("Filter: {label}"),
                    None,
                );
            }
            return false;
        }
        KeyCode::Char('?') => {
            app.ui.push_modal(AppMode::Help);
            return false;
        }
        KeyCode::Char('h') | KeyCode::Char('H')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.ui.push_modal(AppMode::Help);
            return false;
        }
        KeyCode::Char('c') | KeyCode::Char('C')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.ui.push_modal(AppMode::CreateBranch(String::new()));
            return false;
        }
        KeyCode::Char('f') | KeyCode::Char('F')
            if key.modifiers.contains(KeyModifiers::CONTROL) =>
        {
            app.ui.push_modal(AppMode::Filter);
            app.ui.filter_selected = 0;
            return false;
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.ui.show_terminal = !app.ui.show_terminal;
            return false;
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.toggle_primary_mode();
            match app.ui.primary_mode {
                PrimaryMode::Branches => {}
                PrimaryMode::Files => app.load_file_tree(path),
                PrimaryMode::Commits => {
                    app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
                    app.ui.selected_commit_idx = 0;
                    app.fetch_current_commit_details(path);
                }
                PrimaryMode::Stashes => {
                    app.load_stashes(path);
                    app.load_stash_detail(path);
                    app.ui.selected_stash_idx = 0;
                }
            }
            app.ui.track_history(app.ui.view());
            return false;
        }
        // Shift-letter layer: crossterm delivers these as uppercase Char + SHIFT,
        // but some terminals strip the flag — accept bare uppercase too
        // (mirrors `crate::input::from_key`). Lowercase variants stay reserved
        // for type-to-filter.
        KeyCode::Char('B') => {
            app.set_primary_mode(PrimaryMode::Branches);
            app.ui.track_history(View::Branches);
            return false;
        }
        KeyCode::Char('F') => {
            app.set_primary_mode(PrimaryMode::Files);
            app.load_file_tree(path);
            app.ui.track_history(View::Files);
            return false;
        }
        KeyCode::Char('C') => {
            app.set_primary_mode(PrimaryMode::Commits);
            app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
            app.ui.selected_commit_idx = 0;
            app.fetch_current_commit_details(path);
            app.ui.track_history(View::Commits);
            return false;
        }
        KeyCode::Char('R') => {
            app.load_rebase_commits(path);
            app.ui.push_modal(AppMode::InteractiveRebase);
            return false;
        }
        KeyCode::Char('S') => {
            app.set_primary_mode(PrimaryMode::Stashes);
            app.load_stashes(path);
            app.load_stash_detail(path);
            app.ui.selected_stash_idx = 0;
            app.ui.track_history(View::Stashes);
            return false;
        }

        KeyCode::Char('!') => {
            app.ui.push_modal(AppMode::Shell(String::new()));
            return false;
        }
        KeyCode::Char(':') => {
            app.ui.push_modal(AppMode::QuickActions);
            app.ui.quick_actions_state.selected = 0;
            return false;
        }
        _ => {}
    }

    // 3. Delegate to PrimaryMode domain handlers
    match app.ui.primary_mode {
        PrimaryMode::Branches => branches::handle_branches_keyboard(app, key, path),
        PrimaryMode::Files => files::handle_files_keyboard(app, key, path),
        PrimaryMode::Commits => commits::handle_commits_keyboard(app, key, path),
        PrimaryMode::Stashes => stashes::handle_stashes_keyboard(app, key, path),
    }
}
