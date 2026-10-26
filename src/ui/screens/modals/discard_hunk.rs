use crate::app::App;
use crate::state::ui::ConfirmDiscardHunkState;
use ratatui::{
    Frame,
    layout::Alignment,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render_confirm_discard_hunk(f: &mut Frame, app: &App, state: &ConfirmDiscardHunkState) {
    let theme = app.theme();
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area, &theme);

    let (mx, my) = (56, 32);
    let inner = crate::ui::components::centered_rect(mx, my, full_area);
    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(" ⚠️ Confirm Discard Hunk ")
        .borders(Borders::ALL)
        .border_style(theme.danger.add_modifier(Modifier::BOLD));
    let content_area = block.inner(inner);
    f.render_widget(block, inner);

    let line_range = if state.new_lines <= 1 {
        format!("Line {}", state.new_start)
    } else {
        format!(
            "Lines {}-{}",
            state.new_start,
            state.new_start + state.new_lines.saturating_sub(1)
        )
    };

    let lines = vec![
        Line::default(),
        Line::from(vec![
            Span::styled(" Are you sure you want to discard ", theme.base),
            Span::styled(
                format!("hunk #{}", state.hunk_idx + 1),
                theme.peach.add_modifier(Modifier::BOLD),
            ),
            Span::styled("?", theme.base),
        ])
        .alignment(Alignment::Center),
        Line::default(),
        Line::from(vec![
            Span::styled(" File:  ", theme.dim),
            Span::styled(&state.file_path, theme.accent),
        ])
        .alignment(Alignment::Center),
        Line::from(vec![
            Span::styled(" Range: ", theme.dim),
            Span::styled(line_range, theme.base),
        ])
        .alignment(Alignment::Center),
        Line::default(),
        Line::from(Span::styled(
            " This action will permanently revert these local changes in your working tree! ",
            theme.danger,
        ))
        .alignment(Alignment::Center),
        Line::default(),
        Line::from(vec![
            Span::styled(
                " [y / Enter: Discard] ",
                theme.danger.add_modifier(Modifier::BOLD),
            ),
            Span::raw("    "),
            Span::styled(" [n / Esc: Cancel] ", theme.subtle),
        ])
        .alignment(Alignment::Center),
    ];

    f.render_widget(Paragraph::new(lines), content_area);
}
