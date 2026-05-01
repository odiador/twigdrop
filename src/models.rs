#[derive(Debug, Clone, PartialEq)]
pub enum BranchStatus {
    Safe,
    Gone,
    Merged,
    HasUniqueCommits,
    Ahead,
    Behind,
    Local,
    Stashed,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub status: Vec<BranchStatus>,
}
