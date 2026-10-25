use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;

pub fn render_stash_detail(f: &mut Frame, area: Rect, app: &App) {
    use crate::ui::layout::STASH_SPLIT;
    let theme = app.theme();
    f.render_widget(Clear, area);
    let constraints = if app.ui.maximized_panel {
        [Constraint::Length(0), Constraint::Min(0)]
    } else {
        [
            Constraint::Percentage(STASH_SPLIT.0),
            Constraint::Percentage(STASH_SPLIT.1),
        ]
    };
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.as_ref())
        .split(area);
    let mut stash_items = vec![];
    if app.repo.stashes.is_empty() {
        stash_items.push(ListItem::new("  No stashes found.").style(theme.dim));
        stash_items
            .push(ListItem::new("  Use 'git stash' to save working changes.").style(theme.subtle));
    } else {
        for (i, stash) in app.repo.stashes.iter().enumerate() {
            let mut style = theme.base;
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
    }
    if !app.ui.maximized_panel {
        let stash_list =
            List::new(stash_items).block(Block::default().title(" Stashes ").borders(Borders::ALL));
        f.render_widget(stash_list, chunks[0]);
    }

    let detail_constraints = if app.ui.maximized_panel {
        [Constraint::Length(0), Constraint::Min(0)]
    } else {
        [
            Constraint::Percentage(STASH_SPLIT.0),
            Constraint::Percentage(STASH_SPLIT.1),
        ]
    };
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(detail_constraints.as_ref())
        .split(chunks[1]);

    if !app.ui.maximized_panel {
        let files_p = Paragraph::new(app.repo.stash_files.join("\n")).block(
            Block::default()
                .title(" Files in Stash ")
                .borders(Borders::ALL),
        );
        f.render_widget(files_p, detail_chunks[0]);
    }

    let diff_lines = crate::ui::components::format_diff_with_gutters(&app.repo.stash_diff, &theme);
    let diff_visible_rows = detail_chunks[1].height.saturating_sub(2) as usize;
    let scroll_badge = crate::ui::components::scroll_indicator(
        app.ui.info_scroll as usize,
        diff_visible_rows,
        diff_lines.len(),
    );
    let zoom_hint = if app.ui.maximized_panel {
        "[Z: restore] "
    } else {
        "[Z: zoom] "
    };
    let diff_title = format!(" Diff Preview {} {}[d/u: scroll] ", scroll_badge, zoom_hint);

    let diff_p = Paragraph::new(diff_lines)
        .block(
            Block::default()
                .title(diff_title)
                .borders(Borders::ALL)
                .border_style(if app.ui.maximized_panel {
                    theme.active_border
                } else {
                    theme.border
                }),
        )
        .scroll((app.ui.info_scroll, 0))
        .wrap(ratatui::widgets::Wrap { trim: false });
    f.render_widget(diff_p, detail_chunks[1]);
}
