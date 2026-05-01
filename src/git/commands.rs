use std::process::Command;

pub fn run_git(path: &str, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .expect("git failed");

    String::from_utf8_lossy(&output.stdout).to_string()
}
