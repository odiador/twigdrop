use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

pub const ASCII_LOGO: &str = r#"
 ████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
 ╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
    ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
    ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
    ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
    ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

pub fn render_help(f: &mut Frame, app: &App) {
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area);
    let area = crate::ui::components::centered_rect(84, 86, full_area);
    render_help_content(f, area, app);
}

pub fn render_help_content(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);
    let loc = app.locale();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // ASCII Banner
            Constraint::Min(12),   // 2-Column Content
            Constraint::Length(2), // Footer
        ])
        .split(area);

    // 1. Logo Banner
    let logo_block = Block::default()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::Rgb(203, 166, 247)))
        .style(Style::default().bg(Color::Rgb(24, 24, 37)));

    let logo = Paragraph::new(ASCII_LOGO.trim_matches('\n'))
        .style(
            Style::default()
                .fg(Color::Rgb(137, 220, 235))
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(logo_block);
    f.render_widget(logo, chunks[0]);

    // 2. Main 2-Column Body
    let body_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::Rgb(203, 166, 247)))
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));

    let body_area = chunks[1];
    f.render_widget(body_block, body_area);

    let cols = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(52), Constraint::Percentage(48)])
        .split(Rect::new(
            body_area.x + 2,
            body_area.y,
            body_area.width.saturating_sub(4),
            body_area.height,
        ));

    // Left Column: Branch Operations & Status
    let key_style = Style::default()
        .fg(Color::Rgb(249, 226, 175))
        .add_modifier(Modifier::BOLD);
    let desc_style = Style::default().fg(Color::Rgb(205, 214, 244));
    let header_style = Style::default()
        .fg(Color::Rgb(203, 166, 247))
        .add_modifier(Modifier::BOLD);

    let left_lines = vec![
        Line::from(Span::styled(loc.modals.help_branch_ops_title, header_style)),
        Line::from(vec![
            Span::styled("  Type [a-z0-9]  ", key_style),
            Span::styled(format!("{} {}", loc.branches.type_to_filter, loc.branches.esc_to_clear), desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ↑ / ↓          ", key_style),
            Span::styled("Navigate branches", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Space          ", key_style),
            Span::styled("Toggle selection (bulk)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Enter          ", key_style),
            Span::styled("Manage branch (Spotlight overlay)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  → / ←          ", key_style),
            Span::styled("Inspector drawer / Sidebar", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+P         ", key_style),
            Span::styled(loc.branches.act_checkout_desc, desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+D         ", key_style),
            Span::styled("Safe delete (with confirmation)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+S         ", key_style),
            Span::styled("Cycle sort (Recent, Prunable, A-Z)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+C         ", key_style),
            Span::styled(loc.modals.cmd_create_branch, desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+F         ", key_style),
            Span::styled("Open Filters modal", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+I         ", key_style),
            Span::styled("AI Intelligence Analysis", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(loc.modals.help_status_badges_title, header_style)),
        Line::from(vec![
            Span::styled(
                "  ▲ (Red)        ",
                Style::default()
                    .fg(Color::Rgb(243, 139, 168))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_unique_commits, desc_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  ⨯ (Gray)       ",
                Style::default()
                    .fg(Color::Rgb(147, 153, 178))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_gone, desc_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  ↑ / ↓          ",
                Style::default()
                    .fg(Color::Rgb(249, 226, 175))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_ahead_behind, desc_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  ✓ (Green)      ",
                Style::default()
                    .fg(Color::Rgb(166, 227, 161))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_merged, desc_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  [P] (Yellow)   ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_protected, desc_style),
        ]),
    ];
    f.render_widget(Paragraph::new(left_lines), cols[0]);

    // Right Column: Global Shortcuts & Views
    let right_lines = vec![
        Line::from(Span::styled(loc.modals.help_global_shortcuts_title, header_style)),
        Line::from(vec![
            Span::styled("  Ctrl+K         ", key_style),
            Span::styled("Spotlight Command Palette", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+H / ?     ", key_style),
            Span::styled("Toggle Help & Legend", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Ctrl+Q         ", key_style),
            Span::styled("Quit application safely", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Shift+Tab      ", key_style),
            Span::styled("App View Switcher", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Alt+D          ", key_style),
            Span::styled("Toggle Branches / Files view", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Esc            ", key_style),
            Span::styled("Clear search / Drawers / Main Menu", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Shift+C        ", key_style),
            Span::styled("Unpushed Commits Manager", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Shift+S        ", key_style),
            Span::styled("Stash Manager", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Shift+R        ", key_style),
            Span::styled("Interactive Rebase", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  Shift+F        ", key_style),
            Span::styled("AI Auto-Fix (in Diff mode)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  :              ", key_style),
            Span::styled("Git Quick Actions Palette", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  !              ", key_style),
            Span::styled("Execute Shell command", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(loc.modals.help_files_shortcuts_title, header_style)),
        Line::from(vec![
            Span::styled("  s              ", key_style),
            Span::styled(loc.files.stage_unstage, desc_style),
        ]),
        Line::from(vec![
            Span::styled("  e              ", key_style),
            Span::styled(loc.files.open_explorer, desc_style),
        ]),
        Line::from(vec![
            Span::styled("  v / a          ", key_style),
            Span::styled(loc.files.open_ide, desc_style),
        ]),
        Line::from(vec![
            Span::styled("  [ / ]          ", key_style),
            Span::styled(loc.files.resize_sidebar, desc_style),
        ]),
    ];
    f.render_widget(Paragraph::new(right_lines), cols[1]);

    // 3. Footer Banner
    let footer_block = Block::default()
        .borders(Borders::BOTTOM | Borders::LEFT | Borders::RIGHT)
        .border_style(Style::default().fg(Color::Rgb(203, 166, 247)))
        .style(Style::default().bg(Color::Rgb(24, 24, 37)));

    let footer_line = Line::from(vec![
        Span::styled(
            " [Esc / Ctrl+H / q] ",
            Style::default()
                .fg(Color::Rgb(243, 139, 168))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{}  •  ", loc.modals.help_footer_close), Style::default().fg(Color::Rgb(147, 153, 178))),
        Span::styled(
            "twigdrop ",
            Style::default()
                .fg(Color::Rgb(137, 220, 235))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("by ", Style::default().fg(Color::Rgb(147, 153, 178))),
        Span::styled(
            "odiador",
            Style::default()
                .fg(Color::Rgb(245, 194, 231))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" ❤️ {}", loc.modals.help_footer_credit), Style::default().fg(Color::Rgb(147, 153, 178))),
    ]);

    let footer_p = Paragraph::new(footer_line)
        .alignment(Alignment::Center)
        .block(footer_block);
    f.render_widget(footer_p, chunks[2]);
}
