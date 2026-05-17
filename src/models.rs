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
pub enum MergeStatus {
    #[allow(dead_code)]
    Checking,
    Clean,
    Conflict(String),
    SafeLimit(usize, usize), // (Safe commits, Total commits)
    NotAnalyzed,
}

pub struct Branch {
    pub name: String,
    pub status: Vec<BranchStatus>,
    pub merge_status: MergeStatus,
    pub age: String,
    pub author: String,
    pub commit_date: String,
}
