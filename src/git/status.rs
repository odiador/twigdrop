use crate::git::commands::run_git;

pub fn get_branches(path: &str) -> Vec<String> {
    let out = run_git(path, &["branch", "--format=%(refname:short)"]);
    out.lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn has_unique_commits(path: &str, branch: &str) -> bool {
    let out = run_git(path, &[
        "rev-list",
        branch,
        "--not",
        "--remotes=origin",
        "--count",
    ]);

    out.trim() != "0"
}

pub fn get_merged_branches(path: &str) -> Vec<String> {
    let out = run_git(path, &["branch", "--format=%(refname:short)", "--merged"]);
    out.lines().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

pub struct TrackInfo {
    pub has_upstream: bool,
    pub track: String,
}

pub fn get_upstream_tracks(path: &str) -> std::collections::HashMap<String, TrackInfo> {
    let out = run_git(path, &["branch", "--format=%(refname:short)|%(upstream)|%(upstream:track)"]);
    let mut map = std::collections::HashMap::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.trim().split('|').collect();
        if parts.len() >= 3 {
            map.insert(parts[0].to_string(), TrackInfo {
                has_upstream: !parts[1].is_empty(),
                track: parts[2].to_string(),
            });
        }
    }
    map
}

pub fn get_stashed_branches(path: &str) -> Vec<String> {
    let out = run_git(path, &["stash", "list"]);
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
