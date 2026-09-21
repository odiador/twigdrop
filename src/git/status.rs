use crate::git::commands::run_git;

pub fn get_branches(path: &str) -> Vec<String> {
    run_git(path, &["branch", "--format=%(refname:short)"])
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BranchMetadata {
    pub age: String,
    pub author: String,
    pub commit_date: String,
}

pub fn parse_branch_metadata_line(line: &str) -> Option<(String, BranchMetadata)> {
    let parts: Vec<&str> = line.trim().split('|').collect();
    if parts.len() >= 4 {
        Some((
            parts[0].to_string(),
            BranchMetadata {
                age: parts[1].to_string(),
                author: parts[2].to_string(),
                commit_date: parts[3].to_string(),
            },
        ))
    } else {
        None
    }
}

pub fn get_branch_metadata(path: &str) -> std::collections::HashMap<String, BranchMetadata> {
    let out = run_git(
        path,
        &[
            "branch",
            "--format=%(refname:short)|%(committerdate:relative)|%(authorname)|%(committerdate:short)",
        ],
    )
    .unwrap_or_default();

    let mut map = std::collections::HashMap::new();
    for line in out.lines() {
        if let Some((branch, meta)) = parse_branch_metadata_line(line) {
            map.insert(branch, meta);
        }
    }
    map
}

pub fn get_current_branch(path: &str) -> String {
    run_git(path, &["branch", "--show-current"])
        .unwrap_or_default()
        .trim()
        .to_string()
}

#[allow(dead_code)]
pub fn is_merging(path: &str) -> bool {
    if let Some(git_dir) = crate::git::commands::get_git_dir(path) {
        git_dir.join("MERGE_HEAD").exists()
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn is_rebasing(path: &str) -> bool {
    if let Some(git_dir) = crate::git::commands::get_git_dir(path) {
        git_dir.join("rebase-merge").exists() || git_dir.join("rebase-apply").exists()
    } else {
        false
    }
}

#[allow(dead_code)]
pub fn get_conflicting_files(path: &str) -> Vec<String> {
    run_git(path, &["diff", "--name-only", "--diff-filter=U"])
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn has_unique_commits(path: &str, branch: &str) -> bool {
    // Task 1.3: Verify if remotes exist to avoid false positives
    let remotes = run_git(path, &["remote"]).unwrap_or_default();
    if remotes.is_empty() {
        return false;
    }

    let out = run_git(path, &["rev-list", branch, "--not", "--remotes", "--count"])
        .unwrap_or_else(|_| "0".to_string());

    out.trim() != "0"
}

pub fn get_merged_branches(path: &str) -> Vec<String> {
    run_git(path, &["branch", "--format=%(refname:short)", "--merged"])
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub struct TrackInfo {
    pub has_upstream: bool,
    pub track: String,
    pub ahead: usize,
    pub behind: usize,
}

pub fn get_upstream_tracks(path: &str) -> std::collections::HashMap<String, TrackInfo> {
    let out = run_git(
        path,
        &[
            "branch",
            "--format=%(refname:short)|%(upstream)|%(upstream:track)",
        ],
    )
    .unwrap_or_default();

    let mut map = std::collections::HashMap::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.trim().split('|').collect();
        if parts.len() >= 3 {
            let track_str = parts[2].to_string();
            let (ahead, behind) = parse_ahead_behind(&track_str);

            map.insert(
                parts[0].to_string(),
                TrackInfo {
                    has_upstream: !parts[1].is_empty(),
                    track: track_str,
                    ahead,
                    behind,
                },
            );
        }
    }
    map
}

pub fn parse_ahead_behind(track_str: &str) -> (usize, usize) {
    let mut ahead = 0;
    let mut behind = 0;

    if track_str.contains("ahead")
        && let Some(a) = track_str
            .split("ahead ")
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_numeric()).next())
    {
        ahead = a.parse().unwrap_or(0);
    }
    if track_str.contains("behind")
        && let Some(b) = track_str
            .split("behind ")
            .nth(1)
            .and_then(|s| s.split(|c: char| !c.is_numeric()).next())
    {
        behind = b.parse().unwrap_or(0);
    }

    (ahead, behind)
}

pub fn parse_stashed_branch_line(line: &str) -> Option<String> {
    let lower = line.to_lowercase();
    if let Some(start) = lower.find("on ") {
        let sub = &line[start + 3..];
        if let Some(end) = sub.find(':') {
            return Some(sub[..end].to_string());
        }
    }
    None
}

pub fn get_stashed_branches(path: &str) -> Vec<String> {
    let out = run_git(path, &["stash", "list"]).unwrap_or_default();
    let mut branches = vec![];
    for line in out.lines() {
        if let Some(branch) = parse_stashed_branch_line(line) {
            branches.push(branch);
        }
    }
    branches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ahead_behind() {
        assert_eq!(parse_ahead_behind("[ahead 1, behind 2]"), (1, 2));
        assert_eq!(parse_ahead_behind("[behind 13]"), (0, 13));
        assert_eq!(parse_ahead_behind("[ahead 5]"), (5, 0));
        assert_eq!(parse_ahead_behind("[gone]"), (0, 0));
        assert_eq!(parse_ahead_behind(""), (0, 0));
        assert_eq!(parse_ahead_behind("[ahead 10, behind 3]"), (10, 3));
    }

    #[test]
    fn test_parse_branch_metadata_line() {
        let line = "feat/new-ui|2 hours ago|Jane Doe|2026-09-16";
        let parsed = parse_branch_metadata_line(line);
        assert_eq!(
            parsed,
            Some((
                "feat/new-ui".to_string(),
                BranchMetadata {
                    age: "2 hours ago".to_string(),
                    author: "Jane Doe".to_string(),
                    commit_date: "2026-09-16".to_string(),
                }
            ))
        );

        assert_eq!(parse_branch_metadata_line("invalid|format"), None);
    }

    #[test]
    fn test_parse_stashed_branch_line() {
        let line1 = "stash@{0}: WIP on main: 1a2b3c4 Commit message";
        assert_eq!(parse_stashed_branch_line(line1), Some("main".to_string()));

        let line2 = "stash@{1}: On feature/auth: Some message";
        assert_eq!(parse_stashed_branch_line(line2), Some("feature/auth".to_string()));

        let line3 = "stash@{2}: custom stash without on";
        assert_eq!(parse_stashed_branch_line(line3), None);
    }

    #[test]
    fn test_is_merging_and_rebasing() {
        // In a normal repo state, neither merging nor rebasing is active
        assert!(!is_merging("."));
        assert!(!is_rebasing("."));

        // In a non-git dir, neither should panic, both should return false
        let temp = tempfile::tempdir().unwrap();
        assert!(!is_merging(temp.path().to_str().unwrap()));
        assert!(!is_rebasing(temp.path().to_str().unwrap()));
    }
}
