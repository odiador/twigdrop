use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::ui::theme::Theme;

pub fn render_message(f: &mut Frame, msg: &str) {
    let theme = Theme::dark_default();
    let (mx, my) = crate::ui::theme::MODAL_MD;
    let inner = crate::ui::components::centered_rect(mx, my, f.area());

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(Line::from(" Git Response ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(theme.msg_bg));

    let p = Paragraph::new(msg)
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(p, inner);
}
