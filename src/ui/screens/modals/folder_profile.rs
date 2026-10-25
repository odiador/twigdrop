use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::state::ui::FolderProfileState;
use crate::utils::profiles::FolderStatus;

pub fn render_folder_profile(f: &mut Frame, app: &App, state: &FolderProfileState) {
    let theme = app.theme();
    let area = crate::ui::components::centered_rect(70, 70, f.area());

    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    // 1. Header Card
    let header_block = Block::default()
        .title(Line::from(" 📁 Folder Profiles & Architecture Audit ").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_style(theme.border);
    let header_inner = header_block.inner(chunks[0]);
    f.render_widget(header_block, chunks[0]);

    let rep = &state.audit_report;
    let header_lines = vec![
        Line::from(vec![
            Span::styled(" Profile: ", theme.subtle),
            Span::styled(
                format!(" {} ", rep.profile_name),
                Style::default()
                    .bg(theme.surface)
                    .fg(theme.accent2.fg.unwrap_or(Color::Cyan))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  (Press [p] to cycle presets) ", theme.subtle),
        ]),
        Line::from(vec![
            Span::styled(
                format!("  ✓ Canonical: {}  ", rep.canonical_count),
                Style::default()
                    .bg(theme.diff_add_bg)
                    .fg(theme.diff_add_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("  + Missing: {}  ", rep.missing_count),
                Style::default()
                    .bg(theme.surface)
                    .fg(theme.warning.fg.unwrap_or(Color::Yellow))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
            Span::styled(
                format!("  ⚠ Drift: {}  ", rep.rogue_count),
                Style::default()
                    .bg(theme.diff_del_bg)
                    .fg(theme.diff_del_fg)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
    ];
    f.render_widget(Paragraph::new(header_lines), header_inner);

    // 2. Body List of Audit Items
    let body_block = Block::default()
        .title(format!(" Directory Audit ({}) ", rep.items.len()))
        .borders(Borders::ALL)
        .border_style(theme.active_border);
    let body_inner = body_block.inner(chunks[1]);
    f.render_widget(body_block, chunks[1]);

    let total = rep.items.len();
    let visible_rows = body_inner.height as usize;
    let selected = state.selected;

    let start_idx = if total <= visible_rows || selected < visible_rows / 2 {
        0
    } else if selected + visible_rows / 2 >= total {
        total.saturating_sub(visible_rows)
    } else {
        selected - visible_rows / 2
    };
    let end_idx = (start_idx + visible_rows).min(total);

    let mut body_lines = Vec::new();
    if rep.items.is_empty() {
        body_lines.push(Line::from(Span::styled(
            "  No directories found in profile.",
            theme.muted,
        )));
    } else {
        for (i, item) in rep.items[start_idx..end_idx].iter().enumerate() {
            let real_idx = start_idx + i;
            let is_sel = real_idx == selected;

            let pointer = if is_sel { "▸ " } else { "  " };
            let (badge, badge_style) = match item.status {
                FolderStatus::Canonical => (
                    " ✓ CANONICAL ",
                    Style::default()
                        .bg(theme.diff_add_bg)
                        .fg(theme.diff_add_fg)
                        .add_modifier(Modifier::BOLD),
                ),
                FolderStatus::Missing => (
                    " + MISSING   ",
                    Style::default()
                        .bg(theme.surface)
                        .fg(theme.warning.fg.unwrap_or(Color::Yellow))
                        .add_modifier(Modifier::BOLD),
                ),
                FolderStatus::Rogue => (
                    " ⚠ DRIFT     ",
                    Style::default()
                        .bg(theme.diff_del_bg)
                        .fg(theme.diff_del_fg)
                        .add_modifier(Modifier::BOLD),
                ),
            };

            let purpose_str = item.purpose.as_deref().unwrap_or("");
            let spans = vec![
                Span::styled(pointer, theme.accent.add_modifier(Modifier::BOLD)),
                Span::styled(badge, badge_style),
                Span::raw(" "),
                Span::styled(
                    format!("{:<25}", item.path),
                    if is_sel {
                        theme.base.add_modifier(Modifier::BOLD)
                    } else {
                        theme.base
                    },
                ),
                Span::styled(format!("  {purpose_str}"), theme.subtle),
            ];

            let mut line = Line::from(spans);
            if is_sel {
                line = line.style(Style::default().bg(theme.select_soft));
            }
            body_lines.push(line);
        }
    }
    f.render_widget(Paragraph::new(body_lines), body_inner);

    // 3. Footer Controls
    let footer_block = Block::default()
        .borders(Borders::ALL)
        .border_style(theme.border);
    let footer_inner = footer_block.inner(chunks[2]);
    f.render_widget(footer_block, chunks[2]);

    let footer_spans = vec![
        Span::styled(" [s] ", theme.accent.add_modifier(Modifier::BOLD)),
        Span::styled("Scaffold Missing   ", theme.base),
        Span::styled(" [w] ", theme.accent2.add_modifier(Modifier::BOLD)),
        Span::styled("Save Profile   ", theme.base),
        Span::styled(" [p] ", theme.peach.add_modifier(Modifier::BOLD)),
        Span::styled("Next Preset   ", theme.base),
        Span::styled(" [Esc] ", theme.subtle),
        Span::styled("Close", theme.muted),
    ];
    f.render_widget(
        Paragraph::new(Line::from(footer_spans)).alignment(Alignment::Center),
        footer_inner,
    );
}
