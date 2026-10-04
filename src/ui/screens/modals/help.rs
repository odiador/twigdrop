use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::state::ui::PrimaryMode;

pub const ASCII_LOGO: &str = r#"
████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
   ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
   ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
   ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

pub fn render_help(f: &mut Frame, app: &App) {
    let area = crate::ui::components::centered_rect(80, 80, f.area());
    render_help_content(f, area, app);
}

pub fn render_help_content(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(8), // Logo
                Constraint::Min(10),   // Content
                Constraint::Length(1), // Footer
            ]
            .as_ref(),
        )
        .split(area);

    let logo = Paragraph::new(ASCII_LOGO.trim_matches('\n'))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(logo, chunks[0]);

    let block = Block::default()
        .title(Line::from(" Help & Legend ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    let mut help_text = vec![
        "Twigdrop helps you clean up your local branches safely.",
        "",
    ];

    if app.ui.primary_mode == PrimaryMode::Branches {
        help_text.extend(vec![
            "Status Icons (Branches):",
            "  ▲ (Red)     : Has Unique Commits (DANGER: Not in remote!)",
            "  ⨯ (Gray)    : Gone (Upstream branch was deleted)",
            "  ↑ (Yellow)  : Ahead of upstream (Local has new commits)",
            "  ↓ (Cyan)    : Behind upstream (Remote has new commits)",
            "  ✓ (Green)   : Merged (Safe to delete)",
            "  L (Lavender) : Local Only (No tracking branch)",
            "  S (Green)   : Stashed changes exist for this branch",
            "",
            "Shortcuts (Branches):",
            "  Type [a-z0-9]  : Instant type-to-search (Esc to clear)",
            "  ↑, ↓           : Navigate branch list",
            "  Space          : Toggle branch selection",
            "  Enter          : Manage branch (Spotlight overlay)",
            "  → / ←          : Toggle Inspector drawer / Nav sidebar",
            "  Ctrl+P         : Prune 'Gone' branches (Safe only)",
            "  Ctrl+D         : Bulk delete selected branches",
            "  Ctrl+S         : Cycle sort order (Recent, Prunable, A-Z)",
            "  Ctrl+F         : Open Filters",
            "  Ctrl+C         : Create new branch",
            "  Ctrl+I         : AI Intelligence Analysis for branch",
            "  Ctrl+K         : Command Palette (Spotlight)",
            "  Ctrl+H / ?     : Help & Legend",
            "  Esc            : Clear search / Close drawers / Main Menu",
        ]);
    } else {
        help_text.extend(vec![
            "Status Colors (Files):",
            "  Yellow      : Modified",
            "  Green       : Added / New",
            "  Blue        : Staged (in index)",
            "  Pink        : Untracked",
            "  Gray        : Ignored (.gitignore / .twigignore)",
            "",
            "Shortcuts (Files):",
            "  ↑/k, ↓/j    : Navigate tree",
            "  → / Enter   : Open folder / Move into children",
            "  ←           : Close folder / Move to parent",
            "  Enter       : Preview file content",
            "  e           : Open folder in Explorer (Alt+e for selected path)",
            "  s           : Stage / Unstage file",
            "  v           : Open in IDE (Root by default, Path with Alt)",
            "  t           : Internal TTY (Alt+t for External)",
            "  a           : Alt IDE (Root by default, Path with Alt)",
            "  [ / ]       : Expand / Contract sidebar width",
            "  Tab         : Switch focus between sidebar and preview",
        ]);
    }

    help_text.extend(vec![
        "",
        "Global Shortcuts:",
        "  d              : Switch between Branches and Files mode",
        "  Shift+Tab      : Open App Switcher",
        "  Esc            : Open Main Menu (Settings, Help, Quit)",
        "  !              : Open Shell Command Prompt",
        "  :              : Open Git Quick Actions Palette",
        "  ? / h          : Help & Legend",
        "  Shift+S        : Open Stash Manager",
        "  Shift+C        : Open Unpushed Commits Manager",
        "  Shift+F        : AI Auto-Fix (in Diff mode with conflicts)",
        "  Shift+R        : Interactive Rebase",
        "  Ctrl+o         : Open IDE (Root by default, Path with Alt)",
        "  q              : Quit",
    ]);

    let p = Paragraph::new(help_text.join("\n"))
        .block(block)
        .alignment(Alignment::Left);

    let help_inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(chunks[1])[1];

    f.render_widget(p, help_inner);

    let footer_text = vec![Line::from(vec![
        Span::raw("Made by: "),
        Span::styled(
            "odiador",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ❤️ for the community"),
    ])];
    let footer_p = Paragraph::new(footer_text).alignment(Alignment::Center);
    f.render_widget(footer_p, chunks[2]);
}
