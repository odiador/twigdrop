use crate::app::App;
use crate::state::ui::ConflictResolverState;
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

pub fn render_conflict_resolver(f: &mut Frame, app: &App, state: &ConflictResolverState) {
    let full_area = f.area();
    let theme = app.theme();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area, &theme);

    let (mx, my) = (90, 80);
    let inner = crate::ui::components::centered_rect(mx, my, full_area);
    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(" ⚔️ Visual 3-Way Conflict Resolver ")
        .borders(Borders::ALL)
        .border_style(theme.conflict.add_modifier(Modifier::BOLD));
    let content_area = block.inner(inner);
    f.render_widget(block, inner);

    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(2)].as_ref())
        .split(content_area);

    let panels = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(30), Constraint::Min(20)].as_ref())
        .split(main_chunks[0]);

    // 1. Left Panel: Conflicted files list
    let file_block = Block::default()
        .title(" Conflicted Files ")
        .borders(Borders::ALL)
        .border_style(theme.border);
    let file_inner = file_block.inner(panels[0]);
    f.render_widget(file_block, panels[0]);

    let mut file_items = Vec::new();
    if state.files.is_empty() {
        file_items.push(ListItem::new(Line::from(Span::styled(
            " No conflicts! ",
            theme.success,
        ))));
    } else {
        for (idx, file) in state.files.iter().enumerate() {
            let is_selected = idx == state.selected_file_idx;
            let pointer = if is_selected { "▸ " } else { "  " };
            let style = if is_selected {
                Style::default().bg(theme.select_soft).fg(Color::White)
            } else {
                theme.base
            };
            let short_name = crate::ui::components::truncate_ellipsis(file, 22);
            file_items.push(
                ListItem::new(Line::from(vec![
                    Span::styled(
                        pointer,
                        if is_selected {
                            theme.accent
                        } else {
                            theme.subtle
                        },
                    ),
                    Span::styled(short_name, style),
                ]))
                .style(style),
            );
        }
    }
    f.render_widget(List::new(file_items), file_inner);

    // 2. Right Panel: Conflict hunks view
    let hunk_block = Block::default()
        .title(" Hunk Resolution ")
        .borders(Borders::ALL)
        .border_style(theme.border);
    let hunk_inner = hunk_block.inner(panels[1]);
    f.render_widget(hunk_block, panels[1]);

    if state.hunks.is_empty() {
        let msg = vec![
            Line::default(),
            Line::from(Span::styled(
                "  ✓ This file is fully resolved and staged!",
                theme.success.add_modifier(Modifier::BOLD),
            )),
            Line::default(),
            Line::from(Span::styled(
                "  Press 'c' to continue merge/rebase when all files are resolved.",
                theme.subtle,
            )),
        ];
        f.render_widget(Paragraph::new(msg), hunk_inner);
    } else {
        let active_hunk_idx = state
            .selected_hunk_idx
            .min(state.hunks.len().saturating_sub(1));
        let hunk = &state.hunks[active_hunk_idx];

        let hunk_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),      // Hunk counter + action bar
                Constraint::Percentage(50), // Ours
                Constraint::Percentage(50), // Theirs
            ])
            .split(hunk_inner);

        // Header bar with resolution shortcuts
        let header_spans = vec![
            Span::styled(
                format!(" Hunk {} of {}: ", active_hunk_idx + 1, state.hunks.len()),
                theme.peach.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " [o: Accept Ours] ",
                Style::default()
                    .bg(theme.diff_add_bg)
                    .fg(theme.diff_add_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                " [t: Accept Theirs] ",
                Style::default()
                    .bg(theme.surface)
                    .fg(theme.accent.fg.unwrap_or(Color::Cyan))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(
                " [b: Accept Both] ",
                Style::default()
                    .bg(theme.surface)
                    .fg(theme.warning.fg.unwrap_or(Color::Yellow))
                    .add_modifier(Modifier::BOLD),
            ),
        ];
        f.render_widget(Paragraph::new(Line::from(header_spans)), hunk_layout[0]);

        // Block 1: Ours / Current
        let ours_block = Block::default()
            .title(format!(" ◀ OURS / CURRENT: {} ", hunk.our_label))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.diff_add_fg));
        let ours_inner = ours_block.inner(hunk_layout[1]);
        f.render_widget(ours_block, hunk_layout[1]);

        let mut ours_lines = Vec::new();
        if hunk.our_lines.is_empty() {
            ours_lines.push(Line::from(Span::styled("  (empty)", theme.dim)));
        } else {
            for line in &hunk.our_lines {
                ours_lines.push(Line::from(vec![
                    Span::styled(" │ ", Style::default().fg(theme.diff_add_fg)),
                    Span::styled(line.clone(), theme.base),
                ]));
            }
        }
        f.render_widget(Paragraph::new(ours_lines), ours_inner);

        // Block 2: Theirs / Incoming
        let theirs_block = Block::default()
            .title(format!(" ▶ THEIRS / INCOMING: {} ", hunk.their_label))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.accent.fg.unwrap_or(Color::Cyan)));
        let theirs_inner = theirs_block.inner(hunk_layout[2]);
        f.render_widget(theirs_block, hunk_layout[2]);

        let mut theirs_lines = Vec::new();
        if hunk.their_lines.is_empty() {
            theirs_lines.push(Line::from(Span::styled("  (empty)", theme.dim)));
        } else {
            for line in &hunk.their_lines {
                theirs_lines.push(Line::from(vec![
                    Span::styled(
                        " │ ",
                        Style::default().fg(theme.accent.fg.unwrap_or(Color::Cyan)),
                    ),
                    Span::styled(line.clone(), theme.base),
                ]));
            }
        }
        f.render_widget(Paragraph::new(theirs_lines), theirs_inner);
    }

    // 3. Footer Bar with universal shortcuts
    let footer_text = Line::from(vec![Span::styled(
        " (o: accept ours │ t: accept theirs │ b: both │ n/p: next/prev hunk │ Tab: files │ c: continue │ Esc: close) ",
        theme.subtle,
    )]);
    f.render_widget(
        Paragraph::new(footer_text).alignment(Alignment::Center),
        main_chunks[1],
    );
}
