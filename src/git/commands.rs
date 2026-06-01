use anyhow::{Result, anyhow};
use std::process::Command;
use crate::models::CommitTreeItem;

pub fn run_git(path: &str, args: &[&str]) -> Result<String> {
    let output = Command::new("git").current_dir(path).args(args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(anyhow!("git error: {}", stderr))
    }
}

pub fn run_git_with_status(path: &str, args: &[&str]) -> Result<(String, i32)> {
    let output = Command::new("git").current_dir(path).args(args).output()?;

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let exit_code = output.status.code().unwrap_or(-1);

    if output.status.success() {
        Ok((stdout, exit_code))
    } else {
        Ok((format!("{}{}", stdout, stderr), exit_code))
    }
}

pub fn get_commit_tree(path: &str) -> Vec<CommitTreeItem> {
    let args = [
        "log",
        "--graph",
        "--all",
        "--color=never",
        "--format=<|%h|%cd|%an|%s",
        "--date=short",
    ];

    let output = match run_git(path, &args) {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    let mut items = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Some(pos) = line.find("<|") {
            let (graph, rest) = line.split_at(pos);
            let rest = &rest[2..]; // skip "<|"
            let parts: Vec<&str> = rest.split('|').collect();

            if parts.len() >= 4 {
                items.push(CommitTreeItem {
                    graph: graph.to_string(),
                    hash: parts[0].to_string(),
                    date: parts[1].to_string(),
                    author: parts[2].to_string(),
                    message: parts[3..].join("|"),
                });
            } else {
                // Should not happen with this format, but fallback
                items.push(CommitTreeItem {
                    graph: line.to_string(),
                    hash: String::new(),
                    date: String::new(),
                    author: String::new(),
                    message: String::new(),
                });
            }
        } else {
            // Lines with only graph parts
            items.push(CommitTreeItem {
                graph: line.to_string(),
                hash: String::new(),
                date: String::new(),
                author: String::new(),
                message: String::new(),
            });
        }
    }
    items
}
