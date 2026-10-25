use crate::app::App;
use crate::state::ui::{PrReviewKind, PrReviewModalState};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn render_pr_review(f: &mut Frame, app: &App, state: &PrReviewModalState) {
    let full_area = f.area();
    let theme = app.theme();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area, &theme);

    let (mx, my) = (72, 14);
    let inner = crate::ui::components::centered_rect(mx, my, full_area);
    f.render_widget(Clear, inner);

    let (title, border_color, action_hint) = match state.kind {
        PrReviewKind::Approve => (
            " ✓ Approve Pull Request ",
            theme.success.fg.unwrap_or(Color::Green),
            " [Enter: Approve • Esc: Cancel] ",
        ),
        PrReviewKind::RequestChanges => (
            " ✗ Request Changes ",
            theme.danger.fg.unwrap_or(Color::Red),
            " [Enter: Request Changes • Esc: Cancel] ",
        ),
        PrReviewKind::Comment => (
            " 💬 Comment on Pull Request ",
            theme.nav.fg.unwrap_or(Color::Cyan),
            " [Enter: Post Comment • Esc: Cancel] ",
        ),
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(
            Style::default()
                .fg(border_color)
                .add_modifier(Modifier::BOLD),
        );
    let content_area = block.inner(inner);
    f.render_widget(block, inner);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Info header
            Constraint::Min(4),    // Input area
            Constraint::Length(1), // Hints
        ])
        .split(content_area);

    let info_lines = vec![
        Line::from(vec![
            Span::styled("Pull Request: ", theme.soft),
            Span::styled(
                format!("#{}", state.pr_number),
                theme.accent2.add_modifier(Modifier::BOLD),
            ),
            Span::raw(" - "),
            Span::styled(
                crate::ui::components::truncate_ellipsis(&state.pr_title, 45),
                theme.base.add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(match state.kind {
            PrReviewKind::Approve => {
                Span::styled("Submit approval. Optional review comment:", theme.muted)
            }
            PrReviewKind::RequestChanges => Span::styled(
                "Reason for requesting changes (required):",
                theme.warning.add_modifier(Modifier::BOLD),
            ),
            PrReviewKind::Comment => {
                Span::styled("Leave a review comment for the author:", theme.muted)
            }
        }),
    ];
    f.render_widget(Paragraph::new(info_lines), chunks[0]);

    let input_block = Block::default()
        .title(" Message ")
        .borders(Borders::ALL)
        .border_style(theme.border);
    let input_inner = input_block.inner(chunks[1]);
    f.render_widget(input_block, chunks[1]);

    let input_text = if state.body.is_empty() {
        Line::from(vec![
            Span::styled("Type message here... ", theme.dim),
            Span::styled("█", theme.accent2),
        ])
    } else {
        Line::from(vec![
            Span::styled(&state.body, theme.base),
            Span::styled("█", theme.accent2),
        ])
    };
    f.render_widget(Paragraph::new(input_text), input_inner);

    let hint_p = Paragraph::new(action_hint)
        .alignment(Alignment::Center)
        .style(theme.muted);
    f.render_widget(hint_p, chunks[2]);
}
