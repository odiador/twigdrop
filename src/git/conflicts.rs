use crate::git::commands::run_git;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConflictHunk {
    pub hunk_idx: usize,
    pub our_label: String,
    pub our_lines: Vec<String>,
    pub their_label: String,
    pub their_lines: Vec<String>,
    pub raw_block: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictChoice {
    Ours,
    Theirs,
    Both,
}

/// Parses all standard git conflict blocks (`<<<<<<<`, `=======`, `>>>>>>>`) in a file.
pub fn parse_file_conflicts(repo_path: &str, file_rel_path: &str) -> Vec<ConflictHunk> {
    let full_path = Path::new(repo_path).join(file_rel_path);
    let content = match fs::read_to_string(&full_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut hunks = Vec::new();
    let mut in_conflict = false;
    let mut in_theirs = false;

    let mut our_label = String::new();
    let mut our_lines = Vec::new();
    let mut their_lines = Vec::new();
    let mut raw_block = String::new();
    let mut hunk_counter = 0;

    for line in content.lines() {
        if line.starts_with("<<<<<<<") {
            in_conflict = true;
            in_theirs = false;
            our_label = line.trim_start_matches("<<<<<<<").trim().to_string();
            our_lines.clear();
            their_lines.clear();
            raw_block.clear();
            raw_block.push_str(line);
            raw_block.push('\n');
        } else if in_conflict && line.starts_with("=======") {
            in_theirs = true;
            raw_block.push_str(line);
            raw_block.push('\n');
        } else if in_conflict && line.starts_with(">>>>>>>") {
            let their_label = line.trim_start_matches(">>>>>>>").trim().to_string();
            raw_block.push_str(line);
            raw_block.push('\n');

            hunks.push(ConflictHunk {
                hunk_idx: hunk_counter,
                our_label: our_label.clone(),
                our_lines: our_lines.clone(),
                their_label,
                their_lines: their_lines.clone(),
                raw_block: raw_block.clone(),
            });

            hunk_counter += 1;
            in_conflict = false;
            in_theirs = false;
            raw_block.clear();
        } else if in_conflict {
            raw_block.push_str(line);
            raw_block.push('\n');
            if in_theirs {
                their_lines.push(line.to_string());
            } else {
                our_lines.push(line.to_string());
            }
        }
    }

    hunks
}

/// Resolves a single conflict hunk in the file according to the chosen strategy:
/// `ConflictChoice::Ours`, `ConflictChoice::Theirs`, or `ConflictChoice::Both`.
///
/// Automatically stages the file (`git add`) if no more conflict markers remain.
/// Returns `Ok(true)` if the file is now fully resolved and staged, or `Ok(false)` if more hunks remain.
pub fn resolve_hunk_in_file(
    repo_path: &str,
    file_rel_path: &str,
    hunk: &ConflictHunk,
    choice: ConflictChoice,
) -> Result<bool, String> {
    let full_path = Path::new(repo_path).join(file_rel_path);
    let content = fs::read_to_string(&full_path).map_err(|e| e.to_string())?;

    let replacement = match choice {
        ConflictChoice::Ours => {
            if hunk.our_lines.is_empty() {
                String::new()
            } else {
                format!("{}\n", hunk.our_lines.join("\n"))
            }
        }
        ConflictChoice::Theirs => {
            if hunk.their_lines.is_empty() {
                String::new()
            } else {
                format!("{}\n", hunk.their_lines.join("\n"))
            }
        }
        ConflictChoice::Both => {
            let mut s = String::new();
            if !hunk.our_lines.is_empty() {
                s.push_str(&hunk.our_lines.join("\n"));
                s.push('\n');
            }
            if !hunk.their_lines.is_empty() {
                s.push_str(&hunk.their_lines.join("\n"));
                s.push('\n');
            }
            s
        }
    };

    if let Some(pos) = content.find(&hunk.raw_block) {
        let mut new_content = content.clone();
        new_content.replace_range(pos..pos + hunk.raw_block.len(), &replacement);
        fs::write(&full_path, &new_content).map_err(|e| e.to_string())?;

        let fully_resolved = !new_content.contains("<<<<<<<");
        if fully_resolved {
            let _ = run_git(repo_path, &["add", file_rel_path]);
        }
        Ok(fully_resolved)
    } else {
        Err("Conflict block not found in file (may have already been edited)".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_and_resolve_conflicts() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("sample.txt");
        let initial_content = "\
header
<<<<<<< HEAD
local line 1
local line 2
=======
incoming line 1
>>>>>>> feature
footer
";
        fs::write(&file_path, initial_content).unwrap();

        let p = dir.path().to_string_lossy().to_string();
        let hunks = parse_file_conflicts(&p, "sample.txt");
        assert_eq!(hunks.len(), 1);
        assert_eq!(hunks[0].our_label, "HEAD");
        assert_eq!(hunks[0].their_label, "feature");
        assert_eq!(hunks[0].our_lines, vec!["local line 1", "local line 2"]);
        assert_eq!(hunks[0].their_lines, vec!["incoming line 1"]);

        // Resolve choosing Ours
        let fully_resolved =
            resolve_hunk_in_file(&p, "sample.txt", &hunks[0], ConflictChoice::Ours).unwrap();
        assert!(fully_resolved);

        let final_content = fs::read_to_string(&file_path).unwrap();
        assert!(final_content.contains("local line 1\nlocal line 2"));
        assert!(!final_content.contains("incoming line 1"));
        assert!(!final_content.contains("<<<<<<<"));
    }

    #[test]
    fn test_resolve_theirs_and_both() {
        let dir = TempDir::new().unwrap();
        let file_path = dir.path().join("both.txt");
        let initial_content = "\
<<<<<<< ours
A
=======
B
>>>>>>> theirs
";
        fs::write(&file_path, initial_content).unwrap();

        let p = dir.path().to_string_lossy().to_string();
        let hunks = parse_file_conflicts(&p, "both.txt");
        assert_eq!(hunks.len(), 1);

        // Resolve choosing Both
        let fully_resolved =
            resolve_hunk_in_file(&p, "both.txt", &hunks[0], ConflictChoice::Both).unwrap();
        assert!(fully_resolved);

        let final_content = fs::read_to_string(&file_path).unwrap();
        assert!(final_content.contains("A\nB"));
        assert!(!final_content.contains("<<<<<<<"));
    }
}
