use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::app::App;
use crate::models::BranchStatus;
use crate::state::ui::AppMode;
use crate::ui::components::get_status_icons;

pub fn render_main_list(f: &mut Frame, area: Rect, app: &mut App) {
    let filtered_indices = app.ui.filtered_indices.clone();
    let branches_len = filtered_indices.len();
    let inner_height = area.height.saturating_sub(4) as usize;

    if inner_height == 0 {
        return;
    }

    let mut start = 0;
    if branches_len > inner_height {
        let half_height = inner_height / 2;
        if app.ui.selected_branch_idx > half_height {
            start = app.ui.selected_branch_idx - half_height;
        }
        let mut end = start + inner_height;
        if end > branches_len {
            end = branches_len;
            start = end.saturating_sub(inner_height);
        }
    }
    app.ui.list_start_index = start;

    let mut rows: Vec<Row> = vec![];
    app.ui.branch_screen_positions.clear();

    let branch_items_to_show = inner_height.min(branches_len.saturating_sub(start));

    for i in 0..branch_items_to_show {
        let branch_idx = start + i;
        if branch_idx >= branches_len {
            break;
        }

        let actual_idx = filtered_indices[branch_idx];
        let b = &app.repo.branches[actual_idx];
        let selected = branch_idx == app.ui.selected_branch_idx;
        let is_current = b.name == app.repo.current_branch;

        app.ui
            .branch_screen_positions
            .push((actual_idx, area.y + 3 + i as u16));

        let (icons, color) = get_status_icons(&b.status);
        let (merge_text, merge_color) =
            crate::ui::components::get_merge_status_display(&b.merge_status);

        let current_tag = if is_current { " (current)" } else { "" };
        let is_bulk_selected = app.ui.bulk_selected.contains(&b.name);
        let checkbox = if is_bulk_selected { "[x]" } else { "[ ]" };

        let mut diff_counts = String::new();
        if b.ahead_count > 0 {
            diff_counts.push_str(&format!("↑{} ", b.ahead_count));
        }
        if b.behind_count > 0 {
            diff_counts.push_str(&format!("↓{} ", b.behind_count));
        }
        let diff_counts_str = if diff_counts.is_empty() {
            String::new()
        } else {
            format!(" [{}]", diff_counts.trim_end())
        };

        let branch_name = format!("{}{}{}", b.name, current_tag, diff_counts_str);
        let status_str = if b.status.contains(&BranchStatus::Merged) {
            "merged"
        } else {
            "unmerged"
        };
        let type_str = if b.status.contains(&BranchStatus::RemoteTracked) {
            "remote"
        } else {
            "local"
        };
        let author_str = format!("{} {}", b.commit_date, b.author);

        let mut row_style = Style::default().fg(Color::Rgb(205, 214, 244));
        let mut branch_style = Style::default().fg(color);
        if is_current {
            branch_style = branch_style.add_modifier(Modifier::BOLD).fg(Color::White);
        }

        if selected {
            row_style = row_style.bg(Color::White).fg(Color::Black);
            branch_style = branch_style.fg(Color::Black);
        }

        let cells = vec![
            Cell::from(checkbox).style(if selected {
                Style::default().fg(Color::Black)
            } else {
                Style::default().fg(Color::Rgb(124, 128, 156))
            }),
            Cell::from(Line::from(vec![
                Span::styled(
                    format!("{:<4} ", icons),
                    if selected {
                        Style::default().fg(Color::Black)
                    } else {
                        Style::default().fg(color)
                    },
                ),
                Span::styled(branch_name, branch_style),
            ])),
            Cell::from(b.age.clone()),
            Cell::from(status_str),
            Cell::from(merge_text).style(if selected {
                Style::default().fg(Color::Black)
            } else {
                Style::default().fg(merge_color)
            }),
            Cell::from(type_str),
            Cell::from(author_str),
        ];

        rows.push(Row::new(cells).style(row_style));
    }

    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(35),
        Constraint::Length(12),
        Constraint::Length(10),
        Constraint::Length(15),
        Constraint::Length(8),
        Constraint::Percentage(25),
    ];

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                "",
                "Branch",
                "Age",
                "Status",
                "Merge",
                "Type",
                "Last Commit",
            ])
            .style(
                Style::default()
                    .fg(Color::Rgb(124, 128, 156))
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(74, 79, 106))),
        );

    if app.ui.current_mode() == &AppMode::Diff {
        super::diff::render_diff_overlay(f, area, app);
    } else {
        f.render_widget(table, area);
    }
}

pub fn render_filter(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(f.area())[1];
    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(area)[1];
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
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.filter_selected {
            style = style.fg(Color::Magenta).bg(Color::Rgb(40, 40, 40));
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let block = Block::default()
        .title(Line::from(" Filter by Status ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL);
    let list = List::new(items).block(block);
    f.render_widget(list, inner);
}

pub fn render_confirm_delete(f: &mut Frame, names: &[String]) {
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
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(area)[1];

    f.render_widget(Clear, inner);

    let branch_list = if names.len() > 3 {
        format!("{} branches (including {})", names.len(), names[0])
    } else {
        names.join(", ")
    };

    let block = Block::default()
        .title(Line::from(" ⚠️ UNPUSHED COMMITS DETECTED ⚠️ ").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
        .style(Style::default().bg(Color::Rgb(30, 10, 10)));

    let text = vec![
        Line::from(""),
        Line::from(vec![
            Span::raw("The following branch(es) have "),
            Span::styled(
                "unique commits",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" not found in remote:"),
        ])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(branch_list, Style::default().fg(Color::Cyan)))
            .alignment(Alignment::Center),
        Line::from(""),
        Line::from("Deleting these branches will result in PERMANENT data loss.")
            .alignment(Alignment::Center),
        Line::from(""),
        Line::from(vec![
            Span::raw("Are you absolutely sure? ("),
            Span::styled(
                "y",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("/"),
            Span::styled(
                "n",
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ),
            Span::raw(")"),
        ])
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(
            Span::styled(
                "(Press 'y' to confirm deletion, any other key to cancel)",
                Style::default().fg(Color::DarkGray),
            ),
        )
        .alignment(Alignment::Center),
    ];

    let p = Paragraph::new(text).block(block);
    f.render_widget(p, inner);
}

pub fn render_create_branch(f: &mut Frame, input: &str) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(35),
                Constraint::Percentage(30),
                Constraint::Percentage(35),
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

    let block = Block::default()
        .title(Line::from(" Create New Branch ").alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let p = Paragraph::new(format!(
        "\nName: {}\n\n(Enter to create, Esc to cancel)",
        input
    ))
    .block(block)
    .alignment(Alignment::Center);
    f.render_widget(p, inner);
}

pub fn render_manage(f: &mut Frame, app: &App) {
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
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(area)[1];

    f.render_widget(Clear, inner);

    let options = [
        "1. Checkout",
        "2. View Diff / AI Analysis",
        "3. Delete (Snap)",
        "4. Rename Branch",
        "5. Create Stash from current",
        "6. Help",
        "7. Cancel",
    ];
    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.manage_selected {
            style = style
                .fg(Color::Cyan)
                .bg(Color::Rgb(40, 40, 40))
                .add_modifier(Modifier::BOLD);
        }
        if i == 2 {
            style = style.fg(Color::Red);
            if i == app.ui.manage_selected {
                style = style
                    .bg(Color::Rgb(40, 40, 40))
                    .add_modifier(Modifier::BOLD);
            }
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let b_name = app
        .get_filtered_branches()
        .get(app.ui.selected_branch_idx)
        .map(|b| b.name.as_str())
        .unwrap_or("none");
    let block = Block::default()
        .title(Line::from(format!(" Manage: {} ", b_name)).alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL);
    let list = List::new(items).block(block);
    f.render_widget(list, inner);
}
