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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PullRequestAuthor {
    pub login: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PullRequest {
    pub number: u64,
    pub title: String,
    pub author: PullRequestAuthor,
    #[serde(rename = "headRefName")]
    pub head_ref_name: String,
    #[serde(rename = "baseRefName")]
    pub base_ref_name: String,
    pub state: String,
    #[serde(rename = "updatedAt", default)]
    pub updated_at: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrComment {
    pub author: PullRequestAuthor,
    pub body: String,
    #[serde(rename = "createdAt", default)]
    pub created_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CiStatus {
    #[default]
    Unknown,
    Passing,
    Failing,
    Pending,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PrCheck {
    pub name: String,
    pub state: String,
    #[serde(default)]
    pub bucket: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub link: String,
}

#[must_use]
pub fn compute_ci_status(checks: &[PrCheck]) -> CiStatus {
    if checks.is_empty() {
        return CiStatus::Unknown;
    }
    if checks.iter().any(|c| {
        c.bucket == "fail"
            || c.bucket == "cancel"
            || c.state == "FAILURE"
            || c.state == "ERROR"
            || c.state == "CANCELLED"
    }) {
        CiStatus::Failing
    } else if checks
        .iter()
        .any(|c| c.bucket == "pending" || c.state == "PENDING" || c.state == "IN_PROGRESS")
    {
        CiStatus::Pending
    } else {
        CiStatus::Passing
    }
}
