pub mod branch_ops;
pub mod code_preview;
pub mod command_palette;
pub mod date_picker;
pub mod filter;
pub mod folder_profile;
pub mod help;
pub mod main_menu;
pub mod manage;
pub mod message;
pub mod pr_review;
pub mod quick_actions;
pub mod rebase;
pub mod settings;
pub mod shell;
pub mod switcher;

pub use date_picker::handle_date_picker_keyboard;
pub use filter::handle_filter_keyboard;
pub use manage::handle_manage_keyboard;
pub use settings::handle_settings_keyboard;

use crate::app::App;
use crate::state::ui::AppMode;
use ratatui::crossterm::event::KeyEvent;

pub fn handle_modal_keyboard(app: &mut App, key: KeyEvent, path: &str) -> Option<bool> {
    let current_mode = app.ui.current_mode().clone();
    match current_mode {
        AppMode::Normal => None,
        AppMode::Switcher => switcher::handle(app, key, path),
        AppMode::Message(_) => message::handle(app, key),
        AppMode::Settings => settings::handle(app, key),
        AppMode::Filter => filter::handle(app, key),
        AppMode::Manage => manage::handle(app, key, path),
        AppMode::ConfirmDelete(names) => branch_ops::handle_confirm_delete(app, key, names),
        AppMode::DatePicker(state) => date_picker::handle(app, key, state, path),
        AppMode::CommitFiles(hash, files) => Some(super::commits::handle_commit_files_keyboard(
            app, key, &hash, &files, path,
        )),
        AppMode::CommitAction(hash) => Some(super::commits::handle_commit_action_keyboard(
            app, key, &hash, path,
        )),
        AppMode::Shell(input) => shell::handle(app, key, path, input),
        AppMode::MainMenu => main_menu::handle(app, key),
        AppMode::QuickActions => quick_actions::handle(app, key, path),
        AppMode::InteractiveRebase => rebase::handle(app, key, path),
        AppMode::CreateBranch(input) => branch_ops::handle_create_branch(app, key, path, input),
        AppMode::RenameBranch(old, input) => {
            branch_ops::handle_rename_branch(app, key, path, old, input)
        }
        AppMode::CodePreview(state) => code_preview::handle(app, key, state),
        AppMode::CommandPalette(state) => command_palette::handle(app, key, path, state),
        AppMode::FolderProfile(state) => folder_profile::handle(app, state, key, path),
        AppMode::PrReview(state) => pr_review::handle(app, key, state, path),
        AppMode::Diff => Some(super::diff::handle_diff_keyboard(app, key, path)),
        AppMode::Help => help::handle(app, key),
    }
}
