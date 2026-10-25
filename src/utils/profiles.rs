//! Folder structure profiles, architecture templates, and drift detection.
//!
//! Allows defining and saving expected directory hierarchies per project (e.g. Clean Architecture,
//! Rust CLI, Next.js, Python), auditing the workspace against the active profile to spot
//! rogue/clutter folders and missing directories, and scaffolding missing folders with one keystroke.
//! Works independently of Git.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderRule {
    pub path: String,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub purpose: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FolderProfile {
    pub name: String,
    pub description: String,
    pub expected_dirs: Vec<FolderRule>,
    #[serde(default)]
    pub rogue_patterns: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderStatus {
    Canonical,
    Missing,
    Rogue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderAuditItem {
    pub path: String,
    pub status: FolderStatus,
    pub purpose: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileAuditReport {
    pub profile_name: String,
    pub items: Vec<FolderAuditItem>,
    pub canonical_count: usize,
    pub missing_count: usize,
    pub rogue_count: usize,
}

impl FolderProfile {
    #[must_use]
    pub fn builtin_presets() -> Vec<Self> {
        vec![
            Self {
                name: "Rust Project".to_string(),
                description: "Standard idiomatic Rust workspace".to_string(),
                expected_dirs: vec![
                    FolderRule {
                        path: "src".to_string(),
                        optional: false,
                        purpose: Some("Application and library source code".to_string()),
                    },
                    FolderRule {
                        path: "tests".to_string(),
                        optional: false,
                        purpose: Some("Integration tests".to_string()),
                    },
                    FolderRule {
                        path: "docs".to_string(),
                        optional: true,
                        purpose: Some("Documentation and specifications".to_string()),
                    },
                    FolderRule {
                        path: "benches".to_string(),
                        optional: true,
                        purpose: Some("Criterion benchmarks".to_string()),
                    },
                    FolderRule {
                        path: "examples".to_string(),
                        optional: true,
                        purpose: Some("Usage examples and demos".to_string()),
                    },
                ],
                rogue_patterns: vec![
                    "temp".to_string(),
                    "scratch".to_string(),
                    "tmp".to_string(),
                    "dist".to_string(),
                ],
            },
            Self {
                name: "Clean Architecture".to_string(),
                description: "Domain-Driven Design / Clean Architecture layers".to_string(),
                expected_dirs: vec![
                    FolderRule {
                        path: "src/domain".to_string(),
                        optional: false,
                        purpose: Some("Entities, value objects, domain logic".to_string()),
                    },
                    FolderRule {
                        path: "src/application".to_string(),
                        optional: false,
                        purpose: Some("Use cases, services, commands".to_string()),
                    },
                    FolderRule {
                        path: "src/infrastructure".to_string(),
                        optional: false,
                        purpose: Some("Database, external APIs, persistence".to_string()),
                    },
                    FolderRule {
                        path: "src/presentation".to_string(),
                        optional: false,
                        purpose: Some("Controllers, UI, CLI handlers".to_string()),
                    },
                    FolderRule {
                        path: "tests".to_string(),
                        optional: false,
                        purpose: Some("Test suites".to_string()),
                    },
                ],
                rogue_patterns: vec!["scratch".to_string(), "temp".to_string()],
            },
            Self {
                name: "Web / Fullstack".to_string(),
                description: "Modern component-based web frontend/fullstack".to_string(),
                expected_dirs: vec![
                    FolderRule {
                        path: "src/components".to_string(),
                        optional: false,
                        purpose: Some("Reusable UI components".to_string()),
                    },
                    FolderRule {
                        path: "src/hooks".to_string(),
                        optional: true,
                        purpose: Some("State & utility hooks".to_string()),
                    },
                    FolderRule {
                        path: "src/lib".to_string(),
                        optional: false,
                        purpose: Some("Core helpers and clients".to_string()),
                    },
                    FolderRule {
                        path: "public".to_string(),
                        optional: false,
                        purpose: Some("Static assets".to_string()),
                    },
                ],
                rogue_patterns: vec!["build".to_string(), "dist".to_string()],
            },
            Self {
                name: "Python Package".to_string(),
                description: "Standard Python application / library".to_string(),
                expected_dirs: vec![
                    FolderRule {
                        path: "src".to_string(),
                        optional: false,
                        purpose: Some("Package source code".to_string()),
                    },
                    FolderRule {
                        path: "tests".to_string(),
                        optional: false,
                        purpose: Some("pytest unit and integration tests".to_string()),
                    },
                    FolderRule {
                        path: "docs".to_string(),
                        optional: true,
                        purpose: Some("Documentation".to_string()),
                    },
                ],
                rogue_patterns: vec!["__pycache__".to_string(), ".pytest_cache".to_string()],
            },
        ]
    }

    /// Automatically suggests a matching preset based on workspace marker files.
    #[must_use]
    pub fn detect_preset(root: &str) -> Self {
        let p = Path::new(root);
        let presets = Self::builtin_presets();
        if p.join("Cargo.toml").exists() {
            presets[0].clone()
        } else if p.join("package.json").exists() {
            presets[2].clone()
        } else if p.join("pyproject.toml").exists() || p.join("requirements.txt").exists() {
            presets[3].clone()
        } else {
            presets[0].clone()
        }
    }

    /// Captures the existing directory structure as a reusable custom profile.
    #[must_use]
    pub fn from_workspace(root: &str, name: &str) -> Self {
        let mut dirs = Vec::new();
        scan_dirs_recursive(Path::new(root), "", 0, 3, &mut dirs);
        let expected_dirs = dirs
            .into_iter()
            .map(|d| FolderRule {
                path: d,
                optional: false,
                purpose: Some("Existing project directory".to_string()),
            })
            .collect();

        Self {
            name: name.to_string(),
            description: format!("Captured profile from {name}"),
            expected_dirs,
            rogue_patterns: vec!["temp".to_string(), "scratch".to_string(), "tmp".to_string()],
        }
    }
}

fn scan_dirs_recursive(
    base: &Path,
    current: &str,
    depth: usize,
    max_depth: usize,
    acc: &mut Vec<String>,
) {
    if depth > max_depth {
        return;
    }
    let full = if current.is_empty() {
        base.to_path_buf()
    } else {
        base.join(current)
    };

    if let Ok(entries) = fs::read_dir(&full) {
        let mut subdirs = Vec::new();
        for entry in entries.flatten() {
            if let Ok(ft) = entry.file_type()
                && ft.is_dir()
            {
                let name = entry.file_name().to_string_lossy().to_string();
                if name == ".git" || name == "target" || name == "node_modules" || name == ".venv" {
                    continue;
                }
                let rel = if current.is_empty() {
                    name
                } else {
                    format!("{current}/{name}")
                };
                acc.push(rel.clone());
                subdirs.push(rel);
            }
        }
        for sub in subdirs {
            scan_dirs_recursive(base, &sub, depth + 1, max_depth, acc);
        }
    }
}

/// Audits a project directory against the given profile.
#[must_use]
pub fn audit_workspace(root: &str, profile: &FolderProfile) -> ProfileAuditReport {
    let base = Path::new(root);
    let mut actual_dirs = Vec::new();
    scan_dirs_recursive(base, "", 0, 3, &mut actual_dirs);
    let actual_set: HashSet<String> = actual_dirs.into_iter().collect();

    let mut items = Vec::new();
    let mut canonical_count = 0;
    let mut missing_count = 0;
    let mut rogue_count = 0;

    let mut expected_matched = HashSet::new();

    // 1. Check expected dirs
    for rule in &profile.expected_dirs {
        let norm_path = rule.path.trim_matches('/').replace('\\', "/");
        if actual_set.contains(&norm_path) {
            canonical_count += 1;
            expected_matched.insert(norm_path.clone());
            items.push(FolderAuditItem {
                path: norm_path,
                status: FolderStatus::Canonical,
                purpose: rule.purpose.clone(),
            });
        } else if !rule.optional {
            missing_count += 1;
            items.push(FolderAuditItem {
                path: norm_path,
                status: FolderStatus::Missing,
                purpose: rule.purpose.clone(),
            });
        }
    }

    // 2. Check for rogue / unlisted directories
    for actual in &actual_set {
        if !expected_matched.contains(actual) {
            let is_rogue = profile
                .rogue_patterns
                .iter()
                .any(|pat| actual.contains(pat));
            rogue_count += 1;
            items.push(FolderAuditItem {
                path: actual.clone(),
                status: FolderStatus::Rogue,
                purpose: if is_rogue {
                    Some("Matches rogue / clutter pattern".to_string())
                } else {
                    Some("Unregistered directory (architecture drift)".to_string())
                },
            });
        }
    }

    // Sort items: Missing first (need attention), then Rogue, then Canonical
    items.sort_by_key(|i| match i.status {
        FolderStatus::Missing => 0,
        FolderStatus::Rogue => 1,
        FolderStatus::Canonical => 2,
    });

    ProfileAuditReport {
        profile_name: profile.name.clone(),
        items,
        canonical_count,
        missing_count,
        rogue_count,
    }
}

/// Creates any missing non-optional directories defined by the audit report.
pub fn scaffold_missing(root: &str, report: &ProfileAuditReport) -> std::io::Result<Vec<String>> {
    let base = Path::new(root);
    let mut created = Vec::new();

    for item in &report.items {
        if item.status == FolderStatus::Missing {
            let target = base.join(&item.path);
            fs::create_dir_all(&target)?;
            created.push(item.path.clone());
        }
    }

    Ok(created)
}

/// Saves a custom profile to `.twigdrop/profiles/<name>.json`.
pub fn save_profile(root: &str, profile: &FolderProfile) -> std::io::Result<PathBuf> {
    let dir = Path::new(root).join(".twigdrop").join("profiles");
    fs::create_dir_all(&dir)?;

    let file_stem = profile.name.to_lowercase().replace([' ', '/', '\\'], "-");
    let file_path = dir.join(format!("{file_stem}.json"));
    let json = serde_json::to_string_pretty(profile)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    fs::write(&file_path, json)?;
    Ok(file_path)
}

/// Loads all profiles available for the given workspace (built-in + `.twigdrop/profiles/*.json`).
#[must_use]
pub fn load_profiles(root: &str) -> Vec<FolderProfile> {
    let mut profiles = FolderProfile::builtin_presets();
    let dir = Path::new(root).join(".twigdrop").join("profiles");

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|ext| ext == "json")
                && let Ok(content) = fs::read_to_string(&p)
                && let Ok(prof) = serde_json::from_str::<FolderProfile>(&content)
            {
                profiles.push(prof);
            }
        }
    }

    profiles
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_audit_workspace_finds_canonical_and_missing() {
        let dir = tempdir().unwrap();
        let p = dir.path().to_str().unwrap();

        // Create only 'src'
        fs::create_dir_all(dir.path().join("src")).unwrap();
        // Create an unexpected folder
        fs::create_dir_all(dir.path().join("scratch_junk")).unwrap();

        let profile = FolderProfile::builtin_presets()[0].clone(); // Rust Project (expects src, tests)
        let report = audit_workspace(p, &profile);

        assert_eq!(report.canonical_count, 1);
        assert!(report.missing_count >= 1); // tests is missing
        assert_eq!(report.rogue_count, 1); // scratch_junk is rogue

        // Scaffold missing
        let created = scaffold_missing(p, &report).unwrap();
        assert!(created.contains(&"tests".to_string()));
        assert!(dir.path().join("tests").exists());

        // Audit again: tests is now canonical!
        let report2 = audit_workspace(p, &profile);
        assert_eq!(report2.canonical_count, 2);
    }
}
