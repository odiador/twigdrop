pub use crate::state::domain::RepositoryState;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BranchStatus {
    Merged,
    HasUniqueCommits,
    Gone,
    Ahead,
    Behind,
    Stashed,
    Local,
    Safe,
    RemoteTracked,
    RemoteUntracked,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConflictBlock {
    pub file_path: String,
    pub content: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MergeStatus {
    Checking,
    Clean,
    Conflict(Vec<ConflictBlock>),
    SafeLimit(usize, usize), // (Safe commits, Total commits)
    NotAnalyzed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GutterStatus {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default)]
pub enum BranchKind {
    #[default]
    Real,
    LocalChanges,
    StagedChanges,
}

pub const SYNTHETIC_LOCAL_CHANGES: &str = "*Local Changes*";
pub const SYNTHETIC_STAGED_CHANGES: &str = "*Staged Changes*";

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub kind: BranchKind,
    pub status: Vec<BranchStatus>,
    pub merge_status: MergeStatus,
    pub age: String,
    pub author: String,
    pub commit_date: String,
    pub ahead_count: usize,
    pub behind_count: usize,
}

impl Branch {
    #[must_use]
    pub fn is_synthetic(&self) -> bool {
        self.kind != BranchKind::Real
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub date: String,
    pub author: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitTreeItem {
    pub graph: String,
    pub hash: String,
    pub branch_info: String,
    pub date: String,
    pub author: String,
    pub message: String,
}
