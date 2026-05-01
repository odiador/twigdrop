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

pub struct Branch {
    pub name: String,
    pub status: Vec<BranchStatus>,
}
