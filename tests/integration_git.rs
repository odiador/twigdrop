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

#[test]
fn mode_switch_to_files_populates_file_tree() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use twigdrop::state::{AppMode, FilePanel, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);
    let mut app = test_app(&p, vec![], "main");

    // Start in Branches mode with empty file_tree
    app.set_primary_mode(PrimaryMode::Branches);
    app.repo.file_tree.clear();
    assert_eq!(app.ui.primary_mode, PrimaryMode::Branches);
    assert!(app.repo.file_tree.is_empty());

    // Switch mode to Files (like via switcher or Alt+D)
    app.set_primary_mode(PrimaryMode::Files);
    app.load_file_tree(&p);

    assert_eq!(app.ui.primary_mode, PrimaryMode::Files);
    assert!(!app.repo.file_tree.is_empty());
    assert!(
        app.repo
            .file_tree
            .iter()
            .any(|e| e.path.to_string_lossy().contains("README.md"))
    );

    // Open code preview
    let preview = app.create_preview_state(&p, "README.md").expect("preview");
    app.ui.push_modal(AppMode::CodePreview(preview));
    assert!(matches!(app.ui.current_mode(), AppMode::CodePreview(_)));

    // Verify drawing renders both the directory block and the preview title
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();

    let buffer = terminal.backend().buffer();
    let content: String = (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(content.contains("Files"));
    assert!(content.contains("Preview: README.md"));

    // Switch away from Files mode to Branches: CodePreview modal should be cleaned up
    app.set_primary_mode(PrimaryMode::Branches);
    assert_eq!(app.ui.primary_mode, PrimaryMode::Branches);
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);
    assert_eq!(app.ui.active_panel, FilePanel::Directory);
}

#[test]
fn code_preview_tab_toggles_focus_and_allows_directory_navigation() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::state::{AppMode, FilePanel, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);
    // Add another file so we have 2 files to navigate
    std::fs::write(dir.path().join("LICENSE"), "MIT").expect("write license");
    let mut app = test_app(&p, vec![], "main");

    app.set_primary_mode(PrimaryMode::Files);
    app.load_file_tree(&p);
    assert_eq!(app.repo.file_tree.len(), 2);

    // Open preview on first file
    let rel_path = app.repo.file_tree[0].path.to_string_lossy().to_string();
    let preview = app.create_preview_state(&p, &rel_path).expect("preview");
    app.ui.push_modal(AppMode::CodePreview(preview));
    assert_eq!(app.ui.active_panel, FilePanel::Directory);

    // When focus is in Directory, down arrow navigates to second file
    let down_key = KeyEvent::new(KeyCode::Down, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, down_key, &p);
    assert_eq!(app.ui.selected_file_idx, 1);

    // Tab key toggles focus to Preview panel
    let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab_key, &p);
    assert_eq!(app.ui.active_panel, FilePanel::Preview);

    // Tab key again toggles back to Directory panel
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab_key, &p);
    assert_eq!(app.ui.active_panel, FilePanel::Directory);

    // Esc closes preview
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc_key, &p);
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);
}
