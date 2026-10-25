use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render_manage(f: &mut Frame, app: &App) {
    let full_area = f.area();
    let loc = app.locale();
    let theme = app.theme();

    // 1. Dim background
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area, &theme);

    // 2. Centered Spotlight floating window (MODAL_SM token)
    let inner = crate::ui::components::centered_rect(
        crate::ui::theme::MODAL_SM.0,
        crate::ui::theme::MODAL_SM.1,
        full_area,
    );
    f.render_widget(Clear, inner);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(crate::ui::layout::SPOTLIGHT_HEADER_H),
            Constraint::Min(crate::ui::layout::SPOTLIGHT_LIST_MIN_H),
            Constraint::Length(crate::ui::layout::SPOTLIGHT_FOOTER_H),
        ])
        .split(inner);

    let b_name = app
        .get_filtered_branches()
        .get(app.ui.selected_branch_idx)
        .map(|b| b.name.as_str())
        .unwrap_or(loc.common.none);

    let is_protected = app.config.is_protected(b_name);

    let header_block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                format!(" {} ", loc.branches.manage_title),
                theme.accent.add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("({b_name}) "), theme.muted),
            if is_protected {
                Span::styled(
                    "[PROTECTED] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            },
        ]))
        .borders(Borders::ALL)
        .border_style(theme.active_border)
        .style(Style::default().bg(theme.surface_alt));

    let header_text = Line::from(vec![
        Span::styled(
            format!("  {}", loc.branches.manage_target_prefix),
            theme.muted,
        ),
        Span::styled(b_name, theme.accent2.add_modifier(Modifier::BOLD)),
        Span::styled(loc.branches.manage_instruction, theme.subtle),
    ]);
    f.render_widget(Paragraph::new(header_text).block(header_block), chunks[0]);

    let highlight_bg = theme.highlight.bg.unwrap_or(Color::DarkGray);
    let list_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(highlight_bg))
        .style(Style::default().bg(theme.surface));

    let success = theme.success.fg.unwrap_or(Color::Green);
    let danger = theme.danger.fg.unwrap_or(Color::Red);
    let actions = [
        (
            loc.branches.act_checkout,
            loc.branches.act_checkout_desc,
            success,
        ),
        (
            loc.branches.act_diff_ai,
            loc.branches.act_diff_ai_desc,
            theme.info.fg.unwrap_or(Color::Blue),
        ),
        (
            loc.branches.act_delete,
            loc.branches.act_delete_desc,
            danger,
        ),
        (
            loc.branches.act_rename,
            loc.branches.act_rename_desc,
            theme.warning.fg.unwrap_or(Color::Yellow),
        ),
        (
            loc.branches.act_push,
            loc.branches.act_push_desc,
            theme.accent2.fg.unwrap_or(Color::Cyan),
        ),
        (
            loc.branches.act_upstream,
            loc.branches.act_upstream_desc,
            theme.info.fg.unwrap_or(Color::Blue),
        ),
        (
            loc.branches.act_merge,
            loc.branches.act_merge_desc,
            theme.success.fg.unwrap_or(Color::Green),
        ),
        (
            loc.branches.act_copy_name,
            loc.branches.act_copy_name_desc,
            theme.soft.fg.unwrap_or(Color::White),
        ),
        (
            loc.branches.act_stash,
            loc.branches.act_stash_desc,
            theme.unique.fg.unwrap_or(Color::Magenta),
        ),
        (
            loc.branches.act_help,
            loc.branches.act_help_desc,
            theme.muted.fg.unwrap_or(Color::Gray),
        ),
        (
            loc.branches.act_cancel,
            loc.branches.act_cancel_desc,
            theme.subtle.fg.unwrap_or(Color::DarkGray),
        ),
    ];

    let mut items = vec![];
    for (i, (tag, desc, tag_color)) in actions.iter().enumerate() {
        let is_selected = i == app.ui.manage_selected;
        let row_style = if is_selected {
            theme.highlight
        } else {
            theme.base
        };

        let prefix = if is_selected { "▎ " } else { "  " };
        let tag_span = Span::styled(
            format!("[{tag}] "),
            Style::default().fg(*tag_color).add_modifier(Modifier::BOLD),
        );
        let desc_span = Span::styled(
            *desc,
            if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.base
            },
        );

        let item_line = Line::from(vec![
            Span::styled(prefix, theme.accent.add_modifier(Modifier::BOLD)),
            tag_span,
            desc_span,
        ]);
        items.push(ListItem::new(item_line).style(row_style));
    }

    f.render_widget(List::new(items).block(list_block), chunks[1]);

    let footer_hints = Line::from(vec![
        Span::styled("  ↑↓", theme.nav.add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {}   ", loc.common.select), theme.muted),
        Span::styled("↵", theme.success.add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {}   ", loc.common.execute), theme.muted),
        Span::styled("Esc", theme.danger.add_modifier(Modifier::BOLD)),
        Span::styled(format!(" {}", loc.common.cancel), theme.muted),
    ]);
    f.render_widget(
        Paragraph::new(footer_hints).style(Style::default().bg(theme.surface_alt)),
        chunks[2],
    );
}
