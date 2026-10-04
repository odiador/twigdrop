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
    let (table_area, maybe_inspector, maybe_sidebar) = if app.ui.show_inspector_drawer {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(area);
        (chunks[0], Some(chunks[1]), None)
    } else if app.ui.show_nav_sidebar {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)])
            .split(area);
        (chunks[1], None, Some(chunks[0]))
    } else {
        (area, None, None)
    };

    if let Some(sidebar_area) = maybe_sidebar {
        super::sidebar::render_sidebar_menu(f, sidebar_area, app);
    }
    if let Some(inspector_area) = maybe_inspector {
        super::inspector::render_inspector_drawer(f, inspector_area, app);
    }

    let filtered_indices = app.ui.filtered_indices.clone();
    let branches_len = filtered_indices.len();
    let inner_height = table_area.height.saturating_sub(4) as usize;

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
            .push((actual_idx, table_area.y + 3 + i as u16));

        let (icons, color) = get_status_icons(&b.status);
        let (merge_text, merge_color) =
            crate::ui::components::get_merge_status_display(&b.merge_status);

        let current_tag = if is_current { " (HEAD)" } else { "" };
        let is_protected = crate::actions::commands::is_protected_branch(&b.name);
        let is_bulk_selected = app.ui.bulk_selected.contains(&b.name);
        let checkbox = if is_protected {
            "[P]"
        } else if is_bulk_selected {
            "[x]"
        } else {
            "[ ]"
        };

        let sync_text = if b.ahead_count > 0 || b.behind_count > 0 {
            let mut parts = vec![];
            if b.ahead_count > 0 {
                parts.push(format!("↑{}", b.ahead_count));
            }
            if b.behind_count > 0 {
                parts.push(format!("↓{}", b.behind_count));
            }
            parts.join(" ")
        } else if b.status.contains(&BranchStatus::RemoteTracked) {
            "✓ synced".to_string()
        } else {
            "local".to_string()
        };

        let sync_color = if b.ahead_count > 0 {
            Color::Rgb(249, 226, 175)
        } else if b.behind_count > 0 {
            Color::Rgb(137, 220, 235)
        } else if b.status.contains(&BranchStatus::RemoteTracked) {
            Color::Rgb(166, 227, 161)
        } else {
            Color::Rgb(147, 153, 178)
        };

        let branch_name = format!("{}{}", b.name, current_tag);
        let author_str = format!("{} by {}", b.commit_date, b.author);

        let mut row_style = Style::default().fg(Color::Rgb(205, 214, 244));
        let mut branch_style = Style::default().fg(color);
        if is_current {
            branch_style = branch_style.add_modifier(Modifier::BOLD).fg(Color::Rgb(166, 227, 161));
        }

        if selected {
            row_style = row_style.bg(Color::Rgb(49, 50, 68));
        }

        let checkbox_prefix = if selected {
            format!("▎{}", checkbox)
        } else {
            format!(" {}", checkbox)
        };

        let checkbox_style = if is_protected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if is_bulk_selected {
            Style::default().fg(Color::Rgb(166, 227, 161)).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Rgb(124, 128, 156))
        };

        let cells = vec![
            Cell::from(checkbox_prefix).style(checkbox_style),
            Cell::from(Line::from(vec![
                Span::styled(format!("{:<4} ", icons), Style::default().fg(color)),
                Span::styled(branch_name, branch_style),
            ])),
            Cell::from(sync_text).style(Style::default().fg(sync_color)),
            Cell::from(merge_text).style(Style::default().fg(merge_color)),
            Cell::from(b.age.clone()).style(Style::default().fg(Color::Rgb(249, 226, 175))),
            Cell::from(author_str).style(Style::default().fg(Color::Rgb(166, 173, 200))),
        ];

        rows.push(Row::new(cells).style(row_style));
    }

    let widths = [
        Constraint::Length(5),
        Constraint::Percentage(30),
        Constraint::Length(12),
        Constraint::Length(16),
        Constraint::Length(12),
        Constraint::Percentage(30),
    ];

    let sort_hint = match app.ui.branch_sort_mode {
        crate::state::ui::BranchSortMode::Recent => "Recent ▾ (Ctrl+S)",
        crate::state::ui::BranchSortMode::PrunableFirst => "Prunable ▾ (Ctrl+S)",
        crate::state::ui::BranchSortMode::Alphabetical => "A-Z ▾ (Ctrl+S)",
    };

    let table = Table::new(rows, widths)
        .header(
            Row::new(vec![
                "",
                "Branch",
                "Sync",
                "Merge Health",
                "Age",
                "Last Commit",
            ])
            .style(
                Style::default()
                    .fg(Color::Rgb(147, 153, 178))
                    .add_modifier(Modifier::BOLD),
            )
            .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(if !app.ui.search_query.is_empty() {
                    Line::from(vec![
                        Span::styled(" Branches ", Style::default().fg(Color::Rgb(203, 166, 247)).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("[{}/{}] ", branch_items_to_show, branches_len), Style::default().fg(Color::Rgb(147, 153, 178))),
                        Span::styled(" ", Style::default().fg(Color::Rgb(249, 226, 175)).add_modifier(Modifier::BOLD)),
                        Span::styled(&app.ui.search_query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled("▌ ", Style::default().fg(Color::Rgb(203, 166, 247))),
                        Span::styled("(Esc to clear) • ", Style::default().fg(Color::Rgb(147, 153, 178))),
                        Span::styled(format!("Sort: {} ", sort_hint), Style::default().fg(Color::Rgb(147, 153, 178))),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(" Branches ", Style::default().fg(Color::Rgb(203, 166, 247)).add_modifier(Modifier::BOLD)),
                        Span::styled(format!("[{}/{}] • ", branch_items_to_show, branches_len), Style::default().fg(Color::Rgb(147, 153, 178))),
                        Span::styled("Type to filter • ", Style::default().fg(Color::Rgb(108, 112, 134))),
                        Span::styled(format!("Sort: {} • ", sort_hint), Style::default().fg(Color::Rgb(147, 153, 178))),
                        Span::styled("[→] Inspector • [←] Sidebar ", Style::default().fg(Color::Rgb(147, 153, 178))),
                    ])
                })
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(74, 79, 106))),
        );

    if app.ui.current_mode() == &AppMode::Diff {
        super::diff::render_diff_overlay(f, table_area, app);
    } else {
        f.render_widget(table, table_area);
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
    let full_area = f.area();
    // 1. Dim background
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area);

    // 2. Centered Spotlight floating window (52% width, 48% height)
    let inner = crate::ui::components::centered_rect(52, 48, full_area);
    f.render_widget(Clear, inner);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header banner
            Constraint::Min(6),    // Action list
            Constraint::Length(1), // Footer hint bar
        ])
        .split(inner);

    let b_name = app
        .get_filtered_branches()
        .get(app.ui.selected_branch_idx)
        .map(|b| b.name.as_str())
        .unwrap_or("none");

    let is_protected = crate::actions::commands::is_protected_branch(b_name);

    let header_block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                " ⚡ MANAGE BRANCH ",
                Style::default()
                    .fg(Color::Rgb(203, 166, 247))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({}) ", b_name),
                Style::default().fg(Color::Rgb(147, 153, 178)),
            ),
            if is_protected {
                Span::styled(
                    "[PROTECTED] ",
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
            } else {
                Span::raw("")
            },
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(203, 166, 247)))
        .style(Style::default().bg(Color::Rgb(24, 24, 37)));

    let header_text = Line::from(vec![
        Span::styled("  Target: ", Style::default().fg(Color::Rgb(147, 153, 178))),
        Span::styled(
            b_name,
            Style::default()
                .fg(Color::Rgb(137, 220, 235))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " • Select an action to execute",
            Style::default().fg(Color::Rgb(108, 112, 134)),
        ),
    ]);
    f.render_widget(Paragraph::new(header_text).block(header_block), chunks[0]);

    let list_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(Color::Rgb(49, 50, 68)))
        .style(Style::default().bg(Color::Rgb(30, 30, 46)));

    let actions = [
        (
            "CHECKOUT",
            "Switch to this branch (git checkout)",
            Color::Rgb(166, 227, 161),
        ),
        (
            "DIFF / AI",
            "View diff and run AI conflict analysis",
            Color::Rgb(137, 180, 250),
        ),
        (
            "DELETE",
            "Delete branch (Snap delete)",
            Color::Rgb(243, 139, 168),
        ),
        ("RENAME", "Rename branch", Color::Rgb(249, 226, 175)),
        (
            "STASH",
            "Create stash from current branch",
            Color::Rgb(245, 194, 231),
        ),
        ("HELP", "Help & Legend", Color::Rgb(147, 153, 178)),
        ("CANCEL", "Close this menu", Color::Rgb(108, 112, 134)),
    ];

    let mut items = vec![];
    for (i, (tag, desc, tag_color)) in actions.iter().enumerate() {
        let is_selected = i == app.ui.manage_selected;
        let row_style = if is_selected {
            Style::default().bg(Color::Rgb(49, 50, 68)).fg(Color::White)
        } else {
            Style::default().fg(Color::Rgb(205, 214, 244))
        };

        let prefix = if is_selected { "▎ " } else { "  " };
        let tag_span = Span::styled(
            format!("[{}] ", tag),
            Style::default().fg(*tag_color).add_modifier(Modifier::BOLD),
        );
        let desc_span = Span::styled(
            *desc,
            if is_selected {
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::Rgb(205, 214, 244))
            },
        );

        let item_line = Line::from(vec![
            Span::styled(
                prefix,
                Style::default()
                    .fg(Color::Rgb(203, 166, 247))
                    .add_modifier(Modifier::BOLD),
            ),
            tag_span,
            desc_span,
        ]);
        items.push(ListItem::new(item_line).style(row_style));
    }

    f.render_widget(List::new(items).block(list_block), chunks[1]);

    let footer_hints = Line::from(vec![
        Span::styled(
            "  ↑↓",
            Style::default()
                .fg(Color::Rgb(180, 190, 254))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " Select   ",
            Style::default().fg(Color::Rgb(147, 153, 178)),
        ),
        Span::styled(
            "↵",
            Style::default()
                .fg(Color::Rgb(166, 227, 161))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            " Execute   ",
            Style::default().fg(Color::Rgb(147, 153, 178)),
        ),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Rgb(243, 139, 168))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Cancel", Style::default().fg(Color::Rgb(147, 153, 178))),
    ]);
    f.render_widget(
        Paragraph::new(footer_hints).style(Style::default().bg(Color::Rgb(24, 24, 37))),
        chunks[2],
    );
}
