use crate::models::{Branch, Commit};
use crate::git::files::FileStatus;
use crate::git::stash::StashEntry;
use std::collections::HashMap;

#[allow(dead_code)]
pub struct RepositoryState {
    pub branches: Vec<Branch>,
    pub current_branch: String,
    pub git_file_statuses: HashMap<String, FileStatus>,
    pub stashes: Vec<StashEntry>,
    pub commits: Vec<Commit>,
}

impl RepositoryState {
    pub fn new(current_branch: String, branches: Vec<Branch>) -> Self {
        Self {
            branches,
            current_branch,
            git_file_statuses: HashMap::new(),
            stashes: Vec::new(),
            commits: Vec::new(),
        }
    }
}
