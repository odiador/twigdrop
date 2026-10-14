use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::app::App;

pub fn render_switcher(f: &mut Frame, app: &App) {
    let theme = app.theme();
    let (mx, my) = crate::ui::layout::SWITCHER_MODAL;
    let inner = crate::ui::components::centered_rect(mx, my, f.area());

    f.render_widget(Clear, inner);

    let mut items = Vec::new();
    let order = app.ui.switcher_order();
    for (i, target) in order.iter().enumerate() {
        let text = target.label();
        let mut style = Style::default().fg(Color::Gray);

        if i == app.ui.switcher_index {
            style = theme.select_hi.add_modifier(Modifier::BOLD);
        }
        items.push(ListItem::new(text).style(style));
    }

    if items.is_empty() {
        items.push(ListItem::new("  No history  ").style(Style::default().fg(Color::Gray)));
    }

    let list = List::new(items).block(
        Block::default()
            .title(ratatui::text::Line::from(" App Switcher ").alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, inner);
}
