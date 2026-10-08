use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::state::ui::CommandPaletteState;
use crate::ui::theme::Theme;

pub fn render_command_palette(f: &mut Frame, state: &CommandPaletteState, locale: &crate::i18n::Locale) {
    let theme = Theme::dark_default();
    let full_area = f.area();
    // 1. Dim the underlying terminal screen without clearing it
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area);

    // 2. Centered Spotlight floating window (PALETTE_MODAL token)
    let (mx, my) = crate::ui::layout::PALETTE_MODAL;
    let inner = crate::ui::components::centered_rect(mx, my, full_area);
    f.render_widget(Clear, inner);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(crate::ui::layout::SPOTLIGHT_HEADER_H),
            Constraint::Min(crate::ui::layout::SPOTLIGHT_LIST_MIN_H),
            Constraint::Length(crate::ui::layout::SPOTLIGHT_FOOTER_H),
        ])
        .split(inner);

    // Search bar block with accent highlights
    let input_block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                format!(" ⚡ {} ", locale.modals.command_palette_title),
                theme.accent.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "(Ctrl+K / Ctrl+P) ",
                theme.muted,
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(theme.active_border)
        .style(Style::default().bg(theme.surface_alt));

    let search_line = if state.query.is_empty() {
        Line::from(vec![
            Span::styled(
                "   ",
                theme.warning.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                locale.modals.command_palette_placeholder,
                theme.subtle,
            ),
            Span::styled("▌", theme.accent),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "   ",
                theme.warning.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &state.query,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("▌", theme.accent),
        ])
    };
    f.render_widget(Paragraph::new(search_line).block(input_block), chunks[0]);

    // Command List
    let highlight_bg = theme.highlight.bg.unwrap_or(Color::DarkGray);
    let list_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(highlight_bg))
        .style(Style::default().bg(theme.surface));

    let mut items = vec![];
    let mut visible_idx = 0;
    for (name, action) in &state.actions {
        if !state.query.is_empty() && !name.to_lowercase().contains(&state.query.to_lowercase()) {
            continue;
        }

        let is_selected = visible_idx == state.selected;
        let success = theme.success.fg.unwrap_or(Color::Green);
        let (tag, tag_color) = match action {
            crate::state::ui::CommandAction::CheckoutBranch
            | crate::state::ui::CommandAction::CreateBranch => {
                ("BRANCH", success)
            }
            crate::state::ui::CommandAction::Fetch
            | crate::state::ui::CommandAction::Pull
            | crate::state::ui::CommandAction::Push => ("SYNC", theme.sync.fg.unwrap_or(Color::Blue)),
            crate::state::ui::CommandAction::StashChanges
            | crate::state::ui::CommandAction::PopStash => ("STASH", theme.unique.fg.unwrap_or(Color::Magenta)),
            crate::state::ui::CommandAction::OpenSettings => ("CONFIG", theme.warning.fg.unwrap_or(Color::Yellow)),
            _ => ("GIT", theme.info.fg.unwrap_or(Color::Blue)),
        };

        let row_style = if is_selected {
            theme.highlight
        } else {
            theme.base
        };

        let prefix = if is_selected { "▎ " } else { "  " };
        let tag_span = Span::styled(
            format!("[{}] ", tag),
            Style::default().fg(tag_color).add_modifier(Modifier::BOLD),
        );
        let name_span = Span::styled(
            name.clone(),
            if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                theme.base
            },
        );

        let item_line = Line::from(vec![
            Span::styled(
                prefix,
                theme.accent.add_modifier(Modifier::BOLD),
            ),
            tag_span,
            name_span,
        ]);

        items.push(ListItem::new(item_line).style(row_style));
        visible_idx += 1;
    }

    if items.is_empty() {
        items.push(ListItem::new(Line::from(vec![Span::styled(
            "   No matching commands found.",
            theme.subtle,
        )])));
    }

    f.render_widget(List::new(items).block(list_block), chunks[1]);

    // Footer Help Bar
    let footer_hints = Line::from(vec![
        Span::styled(
            "  ↑↓",
            theme.nav.add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}   ", locale.common.select),
            theme.muted,
        ),
        Span::styled(
            "↵",
            theme.success.add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}   ", locale.common.execute),
            theme.muted,
        ),
        Span::styled(
            "Esc",
            theme.danger.add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", locale.common.cancel), theme.muted),
    ]);
    f.render_widget(
        Paragraph::new(footer_hints).style(Style::default().bg(theme.surface_alt)),
        chunks[2],
    );
}
