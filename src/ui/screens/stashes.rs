use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render_stash_detail(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);
    let mut stash_items = vec![];
    for (i, stash) in app.repo.stashes.iter().enumerate() {
        let mut style = Style::default().fg(Color::Rgb(205, 214, 244));
        if i == app.ui.selected_stash_idx {
            style = style
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);
        }
        stash_items.push(
            ListItem::new(format!(
                "{} [{}] - {}",
                stash.id, stash.branch, stash.message
            ))
            .style(style),
        );
    }
    let stash_list =
        List::new(stash_items).block(Block::default().title(" Stashes ").borders(Borders::ALL));
    f.render_widget(stash_list, chunks[0]);

    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);
    let files_p = Paragraph::new(app.repo.stash_files.join("\n")).block(
        Block::default()
            .title(" Files in Stash ")
            .borders(Borders::ALL),
    );
    f.render_widget(files_p, detail_chunks[0]);
    let diff_p = Paragraph::new(app.repo.stash_diff.as_str())
        .block(
            Block::default()
                .title(" Diff Preview ")
                .borders(Borders::ALL),
        )
        .scroll((app.ui.info_scroll, 0));
    f.render_widget(diff_p, detail_chunks[1]);
}
