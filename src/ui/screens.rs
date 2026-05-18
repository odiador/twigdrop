use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::app::{App, AppMode, PrimaryMode};
use crate::git::files::FileStatus;
use crate::models::BranchStatus;
use crate::ui::components::get_status_icons;

pub fn render_main_list(f: &mut Frame, area: Rect, app: &mut App) {
    let branches_len = app.branch_state.filtered_indices.len();
    let inner_height = area.height.saturating_sub(4) as usize; // Extra space for header/table

    if inner_height == 0 {
        return;
    }

    let mut start = 0;
    if branches_len > inner_height {
        let half_height = inner_height / 2;
        if app.branch_state.selected > half_height {
            start = app.branch_state.selected - half_height;
        }
        let mut end = start + inner_height;
        if end > branches_len {
            end = branches_len;
            start = end.saturating_sub(inner_height);
        }
    }
    app.branch_state.list_start_index = start;
    let filtered_branches = app.get_filtered_branches();

    let mut rows: Vec<Row> = vec![];

    let branch_items_to_show = inner_height.min(branches_len.saturating_sub(start));

    for i in 0..branch_items_to_show {
        let branch_idx = start + i;
        if branch_idx >= branches_len {
            break;
        }

        let b = filtered_branches[branch_idx];
        let selected = branch_idx == app.branch_state.selected;
        let is_current = b.name == app.current_branch;

        let (icons, color) = get_status_icons(&b.status);
        let (merge_text, merge_color) =
            crate::ui::components::get_merge_status_display(&b.merge_status);

        let current_tag = if is_current { " (current)" } else { "" };

        let is_bulk_selected = app.branch_state.bulk_selected.contains(&b.name);
        let checkbox = if is_bulk_selected { "[x]" } else { "[ ]" };

        let branch_name = format!("{}{}", b.name, current_tag);
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

        let mut row_style = Style::default().fg(Color::Rgb(205, 214, 244)); // Cool Grays
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

    let filter_text = if let Some(f) = &app.branch_state.current_filter {
        format!("sort: {:?}", f)
    } else {
        "sort: None".to_string()
    };

    let title_line = Line::from(vec![
        Span::styled(
            " 🧹 twigdrop ".to_string(),
            Style::default()
                .fg(Color::Rgb(180, 190, 254))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(
                "│ {} · {} branches · {} ",
                app.current_branch, branches_len, filter_text
            ),
            Style::default().fg(Color::Rgb(124, 128, 156)),
        ),
    ]);

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
                "Last Commit Author",
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
                .title(title_line)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(74, 79, 106))),
        );

    if app.mode == AppMode::Diff {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(area);
        f.render_widget(table, chunks[0]);

        let mut info_text = app.branch_state.branch_info.clone();
        if let Some(ai) = &app.ai_state.ai_analysis {
            info_text = format!(
                "--- AI ANALYSIS ---\n\n{}\n\n------------------\n\n{}",
                ai, info_text
            );
        }

        let diff = Paragraph::new(info_text)
            .block(
                Block::default()
                    .title(" Intelligence & Diff ")
                    .borders(Borders::ALL),
            )
            .scroll((app.branch_state.info_scroll, 0));
        f.render_widget(diff, chunks[1]);
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
        if i == app.branch_state.filter_selected {
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

pub fn render_help_content(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(Line::from(" Help & Legend ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    let mut help_text = vec![
        "Twigdrop helps you clean up your local branches safely.",
        "",
    ];

    if app.primary_mode == PrimaryMode::Branches {
        help_text.extend(vec![
            "Status Icons (Branches):",
            "  ▲ (Red)     : Has Unique Commits (DANGER: Not in remote!)",
            "  ⨯ (Gray)    : Gone (Upstream branch was deleted)",
            "  ↑ (Yellow)  : Ahead of upstream (Local has new commits)",
            "  ↓ (Cyan)    : Behind upstream (Remote has new commits)",
            "  ✓ (Green)   : Merged (Safe to delete)",
            "  L (Lavender) : Local Only (No tracking branch)",
            "  S (Green)   : Stashed changes exist for this branch",
            "",
            "Shortcuts (Branches):",
            "  ↑/k, ↓/j    : Navigate list",
            "  Space       : Toggle branch selection (for bulk delete)",
            "  D (Shift+D) : Bulk delete selected branches",
            "  p           : Prune 'Gone' branches (Safe only)",
            "  i           : AI Intelligence Analysis for branch",
            "  f           : Open Filters",
            "  m / Enter   : Manage selected branch (Checkout, Diff, Delete)",
        ]);
    } else {
        help_text.extend(vec![
            "Status Colors (Files):",
            "  Yellow      : Modified",
            "  Green       : Added / New",
            "  Blue        : Staged (in index)",
            "  Pink        : Untracked",
            "  Gray        : Ignored (.gitignore / .twigignore)",
            "",
            "Shortcuts (Files):",
            "  ↑/k, ↓/j    : Navigate tree",
            "  → / Enter   : Open folder / Move into children",
            "  ←           : Close folder / Move to parent",
            "  v           : Open in IDE (Root by default, Path with Alt)",
            "  t           : Internal TTY (Alt+t for External)",
            "  a           : Alt IDE (Root by default, Path with Alt)",
        ]);
    }

    help_text.extend(vec![
        "",
        "Global Shortcuts:",
        "  d           : Switch between Branches and Files mode",
        "  Shift+Tab   : Open Settings Panel",
        "  S (Shift+S) : Open Stash Manager",
        "  Ctrl+o      : Open IDE (Root by default, Path with Alt)",
        "  q / Esc     : Quit / Back",
    ]);

    let p = Paragraph::new(help_text.join("\n"))
        .block(block)
        .alignment(Alignment::Left);

    let help_inner = Layout::default()
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

    f.render_widget(p, help_inner);
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
        "2. View Diff",
        "3. Delete (Individual)",
        "4. Help",
        "5. Cancel",
    ];
    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.branch_state.manage_selected {
            style = style.fg(Color::Cyan).bg(Color::Rgb(40, 40, 40));
        }
        if i == 2 {
            // Delete option
            style = style.fg(Color::Red);
            if i == app.branch_state.manage_selected {
                style = style.bg(Color::Rgb(40, 40, 40));
            }
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let b_name = app
        .get_filtered_branches()
        .get(app.branch_state.selected)
        .map(|b| b.name.as_str())
        .unwrap_or("none");
    let block = Block::default()
        .title(Line::from(format!(" Manage: {} ", b_name)).alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL);
    let list = List::new(items).block(block);
    f.render_widget(list, inner);
}

pub fn render_message(f: &mut Frame, msg: &str) {
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

    let block = Block::default()
        .title(Line::from(" Git Response ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)));

    let p = Paragraph::new(msg)
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(p, inner);
}

pub fn render_directory_searcher(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(
            Line::from(" 📂 Files (d: branches, v: IDE, t: TTY, h: help) ")
                .alignment(Alignment::Left),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(74, 79, 106)));

    let mut items = vec![];
    for (i, entry) in app.file_state.file_tree.iter().enumerate() {
        let indent = "  ".repeat(entry.depth);
        let icon = if entry.is_dir {
            if entry.is_open {
                "▼ 📂 "
            } else {
                "▶ 📁 "
            }
        } else {
            "  📄 "
        };

        let name = entry.path.file_name().unwrap_or_default().to_string_lossy();
        let status_color = match entry.status {
            FileStatus::Modified => Color::Rgb(249, 226, 175), // Yellow
            FileStatus::Added => Color::Rgb(161, 229, 193),    // Green
            FileStatus::Staged => Color::Rgb(137, 180, 250),   // Blue
            FileStatus::Untracked => Color::Rgb(245, 194, 231), // Pink/Red
            FileStatus::Ignored => Color::Rgb(140, 143, 161),  // Gray
            FileStatus::Deleted => Color::Rgb(243, 139, 168),  // Red
            FileStatus::Normal => Color::Rgb(205, 214, 244),   // Gray/White
        };

        let mut style = Style::default().fg(status_color);
        if i == app.file_state.file_selected {
            style = style.bg(Color::White).fg(Color::Black);
        }

        let line = Line::from(vec![
            Span::raw(indent),
            Span::raw(icon),
            Span::styled(name.to_string(), style),
        ]);
        items.push(ListItem::new(line));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, area);
}

pub fn render_stash_detail(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(area);

    // Left: Stash list
    let mut stash_items = vec![];
    for (i, stash) in app.stash_state.stashes.iter().enumerate() {
        let mut style = Style::default().fg(Color::Rgb(205, 214, 244));
        if i == app.stash_state.stash_selected {
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

    // Right: Detail
    let detail_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(chunks[1]);

    let files_text = app.stash_state.stash_files.join("\n");
    let files_p = Paragraph::new(files_text).block(
        Block::default()
            .title(" Files in Stash ")
            .borders(Borders::ALL),
    );
    f.render_widget(files_p, detail_chunks[0]);

    let diff_p = Paragraph::new(app.stash_state.stash_diff.as_str())
        .block(
            Block::default()
                .title(" Diff Preview ")
                .borders(Borders::ALL),
        )
        .scroll((app.branch_state.info_scroll, 0));
    f.render_widget(diff_p, detail_chunks[1]);
}

pub fn render_settings(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ]
            .as_ref(),
        )
        .split(area)[1];

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(Line::from(" [ Twigdrop Settings ] ").alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let options = [
        format!("Primary IDE: {}", app.config.ide_command),
        format!("Alternative IDE: {}", app.config.alternative_ide_command),
        "Save and Exit".to_string(),
    ];

    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.settings_state.selected {
            style = style
                .bg(Color::White)
                .fg(Color::Black)
                .add_modifier(Modifier::BOLD);
        }

        let text = if i == app.settings_state.selected && app.settings_state.editing {
            format!("> {}", app.settings_state.input)
        } else {
            opt.clone()
        };

        items.push(ListItem::new(text).style(style));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, inner);

    // Help text at bottom of settings
    let help = Paragraph::new("↑/↓: navigate │ Enter: edit/select │ Esc: cancel")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));

    let help_area = Rect::new(inner.x, inner.y + inner.height - 2, inner.width, 1);
    f.render_widget(help, help_area);
}
