use crate::git::commands::run_git;

pub struct StashEntry {
    pub id: String,
    pub message: String,
    pub branch: String,
}

pub fn get_stashes(path: &str) -> Vec<StashEntry> {
    let out = run_git(path, &["stash", "list", "--format=%gd|%s"]);
    let mut stashes = vec![];
    for line in out.lines() {
        let parts: Vec<&str> = line.split('|').collect();
        if parts.len() >= 2 {
            let id = parts[0].to_string();
            let message = parts[1].to_string();

            // Extract branch from message like "WIP on branch_name: ..."
            let branch = if let Some(on_idx) = message.find("on ") {
                let sub = &message[on_idx + 3..];
                if let Some(colon_idx) = sub.find(':') {
                    sub[..colon_idx].trim().to_string()
                } else {
                    "unknown".to_string()
                }
            } else {
                "unknown".to_string()
            };

            stashes.push(StashEntry {
                id,
                message,
                branch,
            });
        }
    }
    stashes
}

pub fn get_stash_files(path: &str, id: &str) -> Vec<String> {
    let out = run_git(path, &["stash", "show", "--name-only", id]);
    out.lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

pub fn get_stash_diff(path: &str, id: &str) -> String {
    run_git(path, &["stash", "show", "-p", "--color=never", id])
}
