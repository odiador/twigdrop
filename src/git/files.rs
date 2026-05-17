use std::path::PathBuf;
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FileStatus {
    Modified,
    Added,
    Staged,
    Untracked,
    Ignored,
    Deleted,
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
    let output = Command::new("git")
        .current_dir(path)
        .args(["status", "--porcelain", "--ignored"])
        .output();

    let mut map = std::collections::HashMap::new();
    let ok_output = match output {
        Ok(o) => o,
        Err(_) => return map,
    };

    let stdout = String::from_utf8_lossy(&ok_output.stdout);

    for line in stdout.lines() {
        if line.len() < 4 {
            continue;
        }
        let status_code = &line[0..2];
        let file_path = line[3..].trim_matches('"');

        let status = match status_code {
            " M" | "M " | "MM" => FileStatus::Modified,
            " A" | "A " => FileStatus::Added,
            "??" => FileStatus::Untracked,
            "!!" => FileStatus::Ignored,
            " D" | "D " => FileStatus::Deleted,
            _ => {
                if status_code.starts_with('M') || status_code.ends_with('M') {
                    FileStatus::Modified
                } else if status_code.starts_with('A') || status_code.ends_with('A') {
                    FileStatus::Added
                } else {
                    FileStatus::Normal
                }
            }
        };

        let refined_status = if status_code.starts_with('M')
            || status_code.starts_with('A')
            || status_code.starts_with('D')
        {
            if status_code.ends_with(' ') {
                FileStatus::Staged
            } else {
                status
            }
        } else {
            status
        };

        map.insert(file_path.to_string(), refined_status.clone());

        // Propagate status to parent directories
        let path_obj = std::path::Path::new(file_path);
        for ancestor in path_obj.ancestors().skip(1) {
            let ancestor_str = ancestor.to_string_lossy().to_string();
            if ancestor_str.is_empty() || ancestor_str == "." {
                continue;
            }

            let entry = map.entry(ancestor_str).or_insert(FileStatus::Normal);

            // Priority: Staged > Modified > Added > Untracked > Deleted > Ignored > Normal
            match (&refined_status, &entry) {
                (FileStatus::Staged, _) => *entry = FileStatus::Staged,
                (FileStatus::Modified, FileStatus::Staged) => {}
                (FileStatus::Modified, _) => *entry = FileStatus::Modified,
                (FileStatus::Added, FileStatus::Staged | FileStatus::Modified) => {}
                (FileStatus::Added, _) => *entry = FileStatus::Added,
                (
                    FileStatus::Untracked,
                    FileStatus::Staged | FileStatus::Modified | FileStatus::Added,
                ) => {}
                (FileStatus::Untracked, _) => *entry = FileStatus::Untracked,
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
    let path = PathBuf::from(root).join(current_dir);

    if let Ok(read_dir) = std::fs::read_dir(path) {
        let mut dir_entries: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
        // Sort: dirs first, then files
        dir_entries.sort_by(|a, b| {
            let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
            let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
            if a_is_dir != b_is_dir {
                b_is_dir.cmp(&a_is_dir)
            } else {
                a.file_name().cmp(&b.file_name())
            }
        });

        for entry in dir_entries {
            let file_name = entry.file_name().to_string_lossy().to_string();
            if file_name == ".git" || file_name == "target" {
                continue;
            }

            let rel_path = if current_dir.is_empty() {
                file_name.clone()
            } else {
                format!("{}/{}", current_dir, file_name)
            };

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
    }
    entries
}
