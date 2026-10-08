use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render_shell(f: &mut Frame, input: &str) {
    use crate::ui::layout::{SHELL_INPUT_H, SHELL_INPUT_W};
    use ratatui::layout::Rect;
    let full = f.area();
    let w = full.width * SHELL_INPUT_W / 100;
    let inner = Rect::new(
        full.x + full.width.saturating_sub(w) / 2,
        full.y + full.height.saturating_sub(SHELL_INPUT_H) / 2,
        w,
        SHELL_INPUT_H,
    );

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(" Execute Shell Command ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let p = Paragraph::new(format!("$ {}", input))
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(p, inner);
}

pub fn render_quick_actions(f: &mut Frame, app: &App) {
    let (mx, my) = crate::ui::layout::QUICK_ACTIONS_MODAL;
    let inner = crate::ui::components::centered_rect(mx, my, f.area());

    f.render_widget(Clear, inner);

    let mut items = vec![];
    for (i, action) in app.ui.quick_actions_state.actions.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.quick_actions_state.selected {
            style = crate::ui::theme::Theme::dark_default()
                .select_hi
                .add_modifier(Modifier::BOLD);
        }
        items.push(ListItem::new(format!(" {} ", action)).style(style));
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Git Quick Actions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, inner);
}
