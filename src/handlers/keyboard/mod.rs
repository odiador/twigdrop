pub mod branches;
pub mod commits;
pub mod diff;
pub mod files;
pub mod modals;
pub mod stashes;

pub use diff::update_diff_preview;

use crate::app::App;
use crate::state::ui::{
    AppMode, CommandAction, CommandPaletteState, FilePanel, PrimaryMode,
};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

pub fn handle_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    app.ui.alt_pressed = key.modifiers.contains(KeyModifiers::ALT);
    app.ui.shift_pressed = key.modifiers.contains(KeyModifiers::SHIFT);

    // Global quit
    if key.code == KeyCode::Char('q') && key.kind == KeyEventKind::Press {
        let is_editing = match app.ui.current_mode() {
            AppMode::Settings => app.ui.settings_state.editing || app.ui.settings_state.selecting,
            AppMode::Search
            | AppMode::CreateBranch(_)
            | AppMode::Shell(_)
            | AppMode::DatePicker(_) => true,
            AppMode::InteractiveRebase => app.ui.rebase_state.editing,
            AppMode::CommitAction(_) => app.ui.settings_state.editing,
            _ => false,
        };
        if !is_editing && app.ui.modal_stack.is_empty() {
            if app.ui.current_filter.is_some() {
                app.ui.current_filter = None;
                app.refresh_filtered_branches();
                app.ui.selected_branch_idx = 0;
                return false;
            }
            return true;
        }
    }

    // Shift+Tab App Switcher
    if key.code == KeyCode::BackTab && key.kind == KeyEventKind::Press {
        if *app.ui.current_mode() != AppMode::Switcher {
            app.ui.push_modal(AppMode::Switcher);
            let modes = [
                AppMode::FilesView,
                AppMode::BranchesView,
                AppMode::CommitsView,
                AppMode::Help,
            ];
            app.ui.switcher_index = modes
                .iter()
                .position(|m| {
                    if let AppMode::FilesView = m {
                        app.ui.primary_mode == PrimaryMode::Files
                    } else if let AppMode::BranchesView = m {
                        app.ui.primary_mode == PrimaryMode::Branches
                    } else if let AppMode::CommitsView = m {
                        app.ui.primary_mode == PrimaryMode::Commits
                    } else {
                        false
                    }
                })
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
        KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.ui
                .push_modal(AppMode::CommandPalette(CommandPaletteState {
                    query: String::new(),
                    selected: 0,
                    actions: vec![
                        ("Checkout Branch".to_string(), CommandAction::CheckoutBranch),
                        ("Create Branch".to_string(), CommandAction::CreateBranch),
                        ("Open Settings".to_string(), CommandAction::OpenSettings),
                        (
                            "Interactive Rebase".to_string(),
                            CommandAction::InteractiveRebase,
                        ),
                        ("Stash Changes".to_string(), CommandAction::StashChanges),
                        ("Pop Stash".to_string(), CommandAction::PopStash),
                        ("Fetch from Remote".to_string(), CommandAction::Fetch),
                        ("Pull from Remote".to_string(), CommandAction::Pull),
                        ("Push to Remote".to_string(), CommandAction::Push),
                        ("Commit Changes".to_string(), CommandAction::CommitChanges),
                    ],
                }));
            return false;
        }
        KeyCode::Esc => {
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
            }
            return false;
        }
        KeyCode::Char('?') | KeyCode::Char('h') => {
            app.ui.push_modal(AppMode::Help);
            return false;
        }
        KeyCode::Char('j') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.ui.show_terminal = !app.ui.show_terminal;
            return false;
        }
        KeyCode::Char('d') => {
            app.toggle_primary_mode();
            if app.ui.primary_mode == PrimaryMode::Files {
                app.load_file_tree(path);
                app.ui.track_history(AppMode::FilesView);
            } else {
                app.ui.track_history(AppMode::BranchesView);
            }
            return false;
        }
        KeyCode::Char('C') if app.ui.shift_pressed => {
            app.set_primary_mode(PrimaryMode::Commits);
            app.repo.commit_tree = crate::git::commands::get_commit_tree(path);
            app.ui.selected_commit_idx = 0;
            app.ui.track_history(AppMode::CommitsView);
            return false;
        }
        KeyCode::Char('R') if app.ui.shift_pressed => {
            app.load_rebase_commits(path);
            app.ui.push_modal(AppMode::InteractiveRebase);
            return false;
        }
        KeyCode::Char('S') if app.ui.shift_pressed => {
            app.load_stashes(path);
            app.load_stash_detail(path);
            app.ui.push_modal(AppMode::StashDetail);
            return false;
        }
        KeyCode::Char('c') => {
            app.ui.push_modal(AppMode::CreateBranch(String::new()));
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
