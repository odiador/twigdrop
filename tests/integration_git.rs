//! Integration tests against a live temporary git repository.
//!
//! Covers the `plans/technical-debt.md` checkbox: `tempfile` +
//! `git2::Repository::init` fixture for destructive-path git logic.

use git2::{Repository, Signature};
use tempfile::TempDir;
use tokio::sync::mpsc;
use twigdrop::models::Branch;

/// Init a repo with one commit on the default branch.
fn init_repo() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("init");
    repo.config()
        .expect("config")
        .set_str("user.name", "Twigdrop Test")
        .expect("user.name");
    repo.config()
        .expect("config")
        .set_str("user.email", "test@twigdrop.dev")
        .expect("user.email");
    std::fs::write(dir.path().join("README.md"), "# test\n").expect("write");
    let mut index = repo.index().expect("index");
    index
        .add_path(std::path::Path::new("README.md"))
        .expect("add");
    let tree_id = index.write_tree().expect("tree");
    let tree = repo.find_tree(tree_id).expect("find tree");
    let sig = Signature::now("Twigdrop Test", "test@twigdrop.dev").expect("sig");
    repo.commit(Some("HEAD"), &sig, &sig, "initial commit", &tree, &[])
        .expect("commit");
    dir
}

fn repo_path(dir: &TempDir) -> String {
    dir.path().to_string_lossy().to_string()
}

fn test_app(path: &str, branches: Vec<Branch>, current: &str) -> twigdrop::app::App {
    let (tx, _) = mpsc::channel(1);
    let (ai_tx, _) = mpsc::channel(1);
    let (conflict_tx, _) = mpsc::channel(1);
    twigdrop::app::App::new(path, branches, current.to_string(), tx, ai_tx, conflict_tx)
}

#[test]
fn build_branches_lists_current_branch() {
    let dir = init_repo();
    let path = repo_path(&dir);
    let branches = twigdrop::git::build_branches(&path);
    assert!(!branches.is_empty(), "fresh repo must list a branch");
    let repo = Repository::open(dir.path()).expect("open");
    let head = repo.head().expect("head");
    let current = head.shorthand().unwrap_or("main").to_string();
    assert!(
        branches.iter().any(|b| b.name == current),
        "must list current branch {current}"
    );
}

#[test]
fn build_branches_sees_new_branch() {
    let dir = init_repo();
    let path = repo_path(&dir);
    let repo = Repository::open(dir.path()).expect("open");
    let head = repo.head().expect("head").peel_to_commit().expect("commit");
    repo.branch("feature/x", &head, false).expect("branch");
    let branches = twigdrop::git::build_branches(&path);
    assert!(branches.iter().any(|b| b.name == "feature/x"));
}

#[test]
fn inspector_cache_populates_from_live_repo() {
    let dir = init_repo();
    let path = repo_path(&dir);
    let branches = twigdrop::git::build_branches(&path);
    let current = branches
        .first()
        .map(|b| b.name.clone())
        .unwrap_or_else(|| "main".to_string());
    let mut app = test_app(&path, branches, &current);
    app.refresh_filtered_branches();
    // Select the first real branch (index 0 can be a `*` working-tree row).
    let pos = app
        .get_filtered_branches()
        .iter()
        .position(|b| !b.name.starts_with('*'))
        .expect("a real branch");
    app.ui.selected_branch_idx = pos;
    let selected = app.get_filtered_branches()[pos].name.clone();
    app.refresh_inspector_commits(&path);
    assert_eq!(app.repo.inspector_branch, selected);
    assert!(
        !app.repo.inspector_commits.is_empty(),
        "initial commit must appear in inspector cache"
    );
    assert!(app.repo.inspector_commits[0].contains("initial commit"));
}

#[test]
fn rename_branch_roundtrip() {
    let dir = init_repo();
    let path = repo_path(&dir);
    {
        let repo = Repository::open(dir.path()).expect("open");
        let head = repo.head().expect("head").peel_to_commit().expect("commit");
        repo.branch("feature/old-name", &head, false)
            .expect("branch");
    }

    twigdrop::git::commands::run_git(
        &path,
        &["branch", "-m", "feature/old-name", "feature/new-name"],
    )
    .expect("rename");
    let branches = twigdrop::git::build_branches(&path);
    assert!(branches.iter().any(|b| b.name == "feature/new-name"));
    assert!(!branches.iter().any(|b| b.name == "feature/old-name"));
}

#[test]
fn merge_branch_into_current() {
    let dir = init_repo();
    let path = repo_path(&dir);
    {
        let repo = Repository::open(dir.path()).expect("open");
        let head = repo.head().expect("head").peel_to_commit().expect("commit");
        repo.branch("feature/add", &head, false).expect("branch");
    }
    // No-op fast-forward merge of an even branch must succeed.
    twigdrop::git::commands::run_git(&path, &["merge", "--no-edit", "feature/add"]).expect("merge");
    let branches = twigdrop::git::build_branches(&path);
    assert!(branches.iter().any(|b| b.name == "feature/add"));
}

#[test]
fn delete_then_undo_restores_branch() {
    let dir = init_repo();
    let path = repo_path(&dir);
    {
        let repo = Repository::open(dir.path()).expect("open");
        let head = repo.head().expect("head").peel_to_commit().expect("commit");
        repo.branch("feature/doomed", &head, false).expect("branch");
    }
    let branches = twigdrop::git::build_branches(&path);
    let current = branches
        .first()
        .map(|b| b.name.clone())
        .unwrap_or_else(|| "main".to_string());
    let mut app = test_app(&path, branches, &current);
    app.refresh_filtered_branches();

    app.delete_branches_by_name(&path, &["feature/doomed".to_string()]);
    assert!(
        app.ui.undoable_toast().is_some(),
        "delete must arm the undo toast"
    );
    let gone = twigdrop::git::build_branches(&path);
    assert!(!gone.iter().any(|b| b.name == "feature/doomed"));

    app.undo_delete(&path);
    let back = twigdrop::git::build_branches(&path);
    assert!(back.iter().any(|b| b.name == "feature/doomed"));
    assert!(app.ui.undoable_toast().is_none(), "undo consumes the toast");
}

#[test]
fn snap_confirm_flow_deletes_without_animations() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use twigdrop::ui::animations::snap::SnapAnimation;

    let dir = init_repo();
    let path = repo_path(&dir);
    {
        let repo = Repository::open(dir.path()).expect("open");
        let head = repo.head().expect("head").peel_to_commit().expect("commit");
        repo.branch("feature/snap", &head, false).expect("branch");
    }
    let branches = twigdrop::git::build_branches(&path);
    let current = branches
        .first()
        .map(|b| b.name.clone())
        .unwrap_or_else(|| "main".to_string());
    let mut app = test_app(&path, branches, &current);
    // Animations off: the confirmed delete must still complete on draw.
    app.config.enable_animations = false;
    app.refresh_filtered_branches();
    app.ui.snap_animation = Some(SnapAnimation::new(vec!["feature/snap".to_string()]));

    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).expect("terminal");
    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &path))
        .expect("draw");

    assert!(
        app.ui.snap_animation.is_none(),
        "one-shot path clears the animation"
    );
    let gone = twigdrop::git::build_branches(&path);
    assert!(!gone.iter().any(|b| b.name == "feature/snap"));
    assert!(app.ui.undoable_toast().is_some());
}

#[test]
fn prunable_filter_never_touches_live_branches() {
    use twigdrop::actions::commands::get_prunable_branches;
    let dir = init_repo();
    let path = repo_path(&dir);
    let repo = Repository::open(dir.path()).expect("open");
    let head = repo.head().expect("head").peel_to_commit().expect("commit");
    repo.branch("feature/keep", &head, false).expect("branch");
    let branches = twigdrop::git::build_branches(&path);
    let current = branches.first().map(|b| b.name.clone()).unwrap_or_default();
    let prunable = get_prunable_branches(&branches, &current, &[]);
    assert!(
        !prunable.contains(&current),
        "current branch is never prunable"
    );
    assert!(
        !prunable
            .iter()
            .any(|name| twigdrop::actions::commands::is_protected_branch(name)),
        "protected branches are never prunable"
    );
}

#[test]
fn resolve_remote_prefers_upstream_then_first_remote() {
    use std::process::Command;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().to_str().unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .current_dir(p)
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-q"]);
    assert_eq!(twigdrop::actions::commands::resolve_remote(p, "main"), None);
    git(&["remote", "add", "upstream", "https://example.com/a.git"]);
    assert_eq!(
        twigdrop::actions::commands::resolve_remote(p, "main").as_deref(),
        Some("upstream")
    );
    git(&["remote", "add", "zzz", "https://example.com/b.git"]);
    git(&["config", "branch.main.remote", "zzz"]);
    assert_eq!(
        twigdrop::actions::commands::resolve_remote(p, "main").as_deref(),
        Some("zzz")
    );
}

#[test]
fn get_branch_commits_respects_limit() {
    use std::process::Command;
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().to_str().unwrap();
    let git = |args: &[&str]| {
        assert!(
            Command::new("git")
                .current_dir(p)
                .args(args)
                .status()
                .unwrap()
                .success()
        );
    };
    git(&["init", "-q", "-b", "main"]);
    for i in 0..6 {
        git(&[
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@t",
            "commit",
            "-q",
            "--allow-empty",
            "-m",
            &format!("c{i}"),
        ]);
    }
    let commits = twigdrop::git::get_branch_commits(p, "main", 4);
    assert_eq!(commits.len(), 4);
    assert_eq!(commits[0].message, "c5");
}
