pub mod commands;
pub mod files;
pub mod stash;
pub mod status;

pub use status::get_current_branch;

use crate::git::commands::{run_git, run_git_with_status};
use crate::git::status::{
    get_branch_metadata, get_branches, get_merged_branches, get_stashed_branches,
    get_upstream_tracks, has_unique_commits,
};
use crate::models::{Branch, BranchStatus, ConflictBlock, GutterStatus, MergeStatus};
use std::collections::HashMap;

pub fn build_branches(path: &str) -> Vec<Branch> {
    let mut branches = vec![];

    // Add Pseudo-branches for Local and Staged changes
    branches.push(Branch {
        name: "*Local Changes*".to_string(),
        status: vec![BranchStatus::Local],
        merge_status: MergeStatus::NotAnalyzed,
        age: "".to_string(),
        author: "You".to_string(),
        commit_date: "Now".to_string(),
        ahead_count: 0,
        behind_count: 0,
    });

    branches.push(Branch {
        name: "*Staged Changes*".to_string(),
        status: vec![BranchStatus::Local],
        merge_status: MergeStatus::NotAnalyzed,
        age: "".to_string(),
        author: "You".to_string(),
        commit_date: "Now".to_string(),
        ahead_count: 0,
        behind_count: 0,
    });

    let names = get_branches(path);
    let merged = get_merged_branches(path);
    let tracks = get_upstream_tracks(path);
    let stashed = get_stashed_branches(path);
    let metadata = get_branch_metadata(path);

    let mut git_branches: Vec<Branch> = names
        .into_iter()
        .map(|name| {
            let mut status = vec![];
            let track_info = tracks.get(&name);
            let mut ahead_count = 0;
            let mut behind_count = 0;

            if let Some(ti) = track_info {
                ahead_count = ti.ahead;
                behind_count = ti.behind;

                if !ti.has_upstream {
                    status.push(BranchStatus::Local);
                    status.push(BranchStatus::RemoteUntracked);
                } else {
                    status.push(BranchStatus::RemoteTracked);
                }

                if ti.track.contains("[gone]") {
                    status.push(BranchStatus::Gone);
                }
                if ti.track.contains("ahead") {
                    status.push(BranchStatus::Ahead);
                }
                if ti.track.contains("behind") {
                    status.push(BranchStatus::Behind);
                }
            } else {
                status.push(BranchStatus::Local);
                status.push(BranchStatus::RemoteUntracked);
            }

            if merged.contains(&name) {
                status.push(BranchStatus::Merged);
            }
            if has_unique_commits(path, &name) {
                status.push(BranchStatus::HasUniqueCommits);
            }
            if stashed.contains(&name) {
                status.push(BranchStatus::Stashed);
            }

            if status.is_empty() {
                status.push(BranchStatus::Safe);
            }

            let meta = metadata.get(&name);
            let age = meta.map(|m| m.age.clone()).unwrap_or_default();
            let author = meta.map(|m| m.author.clone()).unwrap_or_default();
            let commit_date = meta.map(|m| m.commit_date.clone()).unwrap_or_default();

            Branch {
                name,
                status,
                merge_status: MergeStatus::NotAnalyzed,
                age,
                author,
                commit_date,
                ahead_count,
                behind_count,
            }
        })
        .collect();

    branches.append(&mut git_branches);
    branches
}

pub fn get_branch_info(path: &str, branch: &str) -> String {
    if branch == "*Local Changes*" {
        return "Unstaged changes in working directory.".to_string();
    } else if branch == "*Staged Changes*" {
        return "Staged changes ready to be committed.".to_string();
    }

    run_git(
        path,
        &["log", "-n", "3", "--stat", "-p", "--color=never", branch],
    )
    .unwrap_or_else(|e| format!("Error loading branch info: {}", e))
}

pub fn get_branch_diff_files(path: &str, branch: &str) -> Vec<String> {
    if branch == "*Local Changes*" {
        return run_git(path, &["diff", "--name-only"])
            .unwrap_or_default()
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    } else if branch == "*Staged Changes*" {
        return run_git(path, &["diff", "--cached", "--name-only"])
            .unwrap_or_default()
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    // Diff against HEAD
    run_git(path, &["diff", "--name-only", "HEAD", branch])
        .unwrap_or_default()
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn get_branch_file_diff(path: &str, branch: &str, file: &str) -> String {
    if branch == "*Local Changes*" {
        return run_git(path, &["diff", "--", file])
            .unwrap_or_else(|e| format!("Error loading diff: {}", e));
    } else if branch == "*Staged Changes*" {
        return run_git(path, &["diff", "--cached", "--", file])
            .unwrap_or_else(|e| format!("Error loading diff: {}", e));
    }

    run_git(path, &["diff", "HEAD", branch, "--", file])
        .unwrap_or_else(|e| format!("Error loading diff: {}", e))
}

#[allow(dead_code)]
pub fn get_branch_commits(path: &str, branch: &str) -> Vec<crate::models::Commit> {
    let range = if branch == "*Local Changes*" || branch == "*Staged Changes*" {
        "HEAD"
    } else {
        branch
    };

    let out = run_git(path, &["log", range, "--format=%h|%s|%cr|%an"]).unwrap_or_default();

    out.lines()
        .map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                crate::models::Commit {
                    hash: parts[0].to_string(),
                    message: parts[1].to_string(),
                    date: parts[2].to_string(),
                    author: parts[3].to_string(),
                }
            } else {
                crate::models::Commit {
                    hash: "".to_string(),
                    message: line.to_string(),
                    date: "".to_string(),
                    author: "".to_string(),
                }
            }
        })
        .filter(|c| !c.hash.is_empty())
        .collect()
}

pub fn get_unpushed_commits(path: &str) -> Vec<crate::models::Commit> {
    // Determine the upstream tracking branch or use origin/HEAD
    let upstream =
        run_git(path, &["rev-parse", "--abbrev-ref", "@{u}"]).unwrap_or_else(|_| "".to_string());

    let range = if upstream.trim().is_empty() {
        "HEAD".to_string() // If no upstream, all commits are unpushed technically, or we could just show last 10
    } else {
        format!("{}..HEAD", upstream.trim())
    };

    let out = run_git(path, &["log", &range, "--format=%h|%s|%cr|%an"]).unwrap_or_default();

    out.lines()
        .map(|line| {
            let parts: Vec<&str> = line.splitn(4, '|').collect();
            if parts.len() == 4 {
                crate::models::Commit {
                    hash: parts[0].to_string(),
                    message: parts[1].to_string(),
                    date: parts[2].to_string(),
                    author: parts[3].to_string(),
                }
            } else {
                crate::models::Commit {
                    hash: "".to_string(),
                    message: line.to_string(),
                    date: "".to_string(),
                    author: "".to_string(),
                }
            }
        })
        .filter(|c| !c.hash.is_empty())
        .collect()
}

pub fn analyze_merge_status(path: &str, target_branch: &str, current_branch: &str) -> MergeStatus {
    if target_branch == current_branch {
        return MergeStatus::Clean;
    }

    // 1. Get merge base
    let merge_base = match run_git(path, &["merge-base", current_branch, target_branch]) {
        Ok(mb) => mb.trim().to_string(),
        Err(_) => return MergeStatus::SafeLimit(0, 0),
    };

    if merge_base.is_empty() {
        return MergeStatus::SafeLimit(0, 0);
    }

    // 2. Get commits to apply
    let commits_str = match run_git(
        path,
        &[
            "log",
            "--reverse",
            "--format=%H",
            &format!("{}..{}", merge_base, target_branch),
        ],
    ) {
        Ok(c) => c,
        Err(_) => return MergeStatus::SafeLimit(0, 0),
    };

    let commits: Vec<&str> = commits_str.lines().collect();

    if commits.is_empty() {
        return MergeStatus::Clean;
    }

    let mut current_tree = match run_git(
        path,
        &["rev-parse", &format!("{}^{{tree}}", current_branch)],
    ) {
        Ok(t) => t.trim().to_string(),
        Err(_) => return MergeStatus::SafeLimit(0, 0),
    };

    let total_commits = commits.len();
    let mut safe_commits = 0;

    for commit in commits {
        let parent = match run_git(path, &["rev-parse", &format!("{}^1", commit)]) {
            Ok(p) => p.trim().to_string(),
            Err(_) => {
                return MergeStatus::SafeLimit(safe_commits, total_commits);
            }
        };

        match run_git_with_status(
            path,
            &[
                "merge-tree",
                "--write-tree",
                &format!("--merge-base={}", parent),
                &current_tree,
                commit,
            ],
        ) {
            Ok((output, 0)) => {
                current_tree = output.trim().to_string();
                safe_commits += 1;
            }
            _ => {
                // Conflict detected at this commit
                let conflicts = get_conflicts_from_merge(path, &parent, &current_tree, commit);
                if !conflicts.is_empty() {
                    return MergeStatus::Conflict(conflicts);
                }
                return MergeStatus::SafeLimit(safe_commits, total_commits);
            }
        }
    }

    if safe_commits == total_commits {
        MergeStatus::Clean
    } else {
        MergeStatus::SafeLimit(safe_commits, total_commits)
    }
}

fn get_conflicts_from_merge(path: &str, base: &str, our: &str, their: &str) -> Vec<ConflictBlock> {
    let mut conflicts = vec![];

    // Use merge-tree with --write-tree to get the OID with markers
    let out = match run_git(
        path,
        &[
            "merge-tree",
            "--write-tree",
            &format!("--merge-base={}", base),
            our,
            their,
        ],
    ) {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let tree_oid = out.lines().next().unwrap_or_default().trim();
    if tree_oid.is_empty() {
        return vec![];
    }

    // Parse conflicted files from the output (skip first line which is tree OID)
    for line in out.lines().skip(1) {
        if line.contains('\t') || line.split_whitespace().count() >= 4 {
            let parts: Vec<&str> = line.split_whitespace().collect();
            // Format: <mode> <object> <stage> <filename>
            if parts.len() >= 4 && parts[2] == "1" { // stage 1 is base? No, 2/3 are ours/theirs.
                // Actually, any entry here is a conflict.
            }
        }
    }

    // Simplest: just run the old trivial merge for parsing markers if needed,
    // but the user wants "Structured Diff blocks".

    // Let's stick to the current naive parser but fix the command.
    let out = match run_git(path, &["merge-tree", "--trivial-merge", base, our, their]) {
        Ok(o) => o,
        Err(_) => return vec![],
    };

    let mut current_file = String::new();
    let mut current_block = String::new();
    let mut in_conflict = false;

    for line in out.lines() {
        // This is a naive parser for merge-tree output
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            current_block.push_str(line);
            current_block.push('\n');
        } else if line.starts_with(">>>>>>>") {
            current_block.push_str(line);
            current_block.push('\n');
            if !current_file.is_empty() {
                conflicts.push(ConflictBlock {
                    file_path: current_file.clone(),
                    content: current_block.clone(),
                });
            }
            current_block.clear();
            in_conflict = false;
        } else if in_conflict {
            current_block.push_str(line);
            current_block.push('\n');
        } else if let Some(stripped) = line.strip_prefix("+++ b/") {
            current_file = stripped.to_string();
        }
    }

    conflicts
}

pub fn get_line_diffs(path: &str, file_path: &str) -> HashMap<usize, GutterStatus> {
    let mut diffs = HashMap::new();
    let out = match run_git(path, &["diff", "--unified=0", "HEAD", "--", file_path]) {
        Ok(o) => o,
        Err(_) => return diffs,
    };

    for line in out.lines() {
        if line.starts_with("@@") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let old_part = parts[1]; // -10,1
                let new_part = parts[2]; // +11,2

                let old_info: Vec<usize> = old_part
                    .trim_start_matches('-')
                    .split(',')
                    .map(|s| s.parse().unwrap_or(0))
                    .collect();
                let new_info: Vec<usize> = new_part
                    .trim_start_matches('+')
                    .split(',')
                    .map(|s| s.parse().unwrap_or(0))
                    .collect();

                let old_count = if old_info.len() > 1 {
                    old_info[1]
                } else if !old_info.is_empty() {
                    1
                } else {
                    0
                };
                let new_count = if new_info.len() > 1 {
                    new_info[1]
                } else if !new_info.is_empty() {
                    1
                } else {
                    0
                };
                let new_start = if !new_info.is_empty() { new_info[0] } else { 0 };

                if new_start > 0 {
                    if old_count > 0 && new_count > 0 {
                        for i in 0..new_count {
                            diffs.insert(new_start + i - 1, GutterStatus::Modified);
                        }
                    } else if old_count == 0 && new_count > 0 {
                        for i in 0..new_count {
                            diffs.insert(new_start + i - 1, GutterStatus::Added);
                        }
                    } else if old_count > 0 && new_count == 0 {
                        diffs.insert(new_start.saturating_sub(1), GutterStatus::Deleted);
                    }
                }
            }
        }
    }
    diffs
}
