use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;
use crate::state::ui::PrimaryMode;
use crate::ui::theme::Theme;

pub fn render_sidebar_menu(f: &mut Frame, area: Rect, app: &App) {
    let theme = Theme::dark_default();
    let block = Block::default()
        .title(" ◀ Menu [→/Esc] ")
        .title_style(
            theme.accent.add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(theme.border_soft);

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(crate::ui::layout::SIDEBAR_VIEWS_H),
            Constraint::Length(crate::ui::layout::SIDEBAR_FILTERS_H),
            Constraint::Min(crate::ui::layout::SIDEBAR_WORKTREES_MIN_H),
        ])
        .split(inner);

    // 1. Views
    let views = [
        ("🌿 Branches", PrimaryMode::Branches),
        ("📁 Changed Files", PrimaryMode::Files),
        ("📜 Commit Graph", PrimaryMode::Commits),
        ("🗃️ Stashes", PrimaryMode::Stashes),
    ];

    let mut view_lines = vec![];
    for (i, (label, mode)) in views.iter().enumerate() {
        let is_active = app.ui.primary_mode == *mode;
        let is_hovered = app.ui.nav_sidebar_selected == i;

        let style = if is_hovered {
            theme.highlight.add_modifier(Modifier::BOLD)
        } else if is_active {
            theme.info.add_modifier(Modifier::BOLD)
        } else {
            theme.soft
        };

        let prefix = if is_hovered {
            "▎ "
        } else if is_active {
            "● "
        } else {
            "  "
        };
        view_lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(*label, style),
        ]));
    }

    f.render_widget(
        Paragraph::new(view_lines).block(
            Block::default()
                .title(" Views ")
                .title_style(Style::default().fg(Color::Gray))
                .borders(Borders::BOTTOM)
                .border_style(theme.divider),
        ),
        chunks[0],
    );

    // 2. Filter Scopes
    let filters = ["All Branches", "Prunable (Gone)", "Protected Only"];
    let mut filter_lines = vec![];
    for (idx, name) in filters.iter().enumerate() {
        let is_hovered = app.ui.nav_sidebar_selected == (4 + idx);
        let style = if is_hovered {
            theme.highlight.add_modifier(Modifier::BOLD)
        } else {
            theme.soft
        };
        let prefix = if is_hovered { "▎ " } else { "  " };
        filter_lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::styled(*name, style),
        ]));
    }

    f.render_widget(
        Paragraph::new(filter_lines).block(
            Block::default()
                .title(" Filters ")
                .title_style(Style::default().fg(Color::Gray))
                .borders(Borders::BOTTOM)
                .border_style(theme.divider),
        ),
        chunks[1],
    );

    // 3. Worktrees List (cached in update — no git in render)
    let mut worktree_lines = vec![];
    for (name, branch) in &app.repo.worktrees {
        worktree_lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::styled(name.clone(), Style::default().fg(Color::White)),
            Span::styled(format!(" [{}]", branch), theme.accent2),
        ]));
    }

    if worktree_lines.is_empty() {
        worktree_lines.push(Line::from(vec![
            Span::styled("• ", Style::default().fg(Color::Green)),
            Span::styled("main (default)", Style::default().fg(Color::White)),
        ]));
    }

    f.render_widget(
        Paragraph::new(worktree_lines).block(
            Block::default()
                .title(" Worktrees ")
                .title_style(Style::default().fg(Color::Gray)),
        ),
        chunks[2],
    );
}
