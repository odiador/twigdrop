use crate::git::commands::{run_git, run_git_with_status};
use crate::state::ui::{RebaseAction, RebaseCommit};
use std::fs;

pub fn bulk_delete_branches(path: &str, names: &[String]) -> String {
    let mut results = String::new();
    for name in names {
        match run_git(path, &["branch", "-D", name]) {
            Ok(output) => {
                results.push_str(&format!("> git branch -D {}\n{}\n", name, output.trim()))
            }
            Err(e) => results.push_str(&format!("Error deleting branch {}: {}\n", name, e)),
        }
    }
    results
}

pub fn prune_branches(
    path: &str,
    branches: &[crate::models::Branch],
    current_branch: &str,
) -> String {
    let mut to_delete = vec![];
    for b in branches {
        if b.name == current_branch {
            continue;
        }

        let is_gone = b.status.contains(&crate::models::BranchStatus::Gone);
        let has_unique = b
            .status
            .contains(&crate::models::BranchStatus::HasUniqueCommits);
        let has_stash = b.status.contains(&crate::models::BranchStatus::Stashed);

        if is_gone && !has_unique && !has_stash {
            to_delete.push(b.name.clone());
        }
    }

    if to_delete.is_empty() {
        return "No safe branches found to prune.".to_string();
    }

    let count = to_delete.len();
    let details = bulk_delete_branches(path, &to_delete);
    format!("Pruned {} branches:\n{}", count, details)
}

pub fn apply_stash(path: &str, id: &str) -> String {
    match run_git(path, &["stash", "apply", id]) {
        Ok(output) => format!("> git stash apply {}\n{}", id, output.trim()),
        Err(e) => format!("Error applying stash {}: {}", id, e),
    }
}

pub fn stage_file(path: &str, file_path: &str) -> Result<String, String> {
    match run_git(path, &["add", file_path]) {
        Ok(_) => Ok(format!("Staged {}", file_path)),
        Err(e) => Err(format!("Error staging {}: {}", file_path, e)),
    }
}

pub fn unstage_file(path: &str, file_path: &str) -> Result<String, String> {
    // Check if it's a new file or already in HEAD
    let is_new = match run_git(path, &["ls-files", "--stage", file_path]) {
        Ok(out) => out.is_empty(),
        Err(_) => true,
    };

    let args = if is_new {
        vec!["rm", "--cached", file_path]
    } else {
        vec!["restore", "--staged", file_path]
    };

    match run_git(path, &args) {
        Ok(_) => Ok(format!("Unstaged {}", file_path)),
        Err(e) => Err(format!("Error unstaging {}: {}", file_path, e)),
    }
}

pub fn apply_resolution_to_file(
    repo_path: &str,
    file_path: &str,
    original_block: &str,
    resolved_content: &str,
) -> Result<(), String> {
    let full_path = std::path::Path::new(repo_path).join(file_path);
    let content = fs::read_to_string(&full_path).map_err(|e| e.to_string())?;

    if let Some(start) = content.find(original_block) {
        let mut new_content = content.clone();
        new_content.replace_range(start..start + original_block.len(), resolved_content);
        fs::write(&full_path, new_content).map_err(|e| e.to_string())?;

        // Stage the file if we fixed a conflict
        let _ = run_git(repo_path, &["add", file_path]);

        Ok(())
    } else {
        Err("Could not find conflict block in file. Maybe it was already resolved?".to_string())
    }
}

pub fn move_file_changes_forward(
    path: &str,
    hash: &str,
    file_rel_path: &str,
) -> Result<String, String> {
    // 1. Find the next commit that touches this file
    let next_commit_hash = match run_git(
        path,
        &[
            "log",
            "--format=%H",
            "--reverse",
            &format!("{}..HEAD", hash),
            "--",
            file_rel_path,
        ],
    ) {
        Ok(out) => out.lines().next().map(|s| s.to_string()),
        Err(_) => None,
    };

    // 2. Extract changes and rebase
    // We use a temporary fixup strategy

    // a. Restore file to state before target commit
    let _ = run_git(
        path,
        &["checkout", &format!("{}^", hash), "--", file_rel_path],
    );
    // b. Fixup the target commit to remove those changes
    let _ = run_git(path, &["commit", "--fixup", hash]);

    // c. Restore file to state at target commit (reintroducing the changes as floating)
    let _ = run_git(path, &["checkout", hash, "--", file_rel_path]);

    let result_msg = if let Some(next_h) = next_commit_hash {
        // d. Fixup the next commit to absorb these changes
        let _ = run_git(path, &["commit", "--fixup", &next_h]);
        format!("Moving changes from {} to next commit {}.", hash, next_h)
    } else {
        "No future commit touches this file. Changes left staged at HEAD.".to_string()
    };

    // 3. Run the rebase to consolidate
    match run_git(
        path,
        &[
            "-c",
            "sequence.editor=:",
            "rebase",
            "-i",
            "--autosquash",
            "--autostash",
            &format!("{}^", hash),
        ],
    ) {
        Ok(_) => Ok(format!("Success! {}", result_msg)),
        Err(e) => Err(format!(
            "Rebase failed: {}. Manual resolution might be needed.",
            e
        )),
    }
}

pub fn execute_interactive_rebase(path: &str, commits: &[RebaseCommit]) {
    use std::io::Write;

    // Fallback: we write a script that git can use as sequence editor
    let editor_script_path = std::path::Path::new(path)
        .join(".git")
        .join("twigdrop-rebase-editor.sh");

    let mut script_content = String::new();
    script_content.push_str("#!/bin/sh\n");
    script_content.push_str("cat << 'REBASE_EOF' > \"$1\"\n");

    for commit in commits.iter().rev() {
        let action = match commit.action {
            RebaseAction::Pick => "pick",
            RebaseAction::Reword => "reword",
            RebaseAction::Drop => "drop",
            RebaseAction::Squash => "squash",
        };
        // For reword, we'd need another editor script to provide the actual new message.
        // Or we can use the `exec` command to run git commit --amend.
        // Actually, git rebase has a trick: `x git commit --amend -m "new message"`
        if commit.action == RebaseAction::Reword {
            script_content.push_str(&format!(
                "pick {} {}\n",
                commit.hash,
                commit.original_message.replace("'", "'\\''")
            ));
            let new_msg = commit
                .new_message
                .clone()
                .unwrap_or_else(|| commit.original_message.clone());
            script_content.push_str(&format!(
                "x git commit --amend -m '{}'\n",
                new_msg.replace("'", "'\\''")
            ));
        } else {
            script_content.push_str(&format!(
                "{} {} {}\n",
                action,
                commit.hash,
                commit.original_message.replace("'", "'\\''")
            ));
        }
    }
    script_content.push_str("REBASE_EOF\n");

    if let Ok(mut file) = std::fs::File::create(&editor_script_path) {
        let _ = file.write_all(script_content.as_bytes());
    }

    #[cfg(unix)]
    let _ = std::process::Command::new("chmod")
        .arg("+x")
        .arg(&editor_script_path)
        .status();

    if let Some(first_commit) = commits.last() {
        let _ = run_git_with_status(
            path,
            &[
                "-c",
                &format!("sequence.editor={}", editor_script_path.display()),
                "rebase",
                "-i",
                "--autostash",
                &format!("{}^", first_commit.hash),
            ],
        );
    }
    let _ = std::fs::remove_file(editor_script_path);
}
