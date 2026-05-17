use std::process::Command;

pub fn open_terminal(dir: &std::path::Path) {
    if is_ghostty_available() {
        let _ = Command::new("ghostty")
            .arg("-e")
            .arg(format!("cd {}; exec $SHELL", dir.to_string_lossy()))
            .spawn();
        return;
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open")
            .arg("-a")
            .arg("Terminal")
            .arg(dir)
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
                .arg(dir)
                .spawn();
        } else if Command::new("x-terminal-emulator")
            .arg("--version")
            .output()
            .is_ok()
        {
            let _ = Command::new("x-terminal-emulator")
                .arg("-e")
                .arg(format!("cd {}; exec $SHELL", dir.to_string_lossy()))
                .spawn();
        }
    }
}

fn is_ghostty_available() -> bool {
    Command::new("ghostty").arg("--version").output().is_ok()
}
