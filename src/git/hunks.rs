use crate::git::commands::{run_git, run_git_with_stdin};
use crate::git::error::GitError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiffHunk {
    pub hunk_idx: usize,
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub header: String,
    pub raw_patch: String,
    pub is_staged: bool,
}

fn parse_range(s: &str) -> (usize, usize) {
    let clean = s.trim_start_matches(['-', '+']);
    if let Some((start_s, count_s)) = clean.split_once(',') {
        let start = start_s.parse::<usize>().unwrap_or(0);
        let count = count_s.parse::<usize>().unwrap_or(0);
        (start, count)
    } else {
        let start = clean.parse::<usize>().unwrap_or(0);
        (start, 1)
    }
}

pub fn parse_hunks_from_diff(diff_output: &str, is_staged: bool) -> Vec<DiffHunk> {
    if diff_output.trim().is_empty() {
        return Vec::new();
    }

    let mut header_lines = Vec::new();
    let mut hunk_blocks: Vec<Vec<String>> = Vec::new();
    let mut current_hunk: Vec<String> = Vec::new();

    for line in diff_output.lines() {
        if line.starts_with("@@") {
            if !current_hunk.is_empty() {
                hunk_blocks.push(current_hunk);
                current_hunk = Vec::new();
            }
            current_hunk.push(line.to_string());
        } else if !current_hunk.is_empty() {
            current_hunk.push(line.to_string());
        } else {
            header_lines.push(line.to_string());
        }
    }
    if !current_hunk.is_empty() {
        hunk_blocks.push(current_hunk);
    }

    let file_header = header_lines.join("\n");
    let mut hunks = Vec::new();

    for (idx, block) in hunk_blocks.into_iter().enumerate() {
        if block.is_empty() {
            continue;
        }
        let header_line = &block[0];
        let parts: Vec<&str> = header_line.split_whitespace().collect();
        let (old_start, old_lines) = parts.get(1).map(|s| parse_range(s)).unwrap_or((0, 0));
        let (new_start, new_lines) = parts.get(2).map(|s| parse_range(s)).unwrap_or((0, 0));

        let hunk_body = block.join("\n");
        let raw_patch = format!("{file_header}\n{hunk_body}\n");

        hunks.push(DiffHunk {
            hunk_idx: idx,
            old_start,
            old_lines,
            new_start,
            new_lines,
            header: header_line.clone(),
            raw_patch,
            is_staged,
        });
    }

    hunks
}

pub fn get_file_hunks(repo_path: &str, file_rel_path: &str) -> Vec<DiffHunk> {
    let mut result = Vec::new();

    // 1. Unstaged hunks
    if let Ok(unstaged_diff) = run_git(repo_path, &["diff", "--unified=3", "--", file_rel_path]) {
        let unstaged_hunks = parse_hunks_from_diff(&unstaged_diff, false);
        result.extend(unstaged_hunks);
    }

    // 2. Staged hunks
    if let Ok(staged_diff) = run_git(
        repo_path,
        &["diff", "--cached", "--unified=3", "--", file_rel_path],
    ) {
        let mut staged_hunks = parse_hunks_from_diff(&staged_diff, true);
        for h in &mut staged_hunks {
            h.hunk_idx += result.len();
        }
        result.extend(staged_hunks);
    }

    result
}

pub fn find_hunk_at_line(hunks: &[DiffHunk], line_1_indexed: usize) -> Option<&DiffHunk> {
    if hunks.is_empty() {
        return None;
    }

    for hunk in hunks {
        let start = hunk.new_start;
        let end = hunk.new_start + hunk.new_lines.max(1).saturating_sub(1);
        if line_1_indexed >= start && line_1_indexed <= end {
            return Some(hunk);
        }
    }

    hunks.iter().min_by_key(|hunk| {
        let start = hunk.new_start;
        let end = hunk.new_start + hunk.new_lines.max(1).saturating_sub(1);
        if line_1_indexed < start {
            start - line_1_indexed
        } else {
            line_1_indexed.saturating_sub(end)
        }
    })
}

pub fn stage_hunk(repo_path: &str, hunk: &DiffHunk) -> Result<(), GitError> {
    if hunk.is_staged {
        return Ok(());
    }
    run_git_with_stdin(
        repo_path,
        &["apply", "--cached", "--whitespace=nowarn", "-"],
        &hunk.raw_patch,
    )?;
    Ok(())
}

pub fn unstage_hunk(repo_path: &str, hunk: &DiffHunk) -> Result<(), GitError> {
    if !hunk.is_staged {
        return Ok(());
    }
    run_git_with_stdin(
        repo_path,
        &["apply", "--cached", "--reverse", "--whitespace=nowarn", "-"],
        &hunk.raw_patch,
    )?;
    Ok(())
}

pub fn discard_hunk(repo_path: &str, hunk: &DiffHunk) -> Result<(), GitError> {
    if hunk.is_staged {
        return Err(GitError::Command(
            "Cannot discard a staged hunk directly. Unstage it first.".to_string(),
        ));
    }
    run_git_with_stdin(
        repo_path,
        &["apply", "--reverse", "--whitespace=nowarn", "-"],
        &hunk.raw_patch,
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_range() {
        assert_eq!(parse_range("-10,5"), (10, 5));
        assert_eq!(parse_range("+12"), (12, 1));
        assert_eq!(parse_range("-0,0"), (0, 0));
    }

    #[test]
    fn test_parse_hunks_from_diff() {
        let diff = r#"diff --git a/file.txt b/file.txt
index 123..456 100644
--- a/file.txt
+++ b/file.txt
@@ -1,5 +1,5 @@
 line 1
-line 2
+MODIFIED 2
 line 3
@@ -20,3 +20,4 @@
 line 20
+line 20.5
 line 21
"#;
        let hunks = parse_hunks_from_diff(diff, false);
        assert_eq!(hunks.len(), 2);
        assert_eq!(hunks[0].new_start, 1);
        assert_eq!(hunks[0].new_lines, 5);
        assert_eq!(hunks[1].new_start, 20);
        assert_eq!(hunks[1].new_lines, 4);

        assert_eq!(find_hunk_at_line(&hunks, 2).unwrap().hunk_idx, 0);
        assert_eq!(find_hunk_at_line(&hunks, 21).unwrap().hunk_idx, 1);
        // Nearest match for line 10 should be hunk 0 or hunk 1
        assert!(find_hunk_at_line(&hunks, 10).is_some());
    }
}
