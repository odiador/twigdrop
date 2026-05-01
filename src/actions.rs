use std::process::Command;

pub fn delete_branch(path: &str, name: &str) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args(["branch", "-D", name])
        .output()
        .expect("failed to execute git branch -D");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{}{}", stdout, stderr).trim().to_string()
}

pub fn checkout_branch(path: &str, name: &str) -> String {
    let output = Command::new("git")
        .current_dir(path)
        .args(["checkout", name])
        .output()
        .expect("failed to execute git checkout");
        
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    format!("{}{}", stdout, stderr).trim().to_string()
}
