use crate::app::App;
use crate::models::{Branch, BranchStatus, MergeStatus};
use crate::state::ui::{AppMode, CommandAction, CommandPaletteState};
use crate::ui::draw;
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Buffer;
use tokio::sync::mpsc;

fn create_test_app() -> App {
    let (tx, _) = mpsc::channel(1);
    let (ai_tx, _) = mpsc::channel(1);
    let (conflict_tx, _) = mpsc::channel(1);
    let branch = Branch {
        name: "main".to_string(),
        status: vec![BranchStatus::Merged],
        merge_status: MergeStatus::Clean,
        age: "1d ago".to_string(),
        author: "Tester".to_string(),
        commit_date: "2026-10-01".to_string(),
        ahead_count: 0,
        behind_count: 0,
    };
    let mut app = App::new(
        ".",
        vec![branch],
        "main".to_string(),
        tx,
        ai_tx,
        conflict_tx,
    );
    app.refresh_filtered_branches();
    app
}

fn render_terminal(app: &mut App, width: u16, height: u16) -> (Terminal<TestBackend>, String) {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|f| draw(f, app, ".")).unwrap();
    let content = buffer_to_string(terminal.backend().buffer());
    (terminal, content)
}

fn buffer_to_string(buffer: &Buffer) -> String {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn buffer_rows(buffer: &Buffer) -> Vec<String> {
    (0..buffer.area.height)
        .map(|y| {
            (0..buffer.area.width)
                .map(|x| buffer[(x, y)].symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
}

#[test]
fn test_ui_draw_normal_table() {
    let mut app = create_test_app();
    let (_, content) = render_terminal(&mut app, 120, 30);
    assert!(content.contains("Branches"));
    assert!(content.contains("main"));
}

#[test]
fn test_ui_draw_empty_branches() {
    let (tx, _) = mpsc::channel(1);
    let (ai_tx, _) = mpsc::channel(1);
    let (conflict_tx, _) = mpsc::channel(1);
    let mut app = App::new(".", vec![], "main".to_string(), tx, ai_tx, conflict_tx);
    app.refresh_filtered_branches();
    let (_, content) = render_terminal(&mut app, 120, 30);
    assert!(content.contains("No branches match"));
}

#[test]
fn test_footer_pinned_to_last_row_without_toast() {
    let mut app = create_test_app();
    assert!(app.ui.toast.is_none());
    let (terminal, _) = render_terminal(&mut app, 120, 30);
    let rows = buffer_rows(terminal.backend().buffer());
    let last = &rows[rows.len() - 1];
    assert!(
        last.contains("twigdrop"),
        "footer must sit on last row, got: {last}"
    );
    let middle = &rows[rows.len() / 2];
    assert!(
        !middle.contains("twigdrop"),
        "main view must own the middle rows, got: {middle}"
    );
}

#[test]
fn test_toast_bar_sits_above_footer() {
    let mut app = create_test_app();
    app.ui.notify(
        crate::state::toast::ToastKind::Success,
        "1 branch(es) deleted".to_string(),
        None,
    );
    let (terminal, _) = render_terminal(&mut app, 120, 30);
    let rows = buffer_rows(terminal.backend().buffer());
    let h = rows.len();
    assert!(
        rows[h - 1].contains("twigdrop"),
        "footer stays last: {}",
        rows[h - 1]
    );
    assert!(
        rows[h - 2].contains("branch(es) deleted"),
        "toast sits above footer: {}",
        rows[h - 2]
    );
}

#[test]
fn test_ui_draw_narrow_collapses_author() {
    let mut app = create_test_app();
    let (_, content) = render_terminal(&mut app, 70, 24);
    assert!(content.contains("main"));
    assert!(!content.contains("Tester"));
}

#[test]
fn test_ui_draw_light_theme() {
    use crate::ui::theme::ThemeMode;
    let mut app = create_test_app();
    app.config.theme_mode = ThemeMode::Light;
    let (_, content) = render_terminal(&mut app, 120, 30);
    assert!(content.contains("Branches"));
    assert!(content.contains("main"));
}

#[test]
fn test_ui_draw_inspector_drawer() {
    let mut app = create_test_app();
    app.ui.show_inspector_drawer = true;
    let (_, content) = render_terminal(&mut app, 120, 30);
    assert!(content.contains("Inspector"));
    assert!(content.contains("Background Workers"));
}

#[test]
fn test_ui_draw_sidebar_drawer() {
    let mut app = create_test_app();
    app.ui.show_nav_sidebar = true;
    let (_, content) = render_terminal(&mut app, 120, 30);
    assert!(content.contains("Menu"));
    assert!(content.contains("Views"));
}

#[test]
fn test_ui_draw_command_palette_overlay() {
    let mut app = create_test_app();
    app.ui
        .push_modal(AppMode::CommandPalette(CommandPaletteState {
            query: String::new(),
            selected: 0,
            actions: vec![("Test Action".to_string(), CommandAction::OpenSettings)],
        }));

    let (terminal, content) = render_terminal(&mut app, 120, 30);
    assert!(content.contains("COMMAND PALETTE"));
    assert!(content.contains("Test Action"));

    // Verify dimming on a corner cell outside the modal window
    let corner_cell = &terminal.backend().buffer()[(0, 0)];
    let theme = crate::ui::theme::Theme::dark_default();
    assert_eq!(corner_cell.bg, theme.backdrop_bg);
    assert_eq!(corner_cell.fg, theme.backdrop_fg);
}
