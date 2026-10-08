use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;
use crate::ui::theme::{MODAL_LG, Theme};

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
    let area = crate::ui::components::centered_rect(MODAL_LG.0, MODAL_LG.1, full_area);
    render_help_content(f, area, app);
}

pub fn render_help_content(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);
    let loc = app.locale();
    let theme = Theme::dark_default();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(crate::ui::layout::HELP_BANNER_H),
            Constraint::Min(crate::ui::layout::HELP_BODY_MIN_H),
            Constraint::Length(crate::ui::layout::HELP_FOOTER_H),
        ])
        .split(area);

    // 1. Logo Banner
    let logo_block = Block::default()
        .borders(Borders::TOP | Borders::LEFT | Borders::RIGHT)
        .border_style(theme.active_border)
        .style(Style::default().bg(theme.surface_alt));

    let logo = Paragraph::new(ASCII_LOGO.trim_matches('\n'))
        .style(
            theme.accent2.add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(logo_block);
    f.render_widget(logo, chunks[0]);

    // 2. Main 2-Column Body
    let body_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_style(theme.active_border)
        .style(Style::default().bg(theme.surface));

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
    // Key labels come from the canonical table (`crate::input::keys`) —
    // never hardcode them here, or help drifts from actual bindings.
    let key = |id: &str| format!("  {:<18}", crate::input::keys(id));
    let key_style = theme.warning.add_modifier(Modifier::BOLD);
    let desc_style = theme.base;
    let header_style = theme.accent.add_modifier(Modifier::BOLD);

    let left_lines = vec![
        Line::from(Span::styled(loc.modals.help_branch_ops_title, header_style)),
        Line::from(vec![
            Span::styled(key("branches.type_filter"), key_style),
            Span::styled(format!("{} {}", loc.branches.type_to_filter, loc.branches.esc_to_clear), desc_style),
        ]),
        Line::from(vec![
            Span::styled("  ↑ / ↓          ", key_style),
            Span::styled("Navigate branches", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("branches.toggle_select"), key_style),
            Span::styled("Toggle selection (bulk)", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("branches.manage"), key_style),
            Span::styled("Manage branch (Spotlight overlay)", desc_style),
        ]),
        Line::from(vec![
            Span::styled("  → / ←              ", key_style),
            Span::styled("Inspector drawer / Sidebar", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("branches.prune"), key_style),
            Span::styled(loc.branches.act_checkout_desc, desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("branches.safe_delete"), key_style),
            Span::styled("Safe delete (with confirmation)", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("branches.cycle_sort"), key_style),
            Span::styled("Cycle sort (Recent, Prunable, A-Z)", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("branches.create"), key_style),
            Span::styled(loc.modals.cmd_create_branch, desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("branches.filter"), key_style),
            Span::styled("Open Filters modal", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("ai.analyze"), key_style),
            Span::styled("AI Intelligence Analysis", desc_style),
        ]),
        Line::from(""),
        Line::from(Span::styled(loc.modals.help_status_badges_title, header_style)),
        Line::from(vec![
            Span::styled(
                "  ▲ (Red)        ",
                theme.danger.add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_unique_commits, desc_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  ⨯ (Gray)       ",
                theme.muted.add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_gone, desc_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  ↑ / ↓          ",
                theme.warning.add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_ahead_behind, desc_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  ✓ (Green)      ",
                theme.success.add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_merged, desc_style),
        ]),
        Line::from(vec![
            Span::styled(
                "  [P] (Yellow)         ",
                theme.warning.add_modifier(Modifier::BOLD),
            ),
            Span::styled(loc.branches.legend_protected, desc_style),
        ]),
    ];
    f.render_widget(Paragraph::new(left_lines), cols[0]);

    // Right Column: Global Shortcuts & Views (labels from BINDINGS)
    let right_lines = vec![
        Line::from(Span::styled(loc.modals.help_global_shortcuts_title, header_style)),
        Line::from(vec![
            Span::styled(key("global.palette"), key_style),
            Span::styled("Spotlight Command Palette", desc_style),
        ]),
        Line::from(vec![
            Span::styled(format!("  {:<18}", format!("{} / ?", crate::input::keys("global.help_toggle_ctrl"))), key_style),
            Span::styled("Toggle Help & Legend", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.quit"), key_style),
            Span::styled("Quit application safely", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.switcher"), key_style),
            Span::styled("App View Switcher", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.toggle_view"), key_style),
            Span::styled("Toggle Branches / Files view", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.close_back"), key_style),
            Span::styled("Clear search / Drawers / Main Menu", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.commits"), key_style),
            Span::styled("Unpushed Commits Manager", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.stash_mgr"), key_style),
            Span::styled("Stash Manager", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.rebase"), key_style),
            Span::styled("Interactive Rebase", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("diff.ai_autofix"), key_style),
            Span::styled("AI Auto-Fix (in Diff mode)", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.quick_actions"), key_style),
            Span::styled("Git Quick Actions Palette", desc_style),
        ]),
        Line::from(vec![
            Span::styled(key("global.shell"), key_style),
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
        .border_style(theme.active_border)
        .style(Style::default().bg(theme.surface_alt));

    let footer_line = Line::from(vec![
        Span::styled(
            format!(
                " [{} / {} / q] ",
                crate::input::keys("global.close_back"),
                crate::input::keys("global.help_toggle_ctrl")
            ),
            theme.danger.add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!("{}  •  ", loc.modals.help_footer_close), theme.muted),
        Span::styled(
            "twigdrop ",
            theme.accent2.add_modifier(Modifier::BOLD),
        ),
        Span::styled("by ", theme.muted),
        Span::styled(
            "odiador",
            theme.unique.add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" ❤️ {}", loc.modals.help_footer_credit), theme.muted),
    ]);

    let footer_p = Paragraph::new(footer_line)
        .alignment(Alignment::Center)
        .block(footer_block);
    f.render_widget(footer_p, chunks[2]);
}
