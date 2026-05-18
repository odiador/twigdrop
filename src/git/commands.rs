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
