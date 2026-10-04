use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::app::App;

pub fn render_search(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area())[0];
    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(format!("> {}", app.ui.search_query)).block(
            Block::default()
                .title(" Search Branches ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        ),
        area,
    );
}
