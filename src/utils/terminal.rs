use std::path::Path;
use std::process::Command;

pub fn open_terminal(dir: &Path) {
    let abs_dir = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
    
    if is_ghostty_available() {
        let _ = Command::new("ghostty")
            .arg("--working-directory")
            .arg(&abs_dir)
            .spawn();
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(&abs_dir)
            .spawn();
    }

    #[cfg(target_os = "linux")]
    {
        // Try common linux terminals
        if Command::new("gnome-terminal")
            .arg("--version")
            .output()
            .is_ok()
        {
            let _ = Command::new("gnome-terminal")
                .arg("--working-directory")
                .arg(&abs_dir)
                .spawn();
        } else if Command::new("x-terminal-emulator")
            .arg("--version")
            .output()
            .is_ok()
        {
            let _ = Command::new("x-terminal-emulator")
                .arg("-e")
                .arg(format!("cd {}; exec $SHELL", abs_dir.to_string_lossy()))
                .spawn();
        }
    }
}

pub fn open_ide(dir: &Path, command: &str) {
    let _ = Command::new(command).arg(dir).spawn();
}

pub fn open_folder(dir: &Path) {
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(dir).spawn();
    }

    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(dir).spawn();
    }
}

fn is_ghostty_available() -> bool {
    Command::new("ghostty").arg("--version").output().is_ok()
}
