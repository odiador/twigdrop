//! GitHub CLI (`gh`) interaction module.
//!
//! Provides non-blocking execution of GitHub CLI commands for listing pull requests,
//! fetching PR unified diffs, checking out PR branches, and opening PRs in the browser.

use crate::git::error::GitError;
use crate::models::{PrCheck, PullRequest};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewAction {
    Approve,
    RequestChanges,
    Comment,
}

/// Retrieves CI status checks for a pull request using `gh pr checks`.
pub fn get_pull_request_checks(path: &str, number: u64) -> Result<Vec<PrCheck>, GitError> {
    let output = Command::new("gh")
        .current_dir(path)
        .args([
            "pr",
            "checks",
            &number.to_string(),
            "--json",
            "bucket,name,state,description,link",
        ])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.contains("no checks reported") || stderr.contains("no checks") {
            return Ok(Vec::new());
        }
        return Err(GitError::Command(format!("gh pr checks failed: {stderr}")));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let checks: Vec<PrCheck> = serde_json::from_str(&stdout).unwrap_or_default();
    Ok(checks)
}

/// Submits a review on a pull request using `gh pr review`.
pub fn review_pull_request(
    path: &str,
    number: u64,
    action: ReviewAction,
    body: Option<&str>,
) -> Result<String, GitError> {
    let mut cmd = Command::new("gh");
    cmd.current_dir(path)
        .args(["pr", "review", &number.to_string()]);

    match action {
        ReviewAction::Approve => {
            cmd.arg("--approve");
            if let Some(b) = body {
                let trimmed = b.trim();
                if !trimmed.is_empty() {
                    cmd.args(["-b", trimmed]);
                }
            }
        }
        ReviewAction::RequestChanges => {
            cmd.arg("--request-changes");
            let b = body.unwrap_or("Changes requested");
            cmd.args(["-b", b]);
        }
        ReviewAction::Comment => {
            cmd.arg("--comment");
            let b = body.unwrap_or("");
            cmd.args(["-b", b]);
        }
    }

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GitError::Command(format!("gh pr review failed: {stderr}")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Posts a general comment on a pull request using `gh pr comment`.
pub fn comment_pull_request(path: &str, number: u64, body: &str) -> Result<String, GitError> {
    let output = Command::new("gh")
        .current_dir(path)
        .args(["pr", "comment", &number.to_string(), "-b", body])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(GitError::Command(format!("gh pr comment failed: {stderr}")));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
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

    #[test]
    fn test_deserialize_checks_json() {
        let json_data = r#"[
            {
                "name": "cargo test",
                "state": "SUCCESS",
                "bucket": "pass",
                "description": "Passed in 1m 20s",
                "link": "https://github.com/checks/1"
            },
            {
                "name": "clippy",
                "state": "FAILURE",
                "bucket": "fail",
                "description": "Found warnings",
                "link": "https://github.com/checks/2"
            }
        ]"#;

        let checks: Vec<PrCheck> = serde_json::from_str(json_data).unwrap();
        assert_eq!(checks.len(), 2);
        assert_eq!(checks[0].name, "cargo test");
        assert_eq!(checks[0].bucket, "pass");
        assert_eq!(checks[1].bucket, "fail");

        let status = crate::models::compute_ci_status(&checks);
        assert_eq!(status, crate::models::CiStatus::Failing);

        let passing_status = crate::models::compute_ci_status(&checks[0..1]);
        assert_eq!(passing_status, crate::models::CiStatus::Passing);

        let empty_status = crate::models::compute_ci_status(&[]);
        assert_eq!(empty_status, crate::models::CiStatus::Unknown);
    }
}
