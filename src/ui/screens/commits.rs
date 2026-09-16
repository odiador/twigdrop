use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::git::files::{FileEntry, FileStatus};
use crate::state::ui::RebaseAction;

pub fn render_commits(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)].as_ref())
        .split(area);

    let list_area = chunks[0];
    let details_area = chunks[1];

    let mut items = vec![];
    if app.repo.commit_tree.is_empty() {
        items.push(
            ListItem::new(
                " No commits found in history. Ensure this is a git repository with commits. ",
            )
            .style(Style::default().fg(Color::Yellow)),
        );
    } else {
        for (i, commit) in app.repo.commit_tree.iter().enumerate() {
            let is_selected = i == app.ui.selected_commit_idx;
            let mut graph_spans = Vec::new();

            for ch in commit.graph.chars() {
                let color = match ch {
                    '*' => Color::Magenta,
                    '|' | '/' | '\\' | '_' => Color::Rgb(100, 100, 120),
                    _ => Color::Gray,
                };
                graph_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }

            let mut commit_spans = vec![Span::styled(
                format!(" {} ", commit.hash),
                Style::default().fg(Color::Yellow),
            )];

            if !commit.branch_info.is_empty() {
                commit_spans.push(Span::styled(
                    format!(" {} ", commit.branch_info),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            commit_spans.extend(vec![
                Span::styled(
                    format!(" {} ", commit.date),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    format!(" [{}] ", commit.author),
                    Style::default().fg(Color::Rgb(180, 180, 200)),
                ),
                Span::styled(commit.message.clone(), Style::default().fg(Color::White)),
            ]);

            let mut all_spans = graph_spans;
            all_spans.append(&mut commit_spans);

            let mut line_style = Style::default();
            if is_selected {
                line_style = line_style.bg(Color::Rgb(80, 80, 100));
            }

            items.push(ListItem::new(Line::from(all_spans)).style(line_style));
        }
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Git Commit Tree (All Branches) ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, list_area);

    let details_block = Block::default()
        .title(" Commit Details ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    if let Some(commit) = app.repo.commit_tree.get(app.ui.selected_commit_idx) {
        let stats = app
            .repo
            .commit_stats
            .as_deref()
            .unwrap_or("Loading stats...");
        let diff = app.repo.commit_diff.as_deref().unwrap_or("Loading diff...");

        let mut detail_lines = vec![
            Line::from(vec![
                Span::styled("Hash: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&commit.hash, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("Author: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&commit.author, Style::default().fg(Color::White)),
            ]),
            Line::from(vec![
                Span::styled("Date: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&commit.date, Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("Message: ", Style::default().fg(Color::DarkGray)),
                Span::styled(
                    &commit.message,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
            Line::from(""),
            Line::from(Span::styled(
                "Stats:",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(stats),
            Line::from(""),
            Line::from(Span::styled(
                "Diff Preview:",
                Style::default()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            )),
        ];

        for line in diff.lines() {
            let style = if line.starts_with('+') {
                Style::default().fg(Color::Green)
            } else if line.starts_with('-') {
                Style::default().fg(Color::Red)
            } else if line.starts_with("@@") {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };
            detail_lines.push(Line::from(Span::styled(line, style)));
        }

        let p = Paragraph::new(detail_lines)
            .block(details_block)
            .scroll((app.ui.info_scroll, 0))
            .wrap(ratatui::widgets::Wrap { trim: false });

        f.render_widget(p, details_area);
    } else {
        f.render_widget(
            Paragraph::new("No commit selected").block(details_block),
            details_area,
        );
    }
}

pub fn render_commit_action(f: &mut Frame, app: &App, hash: &str) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(f.area())[1];
    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(area)[1];
    f.render_widget(Clear, inner);

    let options = [
        "1. Change Date (iOS Style Picker)",
        "2. Amend Staged Files (Fixup current)",
        "3. Browse Files (Surgical Undo / Move Forward)",
        "4. Squash into parent commit (Auto-rebase)",
        "5. Interactive Rebase from here",
    ];

    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.settings_state.selected {
            style = style
                .bg(Color::Rgb(80, 80, 100))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);
        }

        let text = if i == app.ui.settings_state.selected && app.ui.settings_state.editing && i == 0
        {
            format!("> {}", app.ui.settings_state.input)
        } else {
            opt.to_string()
        };

        items.push(ListItem::new(text).style(style));
    }

    let list = List::new(items).block(
        Block::default()
            .title(format!(" Manage Commit: {} ", hash))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(list, inner);
}

pub fn render_interactive_rebase(f: &mut Frame, app: &mut App) {
    let area = crate::ui::components::centered_rect(80, 80, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Interactive Rebase ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(203, 166, 247)));

    let items: Vec<ListItem> = app
        .ui
        .rebase_state
        .commits
        .iter()
        .enumerate()
        .map(|(i, commit)| {
            let action_str = match commit.action {
                RebaseAction::Pick => "pick  ",
                RebaseAction::Reword => "reword",
                RebaseAction::Drop => "drop  ",
                RebaseAction::Squash => "squash",
            };

            let action_color = match commit.action {
                RebaseAction::Pick => Color::DarkGray,
                RebaseAction::Reword => Color::Cyan,
                RebaseAction::Drop => Color::Red,
                RebaseAction::Squash => Color::Yellow,
            };

            let message = if let Some(new_msg) = &commit.new_message {
                new_msg.clone()
            } else {
                commit.original_message.clone()
            };

            let content = if app.ui.rebase_state.editing && app.ui.rebase_state.selected == i {
                format!(
                    "{} {} {}",
                    action_str, commit.hash, app.ui.rebase_state.input
                )
            } else {
                format!("{} {} {}", action_str, commit.hash, message)
            };

            let mut style = Style::default();
            if i == app.ui.rebase_state.selected {
                style = style.bg(Color::Rgb(49, 50, 68)).fg(Color::White);
            } else {
                style = style.fg(action_color);
            }

            ListItem::new(content).style(style)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(list, area);
}

pub fn render_commit_files(f: &mut Frame, app: &App, hash: &str, files: &[FileEntry]) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area)[1];

    f.render_widget(Clear, inner);

    let mut items = vec![];
    if files.is_empty() {
        items.push(
            ListItem::new(" No files found in this commit. ")
                .style(Style::default().fg(Color::Gray)),
        );
    } else {
        for (i, entry) in files.iter().enumerate() {
            let is_selected = i == app.ui.selected_commit_file_idx;
            let mut style = Style::default();

            let status_color = match entry.status {
                FileStatus::Added => Color::Green,
                FileStatus::Modified => Color::Yellow,
                FileStatus::Deleted => Color::Red,
                _ => Color::Gray,
            };

            if is_selected {
                style = style
                    .bg(Color::Rgb(80, 80, 100))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
            }

            let text = Line::from(vec![
                Span::styled(
                    format!(" {:?} ", entry.status),
                    Style::default().fg(status_color),
                ),
                Span::styled(entry.path.to_string_lossy().to_string(), Style::default()),
            ]);
            items.push(ListItem::new(text).style(style));
        }
    }

    let block = Block::default()
        .title(format!(" Files in commit {} ", hash))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list = List::new(items).block(block);
    f.render_widget(list, inner);

    // Footer for this modal
    let footer_area = Rect::new(inner.x + 1, inner.y + inner.height - 1, inner.width - 2, 1);
    f.render_widget(
        Paragraph::new(" ↑/↓: navigate │ u: Move changes forward │ Esc: close ")
            .style(Style::default().fg(Color::DarkGray)),
        footer_area,
    );
}
