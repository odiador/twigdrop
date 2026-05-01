use std::process::Command;
use crate::models::{Branch, BranchStatus};

fn run_git(path: &str, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .expect("git failed");

    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn get_branches(path: &str) -> Vec<String> {
    let out = run_git(path, &["branch", "--format=%(refname:short)"]);
    // Filter out potential empty lines
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

pub fn get_upstream_tracks(path: &str) -> std::collections::HashMap<String, String> {
    let out = run_git(path, &["branch", "--format=%(refname:short) %(upstream:track)"]);
    let mut map = std::collections::HashMap::new();
    for line in out.lines() {
        let parts: Vec<&str> = line.trim().splitn(2, ' ').collect();
        if parts.len() == 2 {
            map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    map
}

pub fn get_branch_info(path: &str, branch: &str) -> String {
    run_git(path, &["log", "-n", "3", "--stat", "-p", "--color=never", branch])
}

pub fn build_branches(path: &str) -> Vec<Branch> {
    let names = get_branches(path);
    let merged = get_merged_branches(path);
    let tracks = get_upstream_tracks(path);

    names
        .into_iter()
        .map(|name| {
            let mut status = vec![];
            let track_info = tracks.get(&name).map(|s| s.as_str()).unwrap_or("");

            if track_info.contains("[gone]") {
                status.push(BranchStatus::Gone);
            }
            if track_info.contains("ahead") {
                status.push(BranchStatus::Ahead);
            }
            if merged.contains(&name) {
                status.push(BranchStatus::Merged);
            }
            if has_unique_commits(path, &name) {
                status.push(BranchStatus::HasUniqueCommits);
            }

            if status.is_empty() {
                status.push(BranchStatus::Safe);
            }

            Branch { name, status }
        })
        .collect()
}
