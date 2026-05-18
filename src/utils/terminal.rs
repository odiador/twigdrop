use std::path::Path;
use std::process::Command;

pub fn open_terminal(dir: &Path) {
    if is_ghostty_available() {
        let _ = Command::new("ghostty")
            .arg("-e")
            .arg("bash")
            .current_dir(dir)
            .spawn();
    } else {
        // Fallback for macOS
        let _ = Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(dir)
            .spawn();
    }
}

pub fn open_ide(dir: &Path, command: &str) {
    let _ = Command::new("sh")
        .arg("-c")
        .arg(format!("{} {}", command, dir.display()))
        .spawn();
}

fn is_ghostty_available() -> bool {
    Command::new("ghostty").arg("--version").output().is_ok()
}
