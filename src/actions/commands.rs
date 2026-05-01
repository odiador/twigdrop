use crate::git::commands::run_git;

pub fn delete_branch(path: &str, name: &str) -> String {
    let output = run_git(path, &["branch", "-D", name]);
    format!("> git branch -D {}\n{}", name, output.trim())
}

pub fn checkout_branch(path: &str, name: &str) -> String {
    let output = run_git(path, &["checkout", name]);
    format!("> git checkout {}\n{}", name, output.trim())
}
