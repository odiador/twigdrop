use crate::git::commands::run_git;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileStatus {
    Modified,
    Added,
    Staged,
    Untracked,
    Ignored,
    Deleted,
    Conflict,
    Normal,
}

pub struct FileEntry {
    pub path: PathBuf,
    pub is_dir: bool,
    pub status: FileStatus,
    pub is_open: bool,
    pub depth: usize,
}

pub fn get_git_file_statuses(path: &str) -> std::collections::HashMap<String, FileStatus> {
    let mut map = std::collections::HashMap::new();
    let stdout = match run_git(path, &["status", "--porcelain", "--ignored"]) {
        Ok(out) => out,
        Err(_) => return map,
    };

    for line in stdout.lines() {
        if line.len() < 4 {
            continue;
        }
        let status_code = &line[0..2];
        let mut file_path = line[3..].to_string();
        
        // Handle renames "R  old -> new" or copies "C  old -> new"
        if status_code.starts_with('R') || status_code.starts_with('C') {
            if let Some(pos) = file_path.find(" -> ") {
                file_path = file_path[pos + 4..].trim_matches('"').to_string();
            }
        } else {
            file_path = file_path.trim_matches('"').to_string();
        }

        let status = match status_code {
            "DD" | "AU" | "UD" | "UA" | "DU" | "AA" | "UU" => FileStatus::Conflict,
            _ => {
                if status_code.contains('U') {
                    FileStatus::Conflict
                } else if status_code.contains('M') || status_code.contains('R') || status_code.contains('C') || status_code.contains('T') {
                    FileStatus::Modified
                } else if status_code.contains('A') {
                    FileStatus::Added
                } else if status_code.contains('D') {
                    FileStatus::Deleted
                } else if status_code == "??" {
                    FileStatus::Untracked
                } else if status_code == "!!" {
                    FileStatus::Ignored
                } else {
                    FileStatus::Normal
                }
            }
        };

        // Determine if it's staged vs worktree change
        // In porcelain v1: XY where X is index, Y is worktree
        let refined_status = if status == FileStatus::Untracked || status == FileStatus::Ignored || status == FileStatus::Conflict {
            status
        } else {
            let worktree_char = status_code.chars().nth(1).unwrap_or(' ');
            if worktree_char != ' ' {
                // There is a change in the worktree (unstaged)
                status
            } else {
                // Change is only in the index (staged)
                FileStatus::Staged
            }
        };

        map.insert(file_path.clone(), refined_status.clone());

        // Propagate status to parent directories
        let path_obj = Path::new(&file_path);
        for ancestor in path_obj.ancestors().skip(1) {
            let ancestor_str = ancestor.to_string_lossy().to_string();
            if ancestor_str.is_empty() || ancestor_str == "." {
                continue;
            }

            let entry = map.entry(ancestor_str).or_insert(FileStatus::Normal);

            // Priority: Conflict > Modified > Added > Staged > Untracked > Deleted > Ignored > Normal
            match (&refined_status, &entry) {
                (FileStatus::Conflict, _) => *entry = FileStatus::Conflict,
                (_, FileStatus::Conflict) => {}
                (FileStatus::Modified, _) => *entry = FileStatus::Modified,
                (_, FileStatus::Modified) => {}
                (FileStatus::Added, _) => *entry = FileStatus::Added,
                (_, FileStatus::Added) => {}
                (FileStatus::Staged, _) => *entry = FileStatus::Staged,
                (_, FileStatus::Staged) => {}
                (FileStatus::Untracked, _) => *entry = FileStatus::Untracked,
                (_, FileStatus::Untracked) => {}
                (FileStatus::Deleted, _) => *entry = FileStatus::Deleted,
                (_, FileStatus::Deleted) => {}
                (FileStatus::Ignored, _) => *entry = FileStatus::Ignored,
                _ => {}
            }
        }
    }
    map
}

pub fn build_file_tree(
    root: &str,
    current_dir: &str,
    depth: usize,
    statuses: &std::collections::HashMap<String, FileStatus>,
) -> Vec<FileEntry> {
    let mut entries = vec![];
    let base_path = Path::new(root);
    let search_path = base_path.join(current_dir);

    // Use ignore crate to walk the directory one level deep
    let walker = WalkBuilder::new(&search_path)
        .max_depth(Some(1))
        .add_custom_ignore_filename(".twigignore")
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .require_git(false) // Still work even if not in a git repo
        .hidden(true) // Ignore hidden files by default
        .build();

    let mut dir_entries = vec![];
    for entry in walker.flatten() {
        let path = entry.path();

        // Skip the search_path itself
        if path == search_path {
            continue;
        }

        let file_name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Explicitly ignore .git and target if they aren't already ignored
        if file_name == ".git" || file_name == "target" {
            continue;
        }

        dir_entries.push(entry);
    }

    // Sort: dirs first, then files
    dir_entries.sort_by(|a, b| {
        let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
        if a_is_dir != b_is_dir {
            b_is_dir.cmp(&a_is_dir)
        } else {
            a.file_name().cmp(b.file_name())
        }
    });

    for entry in dir_entries {
        let path = entry.path();
        let rel_path = path
            .strip_prefix(base_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let is_dir = entry.file_type().map(|t| t.is_dir()).unwrap_or(false);
        let status = statuses
            .get(&rel_path)
            .cloned()
            .unwrap_or(FileStatus::Normal);

        entries.push(FileEntry {
            path: PathBuf::from(&rel_path),
            is_dir,
            status,
            is_open: false,
            depth,
        });
    }

    entries
}
