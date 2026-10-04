use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::state::ui::CommandPaletteState;

pub fn render_command_palette(f: &mut Frame, state: &CommandPaletteState, locale: &crate::i18n::Locale) {
    let full_area = f.area();
    // 1. Dim the underlying terminal screen without clearing it
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area);

    // 2. Centered Spotlight floating window (55% width, 52% height)
    let inner = crate::ui::components::centered_rect(55, 52, full_area);
    f.render_widget(Clear, inner);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Search bar
            Constraint::Min(6),    // Action items list
            Constraint::Length(1), // Footer hint bar
        ])
        .split(inner);

    // Search bar block with Catppuccin Mauve accents
    let input_block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                format!(" ⚡ {} ", locale.modals.command_palette_title),
                Style::default()
                    .fg(Color::Rgb(203, 166, 247))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                "(Ctrl+K / Ctrl+P) ",
                Style::default().fg(Color::Rgb(147, 153, 178)),
            ),
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(203, 166, 247)))
        .style(Style::default().bg(Color::Rgb(24, 24, 37)));

    let search_line = if state.query.is_empty() {
        Line::from(vec![
            Span::styled(
                "   ",
                Style::default()
                    .fg(Color::Rgb(249, 226, 175))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                locale.modals.command_palette_placeholder,
                Style::default().fg(Color::Rgb(108, 112, 134)),
            ),
            Span::styled("▌", Style::default().fg(Color::Rgb(203, 166, 247))),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                "   ",
                Style::default()
                    .fg(Color::Rgb(249, 226, 175))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                &state.query,
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("▌", Style::default().fg(Color::Rgb(203, 166, 247))),
        ])
    };
    f.render_widget(Paragraph::new(search_line).block(input_block), chunks[0]);

    // Command List
    let list_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Rgb(49, 50, 68)))
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));

    let mut items = vec![];
    let mut visible_idx = 0;
    for (name, action) in &state.actions {
        if !state.query.is_empty() && !name.to_lowercase().contains(&state.query.to_lowercase()) {
            continue;
        }

        let is_selected = visible_idx == state.selected;
        let (tag, tag_color) = match action {
            crate::state::ui::CommandAction::CheckoutBranch
            | crate::state::ui::CommandAction::CreateBranch => {
                ("BRANCH", Color::Rgb(166, 227, 161))
            }
            crate::state::ui::CommandAction::Fetch
            | crate::state::ui::CommandAction::Pull
            | crate::state::ui::CommandAction::Push => ("SYNC", Color::Rgb(116, 199, 236)),
            crate::state::ui::CommandAction::StashChanges
            | crate::state::ui::CommandAction::PopStash => ("STASH", Color::Rgb(245, 194, 231)),
            crate::state::ui::CommandAction::OpenSettings => ("CONFIG", Color::Rgb(249, 226, 175)),
            _ => ("GIT", Color::Rgb(137, 180, 250)),
        };

        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::White)
        } else {
            Style::default().fg(Color::Rgb(205, 214, 244))
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
                Style::default().fg(Color::Rgb(205, 214, 244))
            },
        );

        let item_line = Line::from(vec![
            Span::styled(
                prefix,
                Style::default()
                    .fg(Color::Rgb(203, 166, 247))
                    .add_modifier(Modifier::BOLD),
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
            Style::default().fg(Color::Rgb(108, 112, 134)),
        )])));
    }

    f.render_widget(List::new(items).block(list_block), chunks[1]);

    // Footer Help Bar
    let footer_hints = Line::from(vec![
        Span::styled(
            "  ↑↓",
            Style::default()
                .fg(Color::Rgb(180, 190, 254))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}   ", locale.common.select),
            Style::default().fg(Color::Rgb(147, 153, 178)),
        ),
        Span::styled(
            "↵",
            Style::default()
                .fg(Color::Rgb(166, 227, 161))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}   ", locale.common.execute),
            Style::default().fg(Color::Rgb(147, 153, 178)),
        ),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Rgb(243, 139, 168))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", locale.common.cancel), Style::default().fg(Color::Rgb(147, 153, 178))),
    ]);
    f.render_widget(
        Paragraph::new(footer_hints).style(Style::default().bg(Color::Rgb(24, 24, 37))),
        chunks[2],
    );
}
