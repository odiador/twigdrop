pub mod branches;
pub mod commits;
pub mod diff;
pub mod files;
pub mod inspector;
pub mod modals;
pub mod pull_requests;
pub mod sidebar;
pub mod stashes;

pub use branches::render_main_list;
pub use commits::{
    render_commit_action, render_commit_files, render_commits, render_interactive_rebase,
};
pub use diff::render_diff;
pub use files::{render_code_preview, render_directory_searcher};
pub use modals::{
    render_command_palette, render_confirm_delete, render_create_branch, render_date_picker,
    render_filter, render_help, render_main_menu, render_manage, render_message,
    render_quick_actions, render_rename_branch, render_settings, render_shell, render_switcher,
};
pub use pull_requests::render_pull_requests;
pub use stashes::render_stash_detail;
