//! GitHub CLI (`gh`) interaction module.
//!
//! Provides non-blocking execution of GitHub CLI commands for listing pull requests,
//! fetching PR unified diffs, checking out PR branches, and opening PRs in the browser.

use crate::git::error::GitError;
use crate::models::PullRequest;
use std::process::Command;

/// Checks if GitHub CLI `gh` is installed and runnable on the system.
#[must_use]
pub fn check_gh_available() -> bool {
    Command::new("gh")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Lists pull requests for the repository at `path` using `gh pr list`.
/// `state` can be "open", "all", "merged", or "closed".
pub fn list_pull_requests(path: &str, state: &str) -> Result<Vec<PullRequest>, GitError> {
    let output = Command::new("gh")
        .current_dir(path)
        .args([
            "pr",
            "list",
            "--state",
            state,
            "--limit",
            "50",
            "--json",
            "number,title,author,headRefName,baseRefName,state,updatedAt,url,body",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let err_msg = if !stderr.is_empty() { stderr } else { stdout };
        return Err(GitError::Command(format!("GitHub CLI error: {err_msg}")));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let prs: Vec<PullRequest> = serde_json::from_str(&stdout)
        .map_err(|e| GitError::Command(format!("Failed to parse PR JSON: {e}")))?;
    Ok(prs)
}

/// Retrieves the unified diff of a pull request using `gh pr diff`.
pub fn get_pull_request_diff(path: &str, number: u64) -> Result<String, GitError> {
    let output = Command::new("gh")
        .current_dir(path)
        .args(["pr", "diff", &number.to_string()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GitError::Command(format!("gh pr diff failed: {stderr}")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Checks out the branch associated with a pull request using `gh pr checkout`.
pub fn checkout_pull_request(path: &str, number: u64) -> Result<String, GitError> {
    let output = Command::new("gh")
        .current_dir(path)
        .args(["pr", "checkout", &number.to_string()])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GitError::Command(format!(
            "gh pr checkout failed: {stderr}"
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Opens the pull request in the system's default web browser using `gh pr view --web`.
pub fn open_pull_request_web(path: &str, number: u64) -> Result<(), GitError> {
    let output = Command::new("gh")
        .current_dir(path)
        .args(["pr", "view", &number.to_string(), "--web"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GitError::Command(format!(
            "gh pr view --web failed: {stderr}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_sample_pr_json() {
        let json_data = r##"[
            {
                "number": 101,
                "title": "feat: native terminal markdown rendering",
                "author": { "login": "odiador" },
                "headRefName": "feat/markdown-engine",
                "baseRefName": "main",
                "state": "OPEN",
                "updatedAt": "2026-10-25T16:00:00Z",
                "url": "https://github.com/odiador/twigdrop/pull/101",
                "body": "# Description\nAdd markdown rendering support."
            }
        ]"##;

        let prs: Vec<PullRequest> = serde_json::from_str(json_data).unwrap();
        assert_eq!(prs.len(), 1);
        assert_eq!(prs[0].number, 101);
        assert_eq!(prs[0].author.login, "odiador");
        assert_eq!(prs[0].head_ref_name, "feat/markdown-engine");
        assert_eq!(prs[0].state, "OPEN");
        assert!(prs[0].body.contains("# Description"));
    }
}
