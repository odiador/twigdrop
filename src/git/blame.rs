use crate::git::commands::run_git;
use crate::state::ui::BlameEntry;
use std::collections::HashMap;

/// Retrieves blame metadata for each 0-based line index in the specified file.
pub fn get_file_blame(path: &str, file: &str) -> HashMap<usize, BlameEntry> {
    let args = ["blame", "--date=short", file];
    let output = match run_git(path, &args) {
        Ok(out) => out,
        Err(_) => return HashMap::new(),
    };

    let mut map = HashMap::new();
    for line in output.lines() {
        if let Some((idx, entry)) = parse_blame_line(line) {
            map.insert(idx, entry);
        }
    }
    map
}

/// Parses a single line from `git blame --date=short` format:
/// `3493f2e7 (Juan Manuel Amador Roa 2026-05-17 1) use anyhow::Result;`
pub fn parse_blame_line(line: &str) -> Option<(usize, BlameEntry)> {
    let paren_open = line.find('(')?;
    let paren_close = line[paren_open..].find(')')? + paren_open;

    let commit = line[..paren_open].trim().to_string();
    let inside = line[paren_open + 1..paren_close].trim();

    // The inside has: `<author...> <date> <lineno>`
    let parts: Vec<&str> = inside.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let lineno_str = parts.last()?;
    let lineno: usize = lineno_str.parse().ok()?;
    if lineno == 0 {
        return None;
    }

    let date = parts[parts.len() - 2].to_string();
    let author = parts[..parts.len() - 2].join(" ");

    Some((
        lineno - 1,
        BlameEntry {
            commit,
            author,
            date,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_blame_line_normal() {
        let line = "3493f2e7 (Juan Manuel Amador Roa 2026-05-17 1) use anyhow::Result;";
        let (idx, entry) = parse_blame_line(line).unwrap();
        assert_eq!(idx, 0);
        assert_eq!(entry.commit, "3493f2e7");
        assert_eq!(entry.author, "Juan Manuel Amador Roa");
        assert_eq!(entry.date, "2026-05-17");
    }

    #[test]
    fn test_parse_blame_line_single_name() {
        let line = "c93df5d5 (odiador 2026-05-01 42)     crossterm::{";
        let (idx, entry) = parse_blame_line(line).unwrap();
        assert_eq!(idx, 41);
        assert_eq!(entry.commit, "c93df5d5");
        assert_eq!(entry.author, "odiador");
        assert_eq!(entry.date, "2026-05-01");
    }

    #[test]
    fn test_parse_blame_line_uncommitted() {
        let line = "00000000 (Not Committed Yet 2026-10-25 10) let x = 1;";
        let (idx, entry) = parse_blame_line(line).unwrap();
        assert_eq!(idx, 9);
        assert_eq!(entry.commit, "00000000");
        assert_eq!(entry.author, "Not Committed Yet");
    }

    #[test]
    fn test_parse_blame_line_invalid() {
        assert_eq!(parse_blame_line("not a blame line"), None);
        assert_eq!(parse_blame_line(""), None);
    }
}
