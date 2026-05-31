use crate::git::commands::run_git;

pub fn get_branches(path: &str) -> Vec<String> {
    run_git(path, &["branch", "--format=%(refname:short)"])
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub struct BranchMetadata {
    pub age: String,
    pub author: String,
    pub commit_date: String,
}

pub fn get_branch_metadata(path: &str) -> std::collections::HashMap<String, BranchMetadata> {
    let out = run_git(
        path,
        &[
            "branch",
            "--format=%(refname:short)|%(committerdate:relative)|%(authorname)|%(committerdate:short)",
        ],
    ).unwrap_or_default();

    let mut map = std::collections::HashMap::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.trim().split('|').collect();
        if parts.len() >= 4 {
            map.insert(
                parts[0].to_string(),
                BranchMetadata {
                    age: parts[1].to_string(),
                    author: parts[2].to_string(),
                    commit_date: parts[3].to_string(),
                },
            );
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
    let git_dir = std::path::Path::new(path).join(".git");
    git_dir.join("MERGE_HEAD").exists()
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

pub fn get_stashed_branches(path: &str) -> Vec<String> {
    let out = run_git(path, &["stash", "list"]).unwrap_or_default();
    let mut branches = vec![];
    for line in out.lines() {
        let lower = line.to_lowercase();
        if let Some(start) = lower.find("on ") {
            let sub = &line[start + 3..];
            if let Some(end) = sub.find(':') {
                branches.push(sub[..end].to_string());
            }
        }
    }
    branches
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_track_parsing() {
        let track_str = "[ahead 1, behind 2]";
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

        assert_eq!(ahead, 1);
        assert_eq!(behind, 2);

        let track_str2 = "[behind 13]";
        let mut ahead2 = 0;
        let mut behind2 = 0;

        if track_str2.contains("ahead")
            && let Some(a) = track_str2
                .split("ahead ")
                .nth(1)
                .and_then(|s| s.split(|c: char| !c.is_numeric()).next())
        {
            ahead2 = a.parse().unwrap_or(0);
        }
        if track_str2.contains("behind")
            && let Some(b) = track_str2
                .split("behind ")
                .nth(1)
                .and_then(|s| s.split(|c: char| !c.is_numeric()).next())
        {
            behind2 = b.parse().unwrap_or(0);
        }

        assert_eq!(ahead2, 0);
        assert_eq!(behind2, 13);
    }
}
