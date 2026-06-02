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
    #[allow(dead_code)]
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

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub status: Vec<BranchStatus>,
    pub merge_status: MergeStatus,
    pub age: String,
    pub author: String,
    pub commit_date: String,
    pub ahead_count: usize,
    pub behind_count: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Commit {
    pub hash: String,
    pub message: String,
    pub date: String,
    pub author: String,
}

#[derive(Debug, Clone)]
pub struct CommitTreeItem {
    pub graph: String,
    pub hash: String,
    pub branch_info: String,
    pub date: String,
    pub author: String,
    pub message: String,
}
