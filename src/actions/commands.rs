use crate::git::commands::run_git;

pub fn delete_branch(path: &str, name: &str) -> String {
    let output = run_git(path, &["branch", "-D", name]);
    output.trim().to_string()
}

pub fn checkout_branch(path: &str, name: &str) -> String {
    let output = run_git(path, &["checkout", name]);
    output.trim().to_string()
}
