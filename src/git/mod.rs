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
use crate::models::{Branch, BranchStatus, ConflictBlock, MergeStatus, GutterStatus};
use std::collections::HashMap;

pub fn build_branches(path: &str) -> Vec<Branch> {
    let names = get_branches(path);
    let merged = get_merged_branches(path);
    let tracks = get_upstream_tracks(path);
    let stashed = get_stashed_branches(path);
    let metadata = get_branch_metadata(path);

    names
        .into_iter()
        .map(|name| {
            let mut status = vec![];
            let track_info = tracks.get(&name);

            if let Some(ti) = track_info {
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
            }
        })
        .collect()
}

pub fn get_branch_info(path: &str, branch: &str) -> String {
    run_git(
        path,
        &["log", "-n", "3", "--stat", "-p", "--color=never", branch],
    )
    .unwrap_or_else(|e| format!("Error loading branch info: {}", e))
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
            &["merge-tree", "--write-tree", &parent, &current_tree, commit],
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

    // Use merge-tree without --write-tree to get the diff with conflict markers
    let out = match run_git(path, &["merge-tree", base, our, their]) {
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
                
                let old_info: Vec<usize> = old_part.trim_start_matches('-').split(',').map(|s| s.parse().unwrap_or(0)).collect();
                let new_info: Vec<usize> = new_part.trim_start_matches('+').split(',').map(|s| s.parse().unwrap_or(0)).collect();
                
                let old_count = if old_info.len() > 1 { old_info[1] } else if !old_info.is_empty() { 1 } else { 0 };
                let new_count = if new_info.len() > 1 { new_info[1] } else if !new_info.is_empty() { 1 } else { 0 };
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
