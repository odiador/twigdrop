use std::process::Command;

pub fn run_git(path: &str, args: &[&str]) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .expect("git failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    format!("{}{}", stdout, stderr).to_string()
}

pub fn run_git_with_status(path: &str, args: &[&str]) -> (String, i32) {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .expect("git failed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    (format!("{}{}", stdout, stderr), exit_code)
}
