use crate::git::commands::run_git;

pub fn delete_branch(path: &str, name: &str) -> String {
    let output = run_git(path, &["branch", "-D", name]);
    format!("> git branch -D {}\n{}", name, output.trim())
}

pub fn checkout_branch(path: &str, name: &str) -> String {
    let output = run_git(path, &["checkout", name]);
    format!("> git checkout {}\n{}", name, output.trim())
}

pub fn bulk_delete_branches(path: &str, names: &[String]) -> String {
    let mut results = String::new();
    for name in names {
        let output = run_git(path, &["branch", "-D", name]);
        results.push_str(&format!("> git branch -D {}\n{}\n", name, output.trim()));
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
    let output = run_git(path, &["stash", "apply", id]);
    format!("> git stash apply {}\n{}", id, output.trim())
}
