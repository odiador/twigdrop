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

    twigdrop::actions::commands::rename_branch(&path, "feature/old-name", "feature/new-name")
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
    twigdrop::actions::commands::merge_branch(&path, "feature/add").expect("merge");
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

#[test]
fn files_changed_only_filter_and_space_staging() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::git::files::FileStatus;
    use twigdrop::state::PrimaryMode;

    let dir = init_repo();
    let p = repo_path(&dir);

    // Create an uncommitted file and modify README.md
    std::fs::write(dir.path().join("untracked.txt"), "hello").expect("write file");
    std::fs::write(dir.path().join("README.md"), "modified content").expect("modify readme");

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::Files);
    app.load_file_tree(&p);

    assert!(!app.ui.files_changed_only);
    assert_eq!(app.ui.filtered_file_indices.len(), app.repo.file_tree.len());

    // Toggle changed only with 'c'
    let c_key = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, c_key, &p);
    assert!(app.ui.files_changed_only);

    // Every filtered entry must now be non-normal and non-ignored
    for &idx in &app.ui.filtered_file_indices {
        let entry = &app.repo.file_tree[idx];
        assert_ne!(entry.status, FileStatus::Normal);
        assert_ne!(entry.status, FileStatus::Ignored);
    }

    // Space key triggers instant staging with toast notification (no modal)
    let space_key = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, space_key, &p);
    assert!(app.ui.toast.is_some());
    assert_eq!(*app.ui.current_mode(), twigdrop::state::AppMode::Normal);

    // Pressing Space again unstages the file
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, space_key, &p);
    assert!(app.ui.toast.is_some());

    // Esc resets the changed-only filter
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc_key, &p);
    assert!(!app.ui.files_changed_only);
}

#[test]
fn commits_panel_tab_navigation_and_file_diff_explorer() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::state::{CommitPanel, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);

    // Create a commit that changes two files
    std::fs::write(dir.path().join("file_a.txt"), "line 1\nline 2\n").expect("write");
    std::fs::write(dir.path().join("file_b.txt"), "line b\n").expect("write");
    let repo = git2::Repository::open(&p).unwrap();
    let mut index = repo.index().unwrap();
    index.add_path(std::path::Path::new("file_a.txt")).unwrap();
    index.add_path(std::path::Path::new("file_b.txt")).unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let parent = repo.head().unwrap().peel_to_commit().unwrap();
    let sig = git2::Signature::now("Tester", "test@example.com").unwrap();
    repo.commit(
        Some("HEAD"),
        &sig,
        &sig,
        "feat: add files a and b",
        &tree,
        &[&parent],
    )
    .unwrap();

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::Commits);
    app.repo.commit_tree = twigdrop::git::commands::get_commit_tree(&p);
    app.ui.selected_commit_idx = 0;
    app.fetch_current_commit_details(&p);

    assert_eq!(app.ui.commit_panel, CommitPanel::Tree);
    assert!(!app.repo.commit_files.is_empty());
    assert!(app.repo.commit_file_diff.is_some());

    // Tab key switches to CommitPanel::Files
    let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab_key, &p);
    assert_eq!(app.ui.commit_panel, CommitPanel::Files);

    // Navigate to next file in commit with 'j'
    let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, j_key, &p);
    assert_eq!(app.ui.selected_commit_file_idx, 1);
    assert!(app.repo.commit_file_diff.is_some());

    // Render frame to ensure draw doesn't panic and displays formatted diff
    let backend = TestBackend::new(140, 40);
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
    assert!(content.contains("Diff:"));

    // Tab switches back to CommitPanel::Tree
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab_key, &p);
    assert_eq!(app.ui.commit_panel, CommitPanel::Tree);
}

#[test]
fn branches_active_only_tab_toggle_filters_stale_branches() {
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::models::{Branch, BranchKind, BranchStatus, MergeStatus};
    use twigdrop::state::PrimaryMode;

    let dir = init_repo();
    let p = repo_path(&dir);

    let active_branch = Branch {
        name: "feature-active".to_string(),
        kind: BranchKind::Real,
        status: vec![BranchStatus::Ahead, BranchStatus::HasUniqueCommits],
        merge_status: MergeStatus::Clean,
        age: "1h ago".to_string(),
        author: "Dev".to_string(),
        commit_date: "2026-10-25".to_string(),
        ahead_count: 2,
        behind_count: 0,
    };

    let stale_branch = Branch {
        name: "old-merged".to_string(),
        kind: BranchKind::Real,
        status: vec![BranchStatus::Merged],
        merge_status: MergeStatus::Clean,
        age: "30d ago".to_string(),
        author: "Dev".to_string(),
        commit_date: "2026-09-01".to_string(),
        ahead_count: 0,
        behind_count: 0,
    };

    let main_branch = Branch {
        name: "main".to_string(),
        kind: BranchKind::Real,
        status: vec![BranchStatus::Local],
        merge_status: MergeStatus::Clean,
        age: "1d ago".to_string(),
        author: "Dev".to_string(),
        commit_date: "2026-10-24".to_string(),
        ahead_count: 0,
        behind_count: 0,
    };

    let mut app = test_app(&p, vec![main_branch, active_branch, stale_branch], "main");
    app.set_primary_mode(PrimaryMode::Branches);
    app.refresh_filtered_branches();

    assert!(!app.ui.branches_active_only);
    assert_eq!(app.ui.filtered_indices.len(), 3);

    // Tab in Branches mode toggles Active Only filter
    let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab_key, &p);

    assert!(app.ui.branches_active_only);
    // stale_branch (Merged, 0 ahead, 0 behind, not main) must be excluded
    let filtered_names: Vec<String> = app
        .get_filtered_branches()
        .into_iter()
        .map(|b| b.name.clone())
        .collect();

    assert!(filtered_names.contains(&"main".to_string()));
    assert!(filtered_names.contains(&"feature-active".to_string()));
    assert!(!filtered_names.contains(&"old-merged".to_string()));

    // Tab again restores all branches
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab_key, &p);
    assert!(!app.ui.branches_active_only);
    assert_eq!(app.ui.filtered_indices.len(), 3);
}

#[test]
fn pull_requests_mode_keyboard_navigation_and_tab_diff_toggle() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::models::{PullRequest, PullRequestAuthor};
    use twigdrop::state::ui::{PrFilter, PrTab, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);
    let mut app = test_app(&p, vec![], "main");

    // Press Shift+P to switch to PullRequests mode
    let shift_p_key = KeyEvent::new(KeyCode::Char('P'), KeyModifiers::SHIFT);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, shift_p_key, &p);
    assert_eq!(app.ui.primary_mode, PrimaryMode::PullRequests);

    // Mock two PRs
    app.repo.pull_requests = vec![
        PullRequest {
            number: 10,
            title: "feat: first feature".to_string(),
            author: PullRequestAuthor {
                login: "bob".to_string(),
            },
            head_ref_name: "feat/first".to_string(),
            base_ref_name: "main".to_string(),
            state: "OPEN".to_string(),
            updated_at: "2026-10-25".to_string(),
            url: "https://github.com/test/repo/pull/10".to_string(),
            body: "## Summary\nFirst PR description in **markdown**.".to_string(),
        },
        PullRequest {
            number: 11,
            title: "fix: second fix".to_string(),
            author: PullRequestAuthor {
                login: "charlie".to_string(),
            },
            head_ref_name: "fix/second".to_string(),
            base_ref_name: "main".to_string(),
            state: "MERGED".to_string(),
            updated_at: "2026-10-24".to_string(),
            url: "https://github.com/test/repo/pull/11".to_string(),
            body: "Bug fix details.".to_string(),
        },
    ];
    app.repo.pr_diff = Some("diff --git a/a.txt b/a.txt\n+added line\n-removed line\n".to_string());

    assert_eq!(app.ui.selected_pr_idx, 0);
    assert_eq!(app.ui.pr_tab, PrTab::Description);

    // Tab key toggles between Description and Diff tab
    let tab_key = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab_key, &p);
    assert_eq!(app.ui.pr_tab, PrTab::Diff);

    // 'j' navigates down to second PR
    let j_key = KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, j_key, &p);
    assert_eq!(app.ui.selected_pr_idx, 1);

    // Render frame to ensure draw doesn't panic
    let backend = TestBackend::new(140, 40);
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

    assert!(content.contains("Pull Requests"));
    assert!(content.contains("#10"));
    assert!(content.contains("first feature"));
    assert!(content.contains("#11"));

    // 'f' cycles filter from Open -> All
    let f_key = KeyEvent::new(KeyCode::Char('f'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, f_key, &p);
    assert_eq!(app.ui.pr_filter, PrFilter::All);
}

#[test]
fn dotfiles_toggle_and_vscode_breadcrumbs_preview() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::state::{AppMode, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);

    // Create a dotfile and a nested file
    std::fs::write(dir.path().join(".env"), "SECRET=123").expect("write .env");
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");
    std::fs::write(dir.path().join("src/main.rs"), "fn main() {}\n").expect("write main.rs");

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::Files);
    app.load_file_tree(&p);

    // By default, hidden files are not shown
    assert!(!app.ui.show_hidden_files);
    let paths_before: Vec<String> = app
        .ui
        .filtered_file_indices
        .iter()
        .map(|&i| app.repo.file_tree[i].path.to_string_lossy().to_string())
        .collect();
    assert!(!paths_before.iter().any(|s| s.contains(".env")));

    // Press '.' to toggle hidden dotfiles
    let dot_key = KeyEvent::new(KeyCode::Char('.'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, dot_key, &p);
    assert!(app.ui.show_hidden_files);

    let paths_after: Vec<String> = app
        .ui
        .filtered_file_indices
        .iter()
        .map(|&i| app.repo.file_tree[i].path.to_string_lossy().to_string())
        .collect();
    assert!(paths_after.iter().any(|s| s.contains(".env")));

    // Open code preview on src/main.rs
    let preview = app
        .create_preview_state(&p, "src/main.rs")
        .expect("preview");
    app.ui.push_modal(AppMode::CodePreview(preview));

    // Render frame to verify breadcrumbs and line numbers
    let backend = TestBackend::new(140, 40);
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

    assert!(content.contains("src"));
    assert!(content.contains("main.rs"));
    assert!(content.contains("1 │"));
    assert!(content.contains("[Ln 1/"));
}

#[test]
fn folder_profile_modal_audit_and_scaffold() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::state::{AppMode, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);

    // Create only src directory
    std::fs::create_dir_all(dir.path().join("src")).expect("mkdir src");

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::Files);
    app.load_file_tree(&p);

    // Press 'p' to open Folder Profile modal
    let p_key = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, p_key, &p);

    assert!(matches!(app.ui.current_mode(), AppMode::FolderProfile(_)));

    // Render frame to ensure modal renders without panic
    let backend = TestBackend::new(140, 40);
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

    assert!(content.contains("Folder Profiles & Architecture Audit"));
    assert!(content.contains("Missing:"));

    // Press 's' to scaffold missing directories
    let s_key = KeyEvent::new(KeyCode::Char('s'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, s_key, &p);

    // Verify 'tests' directory was scaffolded
    assert!(dir.path().join("tests").exists());

    // Press 'Esc' to exit modal
    let esc_key = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc_key, &p);
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);
}

#[test]
fn test_score_maxing_zoom_and_quick_jump() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::state::{AppMode, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);
    let readme = dir.path().join("README.md");
    std::fs::write(&readme, "# Sample Project\nLine 2\nLine 3\n").expect("write readme");

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::Files);
    app.load_file_tree(&p);

    // Test Ctrl+P activates quick jump search
    let ctrl_p = KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, ctrl_p, &p);
    assert!(app.ui.files_search_active);

    // Cancel search with Esc
    let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc, &p);
    assert!(!app.ui.files_search_active);

    // Open CodePreview
    let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, enter, &p);
    assert!(matches!(app.ui.current_mode(), AppMode::CodePreview(_)));

    // Press Z to maximize preview panel
    let z_key = KeyEvent::new(KeyCode::Char('Z'), KeyModifiers::SHIFT);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, z_key, &p);
    assert!(app.ui.maximized_panel);

    // Render frame to check scroll indicator and [Z: restore]
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

    assert!(content.contains("[Z: restore]"));
    assert!(content.contains("[All]"));

    // Press Esc to restore panel
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc, &p);
    assert!(!app.ui.maximized_panel);
}

#[test]
fn test_pr_ci_checks_and_review_modals() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::models::{PrCheck, PullRequest, PullRequestAuthor};
    use twigdrop::state::{AppMode, PrReviewKind, PrTab, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::PullRequests);

    let test_pr = PullRequest {
        number: 42,
        title: "Super Feature".to_string(),
        author: PullRequestAuthor {
            login: "octocat".to_string(),
        },
        head_ref_name: "feature/super".to_string(),
        base_ref_name: "main".to_string(),
        updated_at: "2026-10-25".to_string(),
        url: "https://github.com/test/repo/pull/42".to_string(),
        state: "OPEN".to_string(),
        body: "Detailed description of PR #42".to_string(),
    };
    app.repo.pull_requests = vec![test_pr];

    let check1 = PrCheck {
        name: "test-suite".to_string(),
        state: "SUCCESS".to_string(),
        bucket: "pass".to_string(),
        description: "All unit tests passed".to_string(),
        link: "https://github.com/test/repo/actions/runs/1".to_string(),
    };
    app.repo.pr_checks.insert(42, vec![check1]);

    // Render initial PR view (Description tab)
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

    assert!(content.contains("Super Feature"));
    assert!(content.contains("3: Checks"));
    assert!(content.contains("✓ 1")); // Passed CI status badge with count

    // Press '3' to switch directly to Checks tab
    let key_3 = KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_3, &p);
    assert_eq!(app.ui.pr_tab, PrTab::Checks);

    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();
    let content_checks: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content_checks.contains("test-suite"));
    assert!(content_checks.contains("All unit tests passed"));
    assert!(content_checks.contains("▸"));
    assert!(content_checks.contains("open log"));

    // Press Enter to open check log in browser
    let enter_key = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, enter_key, &p);
    assert!(app.ui.toast.is_some());

    // Press Tab to cycle to Comments
    let tab = KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab, &p);
    assert_eq!(app.ui.pr_tab, PrTab::Comments);

    // Press Tab again to cycle back to Description
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, tab, &p);
    assert_eq!(app.ui.pr_tab, PrTab::Description);

    // Press 'a' to open Approve modal
    let key_a = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_a, &p);
    assert!(
        matches!(app.ui.current_mode(), AppMode::PrReview(s) if s.kind == PrReviewKind::Approve && s.pr_number == 42)
    );

    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();
    let content_approve: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content_approve.contains("Approve Pull Request"));
    assert!(content_approve.contains("#42"));

    // Close modal with Esc
    let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc, &p);
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);

    // Press 'x' to open Request Changes modal
    let key_x = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_x, &p);
    assert!(
        matches!(app.ui.current_mode(), AppMode::PrReview(s) if s.kind == PrReviewKind::RequestChanges)
    );

    // Pressing Enter with empty body fails validation and modal stays open
    let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, enter, &p);
    assert!(matches!(app.ui.current_mode(), AppMode::PrReview(_)));

    // Close modal with Esc
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc, &p);
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);

    // Press 'm' to open Comment modal
    let key_m = KeyEvent::new(KeyCode::Char('m'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_m, &p);
    assert!(
        matches!(app.ui.current_mode(), AppMode::PrReview(s) if s.kind == PrReviewKind::Comment)
    );
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc, &p);
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);
}

#[test]
fn test_inline_blame_toggle_and_hunk_jump() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::state::{AppMode, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);

    // Create a multi-line file and commit it
    let file_path = dir.path().join("code.txt");
    let mut initial_lines = Vec::new();
    for i in 1..=30 {
        initial_lines.push(format!("Original line {i}"));
    }
    std::fs::write(&file_path, initial_lines.join("\n") + "\n").expect("write code.txt");

    std::process::Command::new("git")
        .current_dir(&p)
        .args(["add", "code.txt"])
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .current_dir(&p)
        .args(["commit", "-m", "add code.txt"])
        .output()
        .expect("git commit");

    // Now introduce changes at line 5 and line 20
    let mut modified_lines = initial_lines.clone();
    modified_lines[4] = "Modified line 5".to_string();
    modified_lines[19] = "Modified line 20".to_string();
    std::fs::write(&file_path, modified_lines.join("\n") + "\n").expect("modify code.txt");

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::Files);
    app.load_file_tree(&p);

    let preview = app.create_preview_state(&p, "code.txt").expect("preview");
    app.ui.push_modal(AppMode::CodePreview(preview));
    assert!(matches!(app.ui.current_mode(), AppMode::CodePreview(_)));

    // Verify git blame was populated
    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert!(!s.line_blames.is_empty(), "line_blames should be populated");
        assert!(!s.show_blame, "blame should be off by default");
        assert!(
            s.line_diffs.contains_key(&4),
            "diff at line 5 (0-indexed 4)"
        );
        assert!(
            s.line_diffs.contains_key(&19),
            "diff at line 20 (0-indexed 19)"
        );
    }

    // Render frame to check active line blame chip in breadcrumbs
    let backend = TestBackend::new(120, 30);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();

    let content_preview: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(content_preview.contains("👤"));
    assert!(content_preview.contains("Twigdrop"));

    // Press 'b' to toggle inline blame ON
    let key_b = KeyEvent::new(KeyCode::Char('b'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_b, &p);

    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert!(s.show_blame);
    }

    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();
    let content_blame: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content_blame.contains("[Blame: ON]"));

    // Press 'b' again to toggle inline blame OFF
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_b, &p);
    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert!(!s.show_blame);
    }

    // Test hunk navigation: 'n' jumps to next change (line 5 -> 0-indexed 4)
    let key_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_n, &p);
    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert_eq!(s.cursor_y, 4);
    }

    // 'n' again jumps to second change (line 20 -> 0-indexed 19)
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_n, &p);
    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert_eq!(s.cursor_y, 19);
    }

    // 'n' again wraps back to line 5 (0-indexed 4)
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_n, &p);
    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert_eq!(s.cursor_y, 4);
    }

    // 'N' jumps backwards to line 20 (0-indexed 19)
    let key_big_n = KeyEvent::new(KeyCode::Char('N'), KeyModifiers::SHIFT);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_big_n, &p);
    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert_eq!(s.cursor_y, 19);
    }

    // Press 'B' to inspect commit details from blame
    let key_b_shift = KeyEvent::new(KeyCode::Char('B'), KeyModifiers::SHIFT);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_b_shift, &p);
    assert!(
        matches!(app.ui.current_mode(), AppMode::Message(msg) if msg.contains("Commit:") && msg.contains("Author:"))
    );

    // Esc closes message modal and returns to CodePreview
    let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc, &p);
    assert!(matches!(app.ui.current_mode(), AppMode::CodePreview(_)));

    // Second Esc closes CodePreview and returns to normal
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc, &p);
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);
}

#[test]
fn test_pr_discussion_tab_and_markdown_rendering() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::collections::HashMap;
    use twigdrop::models::{PrComment, PullRequest, PullRequestAuthor};
    use twigdrop::state::{PrTab, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::PullRequests);

    let pr = PullRequest {
        number: 100,
        title: "Support OAuth2 login".to_string(),
        author: PullRequestAuthor {
            login: "alice".to_string(),
        },
        head_ref_name: "feature/oauth".to_string(),
        base_ref_name: "main".to_string(),
        url: "https://github.com/example/repo/pull/100".to_string(),
        state: "OPEN".to_string(),
        updated_at: "2026-10-25T19:00:00Z".to_string(),
        body: "Implements GitHub and Google SSO".to_string(),
    };
    app.repo.pull_requests = vec![pr];
    app.ui.selected_pr_idx = 0;

    let comment = PrComment {
        author: PullRequestAuthor {
            login: "reviewer_bob".to_string(),
        },
        body: "Great work! Please ensure **expired tokens** are handled correctly:\n```rust\nlet expired = true;\n```".to_string(),
        created_at: "2026-10-25T20:00:00Z".to_string(),
    };
    let mut comments_map = HashMap::new();
    comments_map.insert(100, vec![comment]);
    app.repo.pr_comments = comments_map;

    // Press '4' to jump directly to Discussion tab
    let key_4 = KeyEvent::new(KeyCode::Char('4'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_4, &p);
    assert_eq!(app.ui.pr_tab, PrTab::Comments);

    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();

    let content: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(content.contains("[ 4: Discussion (1) ]"));
    assert!(content.contains("@reviewer_bob"));
    assert!(content.contains("Great work!"));
    assert!(content.contains("expired tokens"));

    // Test scrolling in discussion
    let down_key = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, down_key, &p);
    assert_eq!(app.ui.pr_scroll_y, 5);

    // Press '1' to jump back to Description tab
    let key_1 = KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_1, &p);
    assert_eq!(app.ui.pr_tab, PrTab::Description);
    assert_eq!(app.ui.pr_scroll_y, 0);
}

#[test]
fn test_visual_conflict_resolver_interactive_and_command_palette() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::state::AppMode;

    let dir = init_repo();
    let p = repo_path(&dir);

    // 1. Create a base file and commit on main
    let main_file = dir.path().join("conflict.txt");
    std::fs::write(&main_file, "Line 1: base\nLine 2: base\n").expect("write base");
    std::process::Command::new("git")
        .current_dir(&p)
        .args(["add", "conflict.txt"])
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .current_dir(&p)
        .args(["commit", "-m", "add conflict.txt"])
        .output()
        .expect("git commit");

    // 2. Create feature branch and modify line 2
    std::process::Command::new("git")
        .current_dir(&p)
        .args(["checkout", "-b", "feature-branch"])
        .output()
        .expect("git checkout -b");
    std::fs::write(&main_file, "Line 1: base\nLine 2: feature modification\n")
        .expect("write feature");
    std::process::Command::new("git")
        .current_dir(&p)
        .args(["commit", "-am", "feature commit"])
        .output()
        .expect("git commit feature");

    // 3. Switch back to main and modify line 2 differently
    std::process::Command::new("git")
        .current_dir(&p)
        .args(["checkout", "main"])
        .output()
        .expect("git checkout main");
    std::fs::write(
        &main_file,
        "Line 1: base\nLine 2: main branch modification\n",
    )
    .expect("write main");
    std::process::Command::new("git")
        .current_dir(&p)
        .args(["commit", "-am", "main commit"])
        .output()
        .expect("git commit main");

    // 4. Merge feature-branch into main to trigger a real git conflict
    let merge_output = std::process::Command::new("git")
        .current_dir(&p)
        .args(["merge", "feature-branch"])
        .output()
        .expect("git merge");
    assert!(!merge_output.status.success(), "Merge should conflict");

    // 5. Initialize app and verify conflict detection
    let mut app = test_app(&p, vec![], "main");
    app.refresh_branches(&p);
    assert_eq!(app.repo.conflicting_files, vec!["conflict.txt"]);

    // Render frame to check status bar conflict badge
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();

    let content: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content.contains("CONFLICTS (1) [X: Resolve]"));

    // 6. Test Command Palette has "Resolve Conflicts" as top action
    let ctrl_k = KeyEvent::new(KeyCode::Char('k'), KeyModifiers::CONTROL);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, ctrl_k, &p);
    assert!(matches!(app.ui.current_mode(), AppMode::CommandPalette(_)));
    if let AppMode::CommandPalette(s) = app.ui.current_mode() {
        assert!(s.actions[0].0.contains("Resolve Conflicts"));
    }

    // Close command palette with Esc
    let esc = KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, esc, &p);
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);

    // 7. Open Visual Conflict Resolver with 'X'
    let key_x_shift = KeyEvent::new(KeyCode::Char('X'), KeyModifiers::SHIFT);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_x_shift, &p);
    assert!(matches!(
        app.ui.current_mode(),
        AppMode::ConflictResolver(_)
    ));

    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();
    let content_resolver: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(content_resolver.contains("Visual 3-Way Conflict Resolver"));
    assert!(content_resolver.contains("conflict.txt"));
    assert!(content_resolver.contains("[o: Accept Ours]"));
    assert!(content_resolver.contains("[t: Accept Theirs]"));
    assert!(content_resolver.contains("OURS / CURRENT"));
    assert!(content_resolver.contains("THEIRS / INCOMING"));

    // 8. Press 'o' to Accept Ours
    let key_o = KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_o, &p);

    // Verify resolved content on disk
    let file_content = std::fs::read_to_string(&main_file).expect("read resolved file");
    assert!(file_content.contains("Line 2: main branch modification"));
    assert!(!file_content.contains("<<<<<<<"));
    assert!(!file_content.contains(">>>>>>>"));

    // Check git index: file should be auto-staged and conflicts cleared
    let conflicting = twigdrop::git::status::get_conflicting_files(&p);
    assert!(
        conflicting.is_empty(),
        "All conflicts should be resolved in index"
    );

    // 9. Press 'c' to commit the merge
    let key_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_c, &p);

    // Modal should be popped and merge finalized
    assert_eq!(*app.ui.current_mode(), AppMode::Normal);
    assert!(!twigdrop::git::status::is_merging(&p));
}

#[test]
fn test_partial_hunk_staging_and_discarding_flow() {
    use ratatui::Terminal;
    use ratatui::backend::TestBackend;
    use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use twigdrop::state::{AppMode, FilePanel, PrimaryMode};

    let dir = init_repo();
    let p = repo_path(&dir);

    // 1. Create a 50-line file and commit it
    let file_path = dir.path().join("code.txt");
    let mut lines = Vec::new();
    for i in 1..=50 {
        lines.push(format!("original line {i}"));
    }
    std::fs::write(&file_path, lines.join("\n") + "\n").expect("write code.txt");

    std::process::Command::new("git")
        .current_dir(&p)
        .args(["add", "code.txt"])
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .current_dir(&p)
        .args(["commit", "-m", "add code.txt"])
        .output()
        .expect("git commit");

    // 2. Introduce 2 modifications far apart (line 5 and line 35)
    lines[4] = "MODIFIED LINE 5".to_string();
    lines[34] = "MODIFIED LINE 35".to_string();
    std::fs::write(&file_path, lines.join("\n") + "\n").expect("modify code.txt");

    let mut app = test_app(&p, vec![], "main");
    app.set_primary_mode(PrimaryMode::Files);
    app.load_file_tree(&p);

    // 3. Open CodePreview for code.txt
    let preview = app.create_preview_state(&p, "code.txt").expect("preview");
    app.ui.active_panel = FilePanel::Preview;
    app.ui.push_modal(AppMode::CodePreview(preview));
    assert!(matches!(app.ui.current_mode(), AppMode::CodePreview(_)));

    // 4. Cursor starts at line 0 (1st line). Jump to first change with 'n'
    let key_n = KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_n, &p);
    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert_eq!(s.cursor_y, 4, "Cursor jumped to line 5 (0-indexed 4)");
    }

    // 5. Render frame to check hunk active badge & shortcuts
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();

    let content: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content.contains("[HUNK: Space stage • d discard]"));

    // 6. Press Space to stage hunk 1 (line 5)
    let space = KeyEvent::new(KeyCode::Char(' '), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, space, &p);

    // Verify in git: hunk 1 is staged, hunk 2 is unstaged
    let cached_diff = std::process::Command::new("git")
        .current_dir(&p)
        .args(["diff", "--cached", "--", "code.txt"])
        .output()
        .expect("git diff --cached");
    let cached_str = String::from_utf8_lossy(&cached_diff.stdout);
    assert!(cached_str.contains("MODIFIED LINE 5"));
    assert!(!cached_str.contains("MODIFIED LINE 35"));

    let unstaged_diff = std::process::Command::new("git")
        .current_dir(&p)
        .args(["diff", "--", "code.txt"])
        .output()
        .expect("git diff");
    let unstaged_str = String::from_utf8_lossy(&unstaged_diff.stdout);
    assert!(!unstaged_str.contains("MODIFIED LINE 5"));
    assert!(unstaged_str.contains("MODIFIED LINE 35"));

    // 7. Press Space again to unstage hunk 1
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, space, &p);
    let cached_diff2 = std::process::Command::new("git")
        .current_dir(&p)
        .args(["diff", "--cached", "--", "code.txt"])
        .output()
        .expect("git diff --cached");
    let cached_str2 = String::from_utf8_lossy(&cached_diff2.stdout);
    assert!(cached_str2.is_empty(), "Staged diff should now be empty");

    // Stage hunk 1 again
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, space, &p);

    // 8. Jump to second hunk with 'n'
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_n, &p);
    if let AppMode::CodePreview(s) = app.ui.current_mode() {
        assert_eq!(s.cursor_y, 34, "Cursor jumped to line 35 (0-indexed 34)");
    }

    // 9. Press 'd' to discard hunk 2
    let key_d = KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_d, &p);

    // Confirm discard modal should be active
    assert!(matches!(
        app.ui.current_mode(),
        AppMode::ConfirmDiscardHunk(_)
    ));

    terminal
        .draw(|f| twigdrop::ui::draw(f, &mut app, &p))
        .unwrap();
    let content_discard: String = (0..terminal.backend().buffer().area.height)
        .map(|y| {
            (0..terminal.backend().buffer().area.width)
                .map(|x| terminal.backend().buffer()[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(content_discard.contains("Confirm Discard Hunk"));
    assert!(content_discard.contains("code.txt"));

    // 10. Press 'y' to confirm discard
    let key_y = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE);
    twigdrop::handlers::keyboard::handle_keyboard(&mut app, key_y, &p);

    // Modal popped back to CodePreview
    assert!(matches!(app.ui.current_mode(), AppMode::CodePreview(_)));

    // Verify file on disk: line 35 is restored, line 5 still modified
    let content_file = std::fs::read_to_string(&file_path).expect("read file");
    assert!(content_file.contains("original line 35"));
    assert!(!content_file.contains("MODIFIED LINE 35"));
    assert!(content_file.contains("MODIFIED LINE 5"));
}
