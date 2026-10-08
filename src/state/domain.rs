use crate::git::files::{FileEntry, FileStatus};
use crate::git::stash::StashEntry;
use crate::models::{Branch, CommitTreeItem};
use crate::state::ui::PreviewState;
use std::collections::HashMap;

pub struct RepositoryState {
    pub branches: Vec<Branch>,
    pub current_branch: String,
    pub git_file_statuses: HashMap<String, FileStatus>,
    pub file_tree: Vec<FileEntry>,
    pub stashes: Vec<StashEntry>,
    pub stash_files: Vec<String>,
    pub stash_diff: String,
    pub branch_info: String,
    pub diff_files: Vec<String>,
    pub diff_preview: Option<PreviewState>,
    pub commit_tree: Vec<CommitTreeItem>,
    pub commit_diff: Option<String>,
    pub commit_stats: Option<String>,
    /// Inspector drawer cache: recent commits of the selected branch.
    /// Refreshed in update (`App::refresh_inspector_commits`), never in render.
    pub inspector_branch: String,
    pub inspector_commits: Vec<String>,
    /// Sidebar cache: `(worktree name, branch)` pairs.
    /// Refreshed in update (`App::refresh_sidebar_worktrees`), never in render.
    pub worktrees: Vec<(String, String)>,
}

impl RepositoryState {
    pub fn new(current_branch: String, branches: Vec<Branch>) -> Self {
        Self {
            branches,
            current_branch,
            git_file_statuses: HashMap::new(),
            file_tree: Vec::new(),
            stashes: Vec::new(),
            stash_files: Vec::new(),
            stash_diff: String::new(),
            branch_info: String::new(),
            diff_files: Vec::new(),
            diff_preview: None,
            commit_tree: Vec::new(),
            commit_diff: None,
            commit_stats: None,
            inspector_branch: String::new(),
            inspector_commits: Vec::new(),
            worktrees: Vec::new(),
        }
    }
}
