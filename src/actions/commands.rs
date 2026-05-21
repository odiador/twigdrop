use crate::git::commands::run_git;
use std::fs;

#[allow(dead_code)]
pub fn delete_branch(path: &str, name: &str) -> String {
    match run_git(path, &["branch", "-D", name]) {
        Ok(output) => format!("> git branch -D {}\n{}", name, output.trim()),
        Err(e) => format!("Error deleting branch {}: {}", name, e),
    }
}

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

pub fn apply_resolution_to_file(repo_path: &str, file_path: &str, original_block: &str, resolved_content: &str) -> Result<(), String> {
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
