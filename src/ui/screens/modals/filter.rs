use ratatui::{
    Frame,
    layout::Alignment,
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::app::App;

pub fn render_filter(f: &mut Frame, app: &App) {
    let theme = app.theme();
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area, &theme);
    let (mx, my) = crate::ui::theme::MODAL_MD;
    let inner = crate::ui::components::centered_rect(mx, my, full_area);
    f.render_widget(Clear, inner);

    let options = [
        "0. All",
        "1. Merged (✓)",
        "2. Local Only (L)",
        "3. Stashed (S)",
        "4. Gone (⨯)",
        "5. Ahead (↑)",
        "6. Behind (↓)",
        "7. Unique Commits (▲)",
        "8. Remote Tracked (R)",
        "9. Remote Untracked (U)",
    ];
    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = theme.base;
        if i == app.ui.filter_selected {
            style = theme.highlight;
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let block = Block::default()
        .title(Line::from(" Filter by Status ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(theme.border);
    let list = List::new(items).block(block);
    f.render_widget(list, inner);
}
