#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GitError {
    Command(String),
    NoRemote,
    Conflict(Vec<String>),
    Io(String),
}

impl std::fmt::Display for GitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Command(msg) => write!(f, "git command failed: {msg}"),
            Self::NoRemote => write!(f, "no remote configured"),
            Self::Conflict(files) => write!(f, "merge conflict in: {}", files.join(", ")),
            Self::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for GitError {}

impl From<std::io::Error> for GitError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<anyhow::Error> for GitError {
    fn from(err: anyhow::Error) -> Self {
        Self::Command(err.to_string())
    }
}
