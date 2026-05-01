pub mod commands;
pub mod status;

pub use status::get_current_branch;

use crate::models::{Branch, BranchStatus};
use crate::git::status::{
    get_branches, get_merged_branches, get_upstream_tracks, get_stashed_branches, has_unique_commits
};
use crate::git::commands::run_git;

pub fn build_branches(path: &str) -> Vec<Branch> {
    let names = get_branches(path);
    let merged = get_merged_branches(path);
    let tracks = get_upstream_tracks(path);
    let stashed = get_stashed_branches(path);

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

            Branch { name, status }
        })
        .collect()
}

pub fn get_branch_info(path: &str, branch: &str) -> String {
    run_git(path, &["log", "-n", "3", "--stat", "-p", "--color=never", branch])
}
