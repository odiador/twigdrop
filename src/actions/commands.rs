use crate::git::commands::{run_git, run_git_with_status};
use crate::git::error::GitError;
use crate::state::ui::{RebaseAction, RebaseCommit};
use std::fs;

pub use crate::utils::config::DEFAULT_PROTECTED_BRANCHES;

pub fn is_protected_branch(name: &str) -> bool {
    is_protected_branch_with(name, &[])
}

/// Protected check with user-configured extras (`Config::protected_branches`).
/// Matching is case-insensitive and ignores a leading `*` marker.
pub fn is_protected_branch_with(name: &str, extra: &[String]) -> bool {
    let clean = name.trim().trim_start_matches('*').trim();
    DEFAULT_PROTECTED_BRANCHES
        .iter()
        .any(|&p| clean.eq_ignore_ascii_case(p))
        || extra.iter().any(|p| clean.eq_ignore_ascii_case(p.trim()))
}

/// Resolve the current SHA of `name` without touching the working tree.
/// Used for pre-delete backups; `None` when unresolvable.
pub fn resolve_branch_sha(path: &str, name: &str) -> Option<String> {
    run_git(path, &["rev-parse", name])
        .ok()
        .map(|out| out.trim().to_string())
        .filter(|sha| !sha.is_empty())
}

/// Filesystem-safe slug for a repo path (`/` and `:` become `_`).
pub fn repo_slug(path: &str) -> String {
    path.replace(['/', '\\', ':'], "_")
        .chars()
        .take(120)
        .collect()
}

/// Append a `<branch> <sha>` backup line. Pure filesystem helper —
/// takes `base` explicitly so tests never touch the real home dir.
pub fn write_backup_line(
    base: &std::path::Path,
    slug: &str,
    name: &str,
    sha: &str,
) -> std::io::Result<std::path::PathBuf> {
    let dir = base.join(slug);
    fs::create_dir_all(&dir)?;
    let file = dir.join(format!("{}.log", chrono::Local::now().format("%Y-%m-%d")));
    use std::io::Write;
    let mut f = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&file)?;
    writeln!(
        f,
        "{} {} {}",
        chrono::Local::now().format("%H:%M:%S"),
        name,
        sha
    )?;
    Ok(file)
}

/// Best-effort pre-delete backup into `~/.twigdrop/backups/<slug>/`.
/// Never fails the delete itself — errors are swallowed by design.
pub fn backup_branch_sha(path: &str, name: &str) {
    let (Some(home), Some(sha)) = (directories::BaseDirs::new(), resolve_branch_sha(path, name))
    else {
        return;
    };
    let base = home.home_dir().join(".twigdrop").join("backups");
    let _ = write_backup_line(&base, &repo_slug(path), name, &sha);
}

pub fn bulk_delete_branches(path: &str, names: &[String], extra_protected: &[String]) -> String {
    let mut results = String::new();
    for name in names {
        if is_protected_branch_with(name, extra_protected) {
            results.push_str(&format!("Skipping protected branch: {name}\n"));
            continue;
        }

        backup_branch_sha(path, name);
        match run_git(path, &["branch", "-D", name]) {
            Ok(output) => {
                results.push_str(&format!("> git branch -D {}\n{}\n", name, output.trim()))
            }
            Err(e) => results.push_str(&format!("Error deleting branch {name}: {e}\n")),
        }
    }
    results
}

pub fn get_prunable_branches(
    branches: &[crate::models::Branch],
    current_branch: &str,
    extra_protected: &[String],
) -> Vec<String> {
    branches
        .iter()
        .filter(|b| {
            if b.name == current_branch || is_protected_branch_with(&b.name, extra_protected) {
                return false;
            }

            let is_gone = b.status.contains(&crate::models::BranchStatus::Gone);
            let has_unique = b
                .status
                .contains(&crate::models::BranchStatus::HasUniqueCommits);
            let has_stash = b.status.contains(&crate::models::BranchStatus::Stashed);

            is_gone && !has_unique && !has_stash
        })
        .map(|b| b.name.clone())
        .collect()
}

pub fn prune_branches(
    path: &str,
    branches: &[crate::models::Branch],
    current_branch: &str,
    extra_protected: &[String],
) -> String {
    let to_delete = get_prunable_branches(branches, current_branch, extra_protected);

    if to_delete.is_empty() {
        return "No safe branches found to prune.".to_string();
    }

    let count = to_delete.len();
    let details = bulk_delete_branches(path, &to_delete, extra_protected);
    format!("Pruned {count} branches:\n{details}")
}

pub fn apply_stash(path: &str, id: &str) -> String {
    match run_git(path, &["stash", "apply", id]) {
        Ok(output) => format!("> git stash apply {}\n{}", id, output.trim()),
        Err(e) => format!("Error applying stash {id}: {e}"),
    }
}

pub fn stage_file(path: &str, file_path: &str) -> Result<String, String> {
    match run_git(path, &["add", file_path]) {
        Ok(_) => Ok(format!("Staged {file_path}")),
        Err(e) => Err(format!("Error staging {file_path}: {e}")),
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
        Ok(_) => Ok(format!("Unstaged {file_path}")),
        Err(e) => Err(format!("Error unstaging {file_path}: {e}")),
    }
}

pub fn get_commit_file_diff(path: &str, hash: &str, file_path: &str) -> Result<String, GitError> {
    run_git(path, &["show", "--format=", "-p", hash, "--", file_path])
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
            &format!("{hash}..HEAD"),
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
        &["checkout", &format!("{hash}^"), "--", file_rel_path],
    );
    // b. Fixup the target commit to remove those changes
    let _ = run_git(path, &["commit", "--fixup", hash]);

    // c. Restore file to state at target commit (reintroducing the changes as floating)
    let _ = run_git(path, &["checkout", hash, "--", file_rel_path]);

    let result_msg = if let Some(next_h) = next_commit_hash {
        // d. Fixup the next commit to absorb these changes
        let _ = run_git(path, &["commit", "--fixup", &next_h]);
        format!("Moving changes from {hash} to next commit {next_h}.")
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
            &format!("{hash}^"),
        ],
    ) {
        Ok(_) => Ok(format!("Success! {result_msg}")),
        Err(e) => Err(format!(
            "Rebase failed: {e}. Manual resolution might be needed."
        )),
    }
}

pub fn execute_interactive_rebase(path: &str, commits: &[RebaseCommit]) {
    use std::io::Write;

    let git_dir = crate::git::commands::get_git_dir(path)
        .unwrap_or_else(|| std::path::Path::new(path).join(".git"));
    let editor_script_path = git_dir.join("twigdrop-rebase-editor.sh");

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
                commit.original_message.replace('\'', "'\\''")
            ));
            let new_msg = commit
                .new_message
                .as_ref()
                .unwrap_or(&commit.original_message);
            script_content.push_str(&format!(
                "x git commit --amend -m '{}'\n",
                new_msg.replace('\'', "'\\''")
            ));
        } else {
            script_content.push_str(&format!(
                "{} {} {}\n",
                action,
                commit.hash,
                commit.original_message.replace('\'', "'\\''")
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

/// Resolves the remote for `branch`: its configured upstream remote, else the
/// first remote in `git remote`. Returns `None` when the repo has no remotes.
pub fn resolve_remote(path: &str, branch: &str) -> Option<String> {
    let key = format!("branch.{branch}.remote");
    if let Ok(r) = run_git(path, &["config", "--get", &key])
        && !r.trim().is_empty()
    {
        return Some(r.trim().to_string());
    }
    run_git(path, &["remote"])
        .ok()?
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .map(str::to_string)
}

/// Checks out `branch`.
pub fn checkout_branch(path: &str, branch: &str) -> Result<String, GitError> {
    run_git(path, &["checkout", branch]).map(|s| s.trim().to_string())
}

/// Creates and switches to a new branch named `branch`.
pub fn create_branch(path: &str, branch: &str) -> Result<String, GitError> {
    run_git(path, &["checkout", "-b", branch]).map(|s| s.trim().to_string())
}

/// Renames `old_name` branch to `new_name`.
pub fn rename_branch(path: &str, old_name: &str, new_name: &str) -> Result<String, GitError> {
    run_git(path, &["branch", "-m", old_name, new_name]).map(|s| s.trim().to_string())
}

/// Merges `branch` into the current HEAD.
pub fn merge_branch(path: &str, branch: &str) -> Result<String, GitError> {
    run_git(path, &["merge", branch]).map(|s| s.trim().to_string())
}

/// Recreates a deleted branch pointing to `sha` (used for undo).
pub fn recreate_branch(path: &str, name: &str, sha: &str) -> Result<String, GitError> {
    run_git(path, &["branch", name, sha]).map(|s| s.trim().to_string())
}

/// Resolves the commit date in ISO 8601 format for `hash`.
pub fn resolve_commit_date(path: &str, hash: &str) -> Result<String, GitError> {
    run_git(path, &["log", "-1", "--format=%cI", hash]).map(|s| s.trim().to_string())
}

/// Resolves the parent commit hash (`hash^`).
pub fn resolve_parent_hash(path: &str, hash: &str) -> Result<String, GitError> {
    run_git(path, &["rev-parse", &format!("{hash}^")]).map(|s| s.trim().to_string())
}

/// Resolves the full 40-character SHA from a ref or short SHA.
pub fn resolve_full_hash(path: &str, short_hash: &str) -> Result<String, GitError> {
    run_git(path, &["rev-parse", short_hash]).map(|s| s.trim().to_string())
}

/// Amends the current commit's date.
pub fn amend_commit_date(path: &str, date_iso: &str) -> Result<String, GitError> {
    run_git(
        path,
        &["commit", "--amend", "--no-edit", "--date", date_iso],
    )
    .map(|s| s.trim().to_string())
}

/// Lists all git worktrees in porcelain format.
pub fn list_worktrees(path: &str) -> Result<String, GitError> {
    run_git(path, &["worktree", "list", "--porcelain"])
}

/// Runs an arbitrary git command specified by argument slices.
pub fn run_arbitrary_git(path: &str, args: &[&str]) -> Result<String, GitError> {
    run_git(path, args).map(|s| s.trim().to_string())
}

/// Pushes a stash with a custom message.
pub fn push_stash(path: &str, message: &str) -> Result<String, GitError> {
    run_git(path, &["stash", "push", "-m", message])
}

/// Returns the commit log between `upstream` and `HEAD` for interactive rebase.
pub fn get_rebase_commits_log(path: &str, upstream: &str) -> Result<String, GitError> {
    run_git(
        path,
        &["log", &format!("{upstream}..HEAD"), "--format=%h %s"],
    )
}

/// Returns commits in `range` with `%h|%s` format.
pub fn get_commits_in_range(path: &str, range: &str) -> Result<String, GitError> {
    run_git(path, &["log", "--format=%h|%s", range])
}

/// Returns the commit show diff for `hash`.
pub fn get_commit_show(path: &str, hash: &str) -> Result<String, GitError> {
    run_git(path, &["show", hash])
}

/// Squashes `short_hash` into its parent commit.
pub fn squash_into_parent(path: &str, short_hash: &str) -> Result<String, GitError> {
    let parent_hash = resolve_parent_hash(path, short_hash)?;
    if parent_hash.trim().is_empty() {
        return Err(GitError::Command(
            "Cannot squash: no parent commit found.".to_string(),
        ));
    }

    let git_dir = crate::git::commands::get_git_dir(path)
        .unwrap_or_else(|| std::path::Path::new(path).join(".git"));
    let editor_script_path = git_dir.join("twigdrop-squash-editor.sh");

    let script_content = format!(
        "#!/bin/sh\nawk '{{ if ($1 == \"pick\" && match($2, \"^{short_hash}\")) {{ $1 = \"squash\" }} print }}' \"$1\" > \"$1.tmp\" && mv \"$1.tmp\" \"$1\"\n"
    );

    let _ = std::fs::write(&editor_script_path, script_content);
    #[cfg(unix)]
    let _ = std::process::Command::new("chmod")
        .arg("+x")
        .arg(&editor_script_path)
        .status();

    let res = run_git(
        path,
        &[
            "-c",
            &format!("sequence.editor={}", editor_script_path.display()),
            "rebase",
            "-i",
            "--autostash",
            &format!("{short_hash}^^"),
        ],
    );
    let _ = std::fs::remove_file(&editor_script_path);
    res.map(|_| format!("Squashed {short_hash} into its parent."))
}

/// Updates commit date during an interactive rebase execution.
pub fn rebase_update_commit_date(
    path: &str,
    short_hash: &str,
    new_date: &str,
) -> Result<String, GitError> {
    let full_hash = resolve_full_hash(path, short_hash).unwrap_or_else(|_| short_hash.to_string());
    let exec_cmd = format!(
        "if [ \"$(git rev-parse HEAD)\" = \"{full_hash}\" ]; then GIT_COMMITTER_DATE=\"{new_date}\" git commit --amend --no-edit --date=\"{new_date}\"; fi"
    );

    run_git(
        path,
        &[
            "rebase",
            "--autostash",
            &format!("{short_hash}^"),
            "--exec",
            &exec_cmd,
        ],
    )
}

/// Pushes `branch` to its remote (resolved via [`resolve_remote`]).
pub fn push_branch(path: &str, branch: &str, set_upstream: bool) -> Result<String, GitError> {
    let remote = resolve_remote(path, branch).ok_or(GitError::NoRemote)?;
    let mut args = vec!["push"];
    if set_upstream {
        args.push("-u");
    }
    args.push(&remote);
    args.push(branch);
    run_git(path, &args)
}

/// Creates a fixup commit for `hash` from the staged changes and autosquashes it.
pub fn fixup_commit(path: &str, hash: &str) -> Result<String, GitError> {
    run_git(path, &["commit", "--fixup", hash])?;
    let base = format!("{hash}^");
    run_git(
        path,
        &[
            "-c",
            "sequence.editor=:",
            "rebase",
            "-i",
            "--autosquash",
            "--autostash",
            &base,
        ],
    )
    .map(|_| format!("Amended to {hash}"))
}

pub fn abort_merge(path: &str) -> Result<String, GitError> {
    run_git(path, &["merge", "--abort"]).map(|out| format!("Merge aborted.\n{}", out.trim()))
}

pub fn abort_rebase(path: &str) -> Result<String, GitError> {
    run_git(path, &["rebase", "--abort"])
        .map(|out| format!("Rebase aborted successfully.\n{}", out.trim()))
}

pub fn continue_rebase(path: &str) -> Result<String, GitError> {
    run_git(path, &["rebase", "--continue"]).map(|out| format!("Rebase continued.\n{}", out.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Branch, BranchStatus, MergeStatus};

    fn make_test_branch(name: &str, status: Vec<BranchStatus>) -> Branch {
        Branch {
            name: name.to_string(),
            kind: crate::models::BranchKind::Real,
            status,
            merge_status: MergeStatus::NotAnalyzed,
            age: "1 day ago".to_string(),
            author: "Dev".to_string(),
            commit_date: "2026-09-15".to_string(),
            ahead_count: 0,
            behind_count: 0,
        }
    }

    #[test]
    fn test_get_prunable_branches_filtering() {
        let branches = vec![
            // Safe to prune: Gone without unique commits or stashes
            make_test_branch("feature/done", vec![BranchStatus::Gone]),
            // Must NOT prune: currently checked out branch even if gone
            make_test_branch("main", vec![BranchStatus::Gone]),
            // Must NOT prune: has unique commits (unpushed work!)
            make_test_branch(
                "feature/unpushed",
                vec![BranchStatus::Gone, BranchStatus::HasUniqueCommits],
            ),
            // Must NOT prune: has associated stash
            make_test_branch(
                "feature/stashed",
                vec![BranchStatus::Gone, BranchStatus::Stashed],
            ),
            // Must NOT prune: normal active local branch
            make_test_branch("feature/active", vec![BranchStatus::Local]),
        ];

        let prunable = get_prunable_branches(&branches, "main", &[]);
        assert_eq!(prunable, vec!["feature/done".to_string()]);
    }

    #[test]
    fn test_get_prunable_branches_empty_when_no_gone() {
        let branches = vec![
            make_test_branch("dev", vec![BranchStatus::RemoteTracked]),
            make_test_branch("main", vec![BranchStatus::Local]),
        ];

        let prunable = get_prunable_branches(&branches, "main", &[]);
        assert!(prunable.is_empty());
    }

    #[test]
    fn test_is_protected_branch() {
        assert!(is_protected_branch("main"));
        assert!(is_protected_branch("master"));
        assert!(is_protected_branch("develop"));
        assert!(is_protected_branch("dev"));
        assert!(is_protected_branch("staging"));
        assert!(is_protected_branch("production"));
        assert!(is_protected_branch("* main"));
        assert!(is_protected_branch("MAIN"));
        assert!(!is_protected_branch("feature/login"));
        assert!(!is_protected_branch("bugfix/issue-123"));
    }

    #[test]
    fn test_get_prunable_branches_skips_protected() {
        let branches = vec![
            // Gone, but protected -> MUST NOT prune
            make_test_branch("master", vec![BranchStatus::Gone]),
            make_test_branch("develop", vec![BranchStatus::Gone]),
            make_test_branch("staging", vec![BranchStatus::Gone]),
            // Gone and not protected -> safe to prune
            make_test_branch("feature/old", vec![BranchStatus::Gone]),
        ];

        let prunable = get_prunable_branches(&branches, "other-branch", &[]);
        assert_eq!(prunable, vec!["feature/old".to_string()]);
    }

    #[test]
    fn test_bulk_delete_branches_skips_protected() {
        let res = bulk_delete_branches(".", &["main".to_string(), "develop".to_string()], &[]);
        assert!(res.contains("Skipping protected branch: main"));
        assert!(res.contains("Skipping protected branch: develop"));
    }

    #[test]
    fn test_extra_protected_branches_respected() {
        let extra = vec!["release".to_string()];
        assert!(is_protected_branch_with("release", &extra));
        assert!(is_protected_branch_with("* release", &extra));
        assert!(is_protected_branch_with("RELEASE", &extra));
        assert!(!is_protected_branch_with("feature/x", &extra));
        // Defaults still apply alongside extras.
        assert!(is_protected_branch_with("main", &extra));

        let branches = vec![
            make_test_branch("release", vec![BranchStatus::Gone]),
            make_test_branch("feature/old", vec![BranchStatus::Gone]),
        ];
        let prunable = get_prunable_branches(&branches, "other-branch", &extra);
        assert_eq!(prunable, vec!["feature/old".to_string()]);
    }

    #[test]
    fn test_repo_slug_is_filesystem_safe() {
        assert_eq!(repo_slug("/Users/a/b"), "_Users_a_b");
        assert!(repo_slug("/a/b").len() <= 120);
    }

    #[test]
    fn test_write_backup_line_appends() {
        let dir = tempfile::TempDir::new().expect("tempdir");
        let file = write_backup_line(dir.path(), "slug", "feature-x", "abc123").expect("write");
        let file2 = write_backup_line(dir.path(), "slug", "feature-y", "def456").expect("write");
        assert_eq!(file, file2, "same-day backups share one log file");
        let content = std::fs::read_to_string(&file).expect("read");
        assert!(content.contains("feature-x abc123"));
        assert!(content.contains("feature-y def456"));
    }
}
