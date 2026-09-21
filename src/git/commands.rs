use crate::models::CommitTreeItem;
use anyhow::{Result, anyhow};
use std::process::Command;

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

use std::path::{Path, PathBuf};

pub fn is_inside_git_work_tree(path: &str) -> bool {
    match run_git(path, &["rev-parse", "--is-inside-work-tree"]) {
        Ok(out) => out.trim() == "true",
        Err(_) => false,
    }
}

pub fn get_git_dir(path: &str) -> Option<PathBuf> {
    match run_git(path, &["rev-parse", "--git-dir"]) {
        Ok(out) => {
            let p = Path::new(out.trim());
            if p.is_absolute() {
                Some(p.to_path_buf())
            } else {
                Some(Path::new(path).join(p))
            }
        }
        Err(_) => None,
    }
}

pub fn get_commit_tree(path: &str) -> Vec<CommitTreeItem> {
    get_commit_tree_limited(path, 300)
}

pub fn get_commit_tree_limited(path: &str, limit: usize) -> Vec<CommitTreeItem> {
    let limit_str = format!("--max-count={}", limit);
    let args = [
        "log",
        &limit_str,
        "--graph",
        "--all",
        "--color=never",
        "--format=<|%h%d|%cd|%an|%s",
        "--date=short",
    ];

    let output = match run_git(path, &args) {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };

    output.lines().filter_map(parse_commit_tree_line).collect()
}

pub fn parse_commit_tree_line(line: &str) -> Option<CommitTreeItem> {
    if line.trim().is_empty() {
        return None;
    }

    if let Some(pos) = line.find("<|") {
        let (graph, rest) = line.split_at(pos);
        let rest = &rest[2..]; // skip "<|"
        let parts: Vec<&str> = rest.split('|').collect();

        if parts.len() >= 4 {
            let hash_and_refs = parts[0];
            let (hash, refs) = if let Some(p) = hash_and_refs.find('(') {
                (
                    hash_and_refs[..p].trim().to_string(),
                    hash_and_refs[p..].trim().to_string(),
                )
            } else {
                (hash_and_refs.to_string(), String::new())
            };

            Some(CommitTreeItem {
                graph: graph.to_string(),
                hash,
                branch_info: refs,
                date: parts[1].to_string(),
                author: parts[2].to_string(),
                message: parts[3..].join("|"),
            })
        } else {
            Some(CommitTreeItem {
                graph: line.to_string(),
                hash: String::new(),
                branch_info: String::new(),
                date: String::new(),
                author: String::new(),
                message: String::new(),
            })
        }
    } else {
        Some(CommitTreeItem {
            graph: line.to_string(),
            hash: String::new(),
            branch_info: String::new(),
            date: String::new(),
            author: String::new(),
            message: String::new(),
        })
    }
}

pub fn get_commit_details(path: &str, hash: &str) -> (String, String) {
    let stat_args = ["show", "--stat", "--format=", hash];
    let diff_args = ["show", "-p", "--format=", hash];

    let stats = match run_git(path, &stat_args) {
        Ok(out) => out,
        Err(_) => String::from("Could not load stats."),
    };

    let diff = match run_git(path, &diff_args) {
        Ok(out) => out,
        Err(_) => String::from("Could not load diff."),
    };

    (stats, diff)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_commit_tree_line_with_branch_info() {
        let line = "* | <|a1b2c3d (HEAD -> main, origin/main)|2026-09-16|Alice|feat: add awesome feature";
        let item = parse_commit_tree_line(line).unwrap();

        assert_eq!(item.graph, "* | ");
        assert_eq!(item.hash, "a1b2c3d");
        assert_eq!(item.branch_info, "(HEAD -> main, origin/main)");
        assert_eq!(item.date, "2026-09-16");
        assert_eq!(item.author, "Alice");
        assert_eq!(item.message, "feat: add awesome feature");
    }

    #[test]
    fn test_parse_commit_tree_line_without_branch_info() {
        let line = "* <|e4f5g6h|2026-09-15|Bob|fix: handle edge case";
        let item = parse_commit_tree_line(line).unwrap();

        assert_eq!(item.graph, "* ");
        assert_eq!(item.hash, "e4f5g6h");
        assert_eq!(item.branch_info, "");
        assert_eq!(item.date, "2026-09-15");
        assert_eq!(item.author, "Bob");
        assert_eq!(item.message, "fix: handle edge case");
    }

    #[test]
    fn test_parse_commit_tree_line_graph_only() {
        let line = "| \\";
        let item = parse_commit_tree_line(line).unwrap();

        assert_eq!(item.graph, "| \\");
        assert_eq!(item.hash, "");
        assert_eq!(item.message, "");
    }

    #[test]
    fn test_parse_commit_tree_line_empty() {
        assert_eq!(parse_commit_tree_line(""), None);
        assert_eq!(parse_commit_tree_line("   \n"), None);
    }

    #[test]
    fn test_is_inside_git_work_tree() {
        // Current directory is a git repo
        assert!(is_inside_git_work_tree("."));

        // Temp dir is not a git repo
        let temp = tempfile::tempdir().unwrap();
        assert!(!is_inside_git_work_tree(temp.path().to_str().unwrap()));
    }

    #[test]
    fn test_get_git_dir() {
        // Current repo has a valid git dir that exists
        let git_dir = get_git_dir(".").expect("Should resolve git-dir for current repo");
        assert!(git_dir.exists());

        // Temp dir returns None
        let temp = tempfile::tempdir().unwrap();
        assert!(get_git_dir(temp.path().to_str().unwrap()).is_none());
    }

    #[test]
    fn test_get_commit_tree_limited() {
        let tree = get_commit_tree_limited(".", 2);
        assert!(tree.len() <= 2);
    }
}
