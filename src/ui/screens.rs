use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Cell, Clear, List, ListItem, Paragraph, Row, Table},
};

use crate::app::App;
use crate::git::files::FileStatus;
use crate::models::{BranchStatus, GutterStatus};
use crate::state::ui::{AppMode, DatePickerField, DatePickerState, FilePanel, PreviewState, PrimaryMode, RebaseAction};
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
        let overlay_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(5),
                    Constraint::Percentage(90),
                    Constraint::Percentage(5),
                ]
                .as_ref(),
            )
            .split(area)[1];

        let inner_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(5),
                    Constraint::Percentage(90),
                    Constraint::Percentage(5),
                ]
                .as_ref(),
            )
            .split(overlay_area)[1];

        f.render_widget(Clear, inner_area);

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
            .split(inner_area);

        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
            .split(main_chunks[0]);

        // 1. Files List
        let mut file_items = vec![];
        for (i, file) in app.repo.diff_files.iter().enumerate() {
            let mut style = Style::default().fg(Color::Gray);
            if i == app.ui.diff_file_selected {
                let bg = if app.ui.diff_panel == FilePanel::Directory {
                    Color::White
                } else {
                    Color::Rgb(45, 45, 65)
                };
                let fg = if app.ui.diff_panel == FilePanel::Directory {
                    Color::Black
                } else {
                    Color::White
                };
                style = style.bg(bg).fg(fg).add_modifier(Modifier::BOLD);
            }
            file_items.push(ListItem::new(file.clone()).style(style));
        }
        let list_title = format!(" Changed Files ({}) ", app.repo.diff_files.len());
        let files_list = List::new(file_items).block(
            Block::default()
                .title(list_title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        );
        f.render_widget(files_list, left_chunks[0]);

        // 2. AI Analysis separate
        let mut info_text = "No AI analysis yet. Press 'i' to analyze.".to_string();
        if let Some(ai) = &app.ai_state.ai_analysis {
            info_text = ai.clone();
        }
        let ai_block = Block::default()
            .title(" AI Intelligence ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Magenta));
        let ai_p = Paragraph::new(info_text)
            .block(ai_block)
            .wrap(ratatui::widgets::Wrap { trim: true });
        f.render_widget(ai_p, left_chunks[1]);

        // 3. File Preview (The Diff)
        if let Some(state) = &app.repo.diff_preview {
            let border_color = if app.ui.diff_panel == FilePanel::Preview {
                Color::Green
            } else {
                Color::Rgb(74, 79, 106)
            };
            let mut final_lines = Vec::new();
            let visible_rows = main_chunks[1].height.saturating_sub(2) as usize;
            let start_idx = state.scroll_y;
            let end_idx = (start_idx + visible_rows + 5).min(state.highlighted_lines.len());

            for i in start_idx..end_idx {
                let mut line_style = Style::default();
                if i == state.cursor_y && app.ui.diff_panel == FilePanel::Preview {
                    line_style = line_style.bg(Color::Rgb(60, 60, 80));
                }
                final_lines.push(state.highlighted_lines[i].clone().style(line_style));
            }

            let preview_title = format!(" Diff: {} (Tab: switch) ", state.file_path);
            let diff_preview = Paragraph::new(Text::from(final_lines)).block(
                Block::default()
                    .title(preview_title)
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(border_color)),
            );
            f.render_widget(diff_preview, main_chunks[1]);
        } else {
            let fallback_diff = Paragraph::new("Select a file to see diff").block(
                Block::default()
                    .title(" Diff Preview ")
                    .borders(Borders::ALL),
            );
            f.render_widget(fallback_diff, main_chunks[1]);
        }
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

pub const ASCII_LOGO: &str = r#"
████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
   ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
   ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
   ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

pub fn render_help_content(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(8), // Logo
                Constraint::Min(10),   // Content
                Constraint::Length(1), // Footer
            ]
            .as_ref(),
        )
        .split(area);

    let logo = Paragraph::new(ASCII_LOGO.trim_matches('\n'))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(logo, chunks[0]);

    let block = Block::default()
        .title(Line::from(" Help & Legend ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    let mut help_text = vec![
        "Twigdrop helps you clean up your local branches safely.",
        "",
    ];

    if app.ui.primary_mode == PrimaryMode::Branches {
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
            "  ↑/k, ↓/j       : Navigate list",
            "  /              : Fuzzy search branches",
            "  Space          : Toggle branch selection (for bulk delete)",
            "  Shift+D        : Bulk delete selected branches",
            "  p              : Prune 'Gone' branches (Safe only)",
            "  i              : AI Intelligence Analysis for branch",
            "  c              : Create new branch",
            "  f              : Open Filters",
            "  m / Enter      : Manage selected branch (Checkout, Diff, Delete)",
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
            "  Enter       : Preview file content",
            "  e           : Open folder in Explorer (Alt+e for selected path)",
            "  s           : Stage / Unstage file",
            "  v           : Open in IDE (Root by default, Path with Alt)",
            "  t           : Internal TTY (Alt+t for External)",
            "  a           : Alt IDE (Root by default, Path with Alt)",
            "  [ / ]       : Expand / Contract sidebar width",
            "  Tab         : Switch focus between sidebar and preview",
        ]);
    }

    help_text.extend(vec![
        "",
        "Global Shortcuts:",
        "  d              : Switch between Branches and Files mode",
        "  Shift+Tab      : Open App Switcher",
        "  Esc            : Open Main Menu (Settings, Help, Quit)",
        "  !              : Open Shell Command Prompt",
        "  :              : Open Git Quick Actions Palette",
        "  ? / h          : Help & Legend",
        "  Shift+S        : Open Stash Manager",
        "  Shift+C        : Open Unpushed Commits Manager",
        "  Shift+F        : AI Auto-Fix (in Diff mode with conflicts)",
        "  Shift+R        : Interactive Rebase",
        "  Ctrl+o         : Open IDE (Root by default, Path with Alt)",
        "  q              : Quit",
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
        .split(chunks[1])[1];

    f.render_widget(p, help_inner);

    let footer_text = vec![Line::from(vec![
        Span::raw("Made by: "),
        Span::styled(
            "odiador",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ❤️ for the community"),
    ])];
    let footer_p = Paragraph::new(footer_text).alignment(Alignment::Center);
    f.render_widget(footer_p, chunks[2]);
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
    ];

    let p = Paragraph::new(text)
        .block(block)
        .alignment(Alignment::Center);
    f.render_widget(p, inner);
}

pub fn render_commits(f: &mut Frame, area: Rect, app: &App) {
    let mut items = vec![];
    if app.repo.commit_tree.is_empty() {
        items.push(
            ListItem::new(" No commits found in history. Ensure this is a git repository with commits. ")
                .style(Style::default().fg(Color::Yellow)),
        );
    } else {
        for (i, commit) in app.repo.commit_tree.iter().enumerate() {
            let is_selected = i == app.ui.selected_commit_idx;
            let mut graph_spans = Vec::new();

            for ch in commit.graph.chars() {
                let color = match ch {
                    '*' => Color::Magenta,
                    '|' | '/' | '\\' | '_' => Color::Rgb(100, 100, 120), // Darker gray for lines
                    _ => Color::Gray,
                };
                graph_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }

            let mut commit_spans = vec![
                Span::styled(format!(" {} ", commit.hash), Style::default().fg(Color::Yellow)),
                Span::styled(format!(" {} ", commit.date), Style::default().fg(Color::Cyan)),
                Span::styled(
                    format!(" [{}] ", commit.author),
                    Style::default().fg(Color::Rgb(180, 180, 200)),
                ),
                Span::styled(commit.message.clone(), Style::default().fg(Color::White)),
            ];

            let mut all_spans = graph_spans;
            all_spans.append(&mut commit_spans);

            let mut line_style = Style::default();
            if is_selected {
                line_style = line_style.bg(Color::Rgb(45, 45, 65));
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
    f.render_widget(list, area);
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
        "1. Change Date (git commit --amend --date=...)",
        "2. Amend Staged Files (git commit --fixup & rebase)",
    ];

    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.settings_state.selected {
            style = style
                .bg(Color::Rgb(45, 45, 65))
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

pub fn render_shell(f: &mut Frame, input: &str) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(area)[1];

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
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(area)[1];

    f.render_widget(Clear, inner);

    let mut items = vec![];
    for (i, action) in app.ui.quick_actions_state.actions.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.quick_actions_state.selected {
            style = style
                .bg(Color::Rgb(45, 45, 65))
                .fg(Color::White)
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
    let (sidebar_area, preview_area) = if let AppMode::CodePreview(state) = app.ui.current_mode() {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Percentage(app.ui.sidebar_width),
                    Constraint::Min(0),
                ]
                .as_ref(),
            )
            .split(area);
        (chunks[0], Some((chunks[1], state)))
    } else {
        (area, None)
    };

    let sidebar_border_color = if app.ui.active_panel == FilePanel::Directory
        && matches!(app.ui.current_mode(), AppMode::CodePreview(_))
    {
        Color::Rgb(180, 190, 254)
    } else {
        Color::Rgb(74, 79, 106)
    };

    let block = Block::default()
        .title(Line::from(" 📂 Files (Tab: switch focus) ").alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(sidebar_border_color));

    let mut items = vec![];
    for (i, entry) in app.repo.file_tree.iter().enumerate() {
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
            FileStatus::Modified => Color::Rgb(249, 226, 175),
            FileStatus::Added => Color::Rgb(161, 229, 193),
            FileStatus::Staged => Color::Rgb(137, 180, 250),
            FileStatus::Untracked => Color::Rgb(245, 194, 231),
            FileStatus::Ignored => Color::Rgb(140, 143, 161),
            FileStatus::Deleted => Color::Rgb(243, 139, 168),
            FileStatus::Conflict => Color::Rgb(210, 15, 57),
            FileStatus::Normal => Color::Rgb(205, 214, 244),
        };

        let mut style = Style::default().fg(status_color);

        let status_bg = match entry.status {
            FileStatus::Modified => Some(Color::Rgb(35, 48, 65)),
            FileStatus::Added => Some(Color::Rgb(35, 60, 48)),
            FileStatus::Conflict => Some(Color::Rgb(65, 35, 35)),
            FileStatus::Staged => Some(Color::Rgb(45, 60, 75)),
            _ => None,
        };

        if let Some(bg) = status_bg {
            style = style.bg(bg);
        }

        if i == app.ui.selected_file_idx {
            let bg = if app.ui.active_panel == FilePanel::Directory {
                Color::White
            } else {
                Color::Rgb(54, 58, 79)
            };
            let fg = if app.ui.active_panel == FilePanel::Directory {
                Color::Black
            } else {
                status_color
            };
            style = style.bg(bg).fg(fg);
        }
        items.push(
            ListItem::new(Line::from(vec![
                Span::raw(indent),
                Span::raw(icon),
                Span::styled(name.to_string(), style),
            ]))
            .style(style),
        );
    }

    let list = List::new(items).block(block);
    f.render_widget(list, sidebar_area);

    if let Some((pa, state)) = preview_area {
        render_code_preview(f, app, pa, state);
    }
}

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

pub fn render_settings(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
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

    let options = [
        format!(
            "[ Editor ] Primary IDE          : {}",
            app.config.ide_command
        ),
        format!(
            "[ Editor ] Alternative IDE      : {}",
            app.config.alternative_ide_command
        ),
        format!(
            "[ AI ]     AI Provider          : {}",
            app.config.ai_provider
        ),
        format!(
            "[ AI ]     AI Model             : {}",
            app.config.current_provider().model
        ),
        format!(
            "[ AI ]     OpenAI API Key       : {}",
            if app.config.current_provider().api_key.is_empty() {
                "None".to_string()
            } else {
                "****".to_string()
            }
        ),
        format!(
            "[ AI ]     Ollama URL           : {}",
            app.config.current_provider().url
        ),
        format!(
            "[ UI ]     Enable Animations    : {}",
            app.config.enable_animations
        ),
        format!(
            "[ UI ]     Default Sidebar Width: {}",
            app.config.default_sidebar_width
        ),
        "           [ Save and Exit ]".to_string(),
    ];
    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.settings_state.selected {
            style = style
                .bg(Color::Rgb(45, 45, 65))
                .add_modifier(Modifier::BOLD);
        }

        let text = if i == app.ui.settings_state.selected {
            if app.ui.settings_state.editing {
                if i == 4 {
                    format!("> {}", "*".repeat(app.ui.settings_state.input.len()))
                } else {
                    format!("> {}", app.ui.settings_state.input)
                }
            } else if app.ui.settings_state.selecting {
                format!("{} (Selecting...)", opt)
            } else {
                opt.clone()
            }
        } else {
            opt.clone()
        };

        if i == app.ui.settings_state.selected {
            style = style.fg(Color::White);
        }

        items.push(ListItem::new(text).style(style));
    }

    f.render_widget(
        List::new(items).block(
            Block::default()
                .title(Line::from(" [ Twigdrop Settings ] ").alignment(Alignment::Center))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        inner,
    );

    if app.ui.settings_state.selecting {
        let menu_width = 40;
        let menu_height = app.ui.settings_state.choices.len().min(10) as u16 + 2;

        let center_x = inner.x + (inner.width / 2);
        let center_y = inner.y + (inner.height / 2);

        let menu_area = Rect::new(
            center_x.saturating_sub(menu_width / 2),
            center_y.saturating_sub(menu_height / 2),
            menu_width,
            menu_height,
        );
        f.render_widget(Clear, menu_area);

        let mut choice_items = vec![];
        for (i, choice) in app.ui.settings_state.choices.iter().enumerate() {
            let mut style = Style::default().fg(Color::Gray);
            if i == app.ui.settings_state.choice_idx {
                style = style
                    .bg(Color::Rgb(80, 80, 100))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
            }
            choice_items.push(ListItem::new(format!(" {} ", choice)).style(style));
        }
        let list = List::new(choice_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Select Option "),
        );
        f.render_widget(list, menu_area);
    }

    let footer_msg = if app.ui.settings_state.selecting {
        "↑/↓: cycle options │ Enter: select │ Esc: cancel"
    } else if app.ui.settings_state.editing {
        "Type your value │ Enter: save │ Esc: cancel"
    } else {
        "↑/↓: navigate │ Enter: edit/select │ Esc: cancel"
    };

    let help_area = Rect::new(inner.x, inner.y + inner.height - 2, inner.width, 1);
    f.render_widget(
        Paragraph::new(footer_msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        help_area,
    );
}

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

pub fn render_code_preview(f: &mut Frame, app: &App, area: Rect, state: &PreviewState) {
    let border_color = if app.ui.active_panel == FilePanel::Preview {
        Color::Rgb(180, 190, 254)
    } else {
        Color::Rgb(74, 79, 106)
    };
    let block = Block::default()
        .title(format!(" Preview: {} (Tab: switch) ", state.file_path))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let mut final_lines = Vec::new();
    let visible_rows = area.height.saturating_sub(2) as usize;
    let start_idx = state.scroll_y;
    const OVERSCAN_LINES: usize = 5;
    let end_idx = (start_idx + visible_rows + OVERSCAN_LINES).min(state.highlighted_lines.len());

    for i in start_idx..end_idx {
        let h_line = &state.highlighted_lines[i];
        let mut spans = vec![Span::styled(
            format!("{:>3} ", i + 1),
            Style::default().fg(Color::DarkGray),
        )];
        spans.push(match state.line_diffs.get(&i) {
            Some(GutterStatus::Added) => Span::styled("+ ", Style::default().fg(Color::Green)),
            Some(GutterStatus::Modified) => Span::styled("| ", Style::default().fg(Color::Blue)),
            Some(GutterStatus::Deleted) => Span::styled("~ ", Style::default().fg(Color::Red)),
            None => Span::raw("  "),
        });

        let is_selected =
            if let (Some(start), Some(end)) = (state.selection_start, state.selection_end) {
                let (s, e) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };
                i >= s && i <= e
            } else {
                false
            };
        let is_cursor = i == state.cursor_y;
        let mut line_style = Style::default();
        if is_cursor && app.ui.active_panel == FilePanel::Preview {
            line_style = line_style.bg(Color::Rgb(255, 255, 0)).fg(Color::Black);
        } else if is_cursor {
            line_style = line_style.bg(Color::Rgb(40, 40, 60));
        } else if is_selected {
            line_style = line_style.bg(Color::Rgb(30, 50, 80));
        }

        for span in &h_line.spans {
            let mut s = span.style;
            if is_cursor && app.ui.active_panel == FilePanel::Preview {
                s = s.bg(Color::Rgb(255, 255, 0)).fg(Color::Black);
            } else if is_cursor {
                s = s.bg(Color::Rgb(40, 40, 60));
            } else if is_selected {
                s = s.bg(Color::Rgb(30, 50, 80));
            }
            spans.push(Span::styled(span.content.clone(), s));
        }
        final_lines.push(Line::from(spans).style(line_style));
    }
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(Text::from(final_lines)).block(block), area);
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

fn render_transparent_ascii(
    buf: &mut ratatui::buffer::Buffer,
    ascii: &str,
    area: Rect,
    color: Color,
) {
    let lines: Vec<&str> = ascii.trim_matches('\n').lines().collect();
    if lines.is_empty() {
        return;
    }

    let max_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    let height = lines.len() as u16;

    let start_x = area.x + area.width.saturating_sub(max_width) / 2;
    let start_y = area.y + area.height.saturating_sub(height) / 2;

    for (row_idx, line) in lines.iter().enumerate() {
        let y = start_y + row_idx as u16;
        if y >= area.bottom() {
            break;
        }

        for (x, ch) in (start_x..).zip(line.chars()) {
            if x >= area.right() {
                break;
            }
            if ch != ' ' {
                buf[(x, y)].set_char(ch).set_fg(color);
            }
        }
    }
}

pub fn render_main_menu(f: &mut Frame, app: &App) {
    let area = f.area();
    let buf = f.buffer_mut();

    let ascii_logo = r#"
 ████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
 ╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
    ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
    ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
    ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
    ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

    let opt_options = r#"
  ___  ____ _____ ___ ___  _  _ ___ 
 / _ \|  _ \_   _|_ _/ _ \| \| / __|
| (_) | |_) || |  | | (_) | .` \__ \
 \___/| .__/ |_| |___\___/|_|\_|___/
      |_|                           
"#;

    let opt_help = r#"
 _  _ ___ _    ___ 
| || | __| |  | _ \
| __ | _|| |__|  _/
|_||_|___|____|_|  
"#;

    let opt_quit = r#"
  ___  _   _ ___ _____ 
 / _ \| | | |_ _|_   _|
| (_) | |_| || |  | |  
 \__\_\\___/|___| |_|  
"#;

    let total_height = 8 + 2 + 6 + 5 + 5;
    let v_margin = area.height.saturating_sub(total_height) / 2;

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(v_margin),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    render_transparent_ascii(buf, ascii_logo, v_chunks[1], Color::Cyan);

    let options_ascii = [opt_options, opt_help, opt_quit];
    for (i, opt) in options_ascii.iter().enumerate() {
        let is_selected = i == app.ui.main_menu_state.selected;
        let color = if is_selected {
            Color::White
        } else {
            Color::Rgb(60, 60, 80)
        };

        render_transparent_ascii(buf, opt, v_chunks[3 + i], color);
    }
}

pub fn render_switcher(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(f.area());

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area[1])[1];

    f.render_widget(Clear, inner);

    let mut items = Vec::new();
    for (i, mode) in app.ui.mode_history.iter().enumerate() {
        let text = format_mode(mode);
        let mut style = Style::default().fg(Color::Gray);

        if i == app.ui.switcher_index {
            style = style
                .bg(Color::Rgb(45, 45, 65))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);
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

pub fn format_mode(mode: &AppMode) -> String {
    match mode {
        AppMode::BranchesView => "Branches".to_string(),
        AppMode::FilesView => "Files".to_string(),
        AppMode::CommitsView => "Commit Tree".to_string(),
        AppMode::Normal => "Main Views".to_string(),
        AppMode::Help => "Help".to_string(),
        AppMode::Manage => "Manage Branch".to_string(),
        AppMode::Filter => "Filters".to_string(),
        AppMode::StashDetail => "Stash Details".to_string(),
        AppMode::Settings => "Settings".to_string(),
        AppMode::Search => "Search".to_string(),
        AppMode::Diff => "Diff".to_string(),
        AppMode::CodePreview(state) => format!("Preview: {}", state.file_path),
        AppMode::ConfirmDelete(_) => "Confirm Delete".to_string(),
        AppMode::CreateBranch(_) => "Create Branch".to_string(),
        AppMode::CommitAction(_) => "Commit Actions".to_string(),
        AppMode::InteractiveRebase => "Interactive Rebase".to_string(),
        AppMode::Shell(_) => "Shell".to_string(),
        AppMode::QuickActions => "Quick Actions".to_string(),
        AppMode::MainMenu => "Main Menu".to_string(),
        AppMode::Message(_) => "Message".to_string(),
        AppMode::Switcher => "App Switcher".to_string(),
        AppMode::DatePicker(_) => "Date Picker".to_string(),
    }
}

pub fn render_date_picker(f: &mut Frame, _app: &App, state: &DatePickerState) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
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

    let block = Block::default()
        .title(" Select Date & Time (Enter to Save) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    
    f.render_widget(block, inner);

    let picker_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
        .margin(1)
        .split(inner);

    let fields = [
        ("Year", DatePickerField::Year),
        ("Month", DatePickerField::Month),
        ("Day", DatePickerField::Day),
        ("Hour", DatePickerField::Hour),
        ("Min", DatePickerField::Minute),
    ];

    for (i, (label, field)) in fields.iter().enumerate() {
        let is_focused = state.active_field == *field;
        let col_area = picker_area[i];

        // Column Label
        let label_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(col_area);
        
        f.render_widget(
            Paragraph::new(Span::styled(
                *label,
                Style::default().fg(if is_focused { Color::Cyan } else { Color::DarkGray })
                    .add_modifier(Modifier::BOLD)
            )).alignment(Alignment::Center),
            label_area[0]
        );

        // Wheel (Previous, Current, Next)
        let wheel_items = match field {
            DatePickerField::Year => vec![
                (state.year - 1).to_string(),
                state.year.to_string(),
                (state.year + 1).to_string(),
            ],
            DatePickerField::Month => vec![
                get_month_name(if state.month == 1 { 12 } else { state.month - 1 }).to_string(),
                get_month_name(state.month).to_string(),
                get_month_name(if state.month == 12 { 1 } else { state.month + 1 }).to_string(),
            ],
            DatePickerField::Day => {
                let max_days = crate::utils::days_in_month(state.month, state.year);
                vec![
                    (if state.day == 1 { max_days } else { state.day - 1 }).to_string(),
                    state.day.to_string(),
                    (if state.day == max_days { 1 } else { state.day + 1 }).to_string(),
                ]
            }
            DatePickerField::Hour => vec![
                format!("{:02}", if state.hour == 0 { 23 } else { state.hour - 1 }),
                format!("{:02}", state.hour),
                format!("{:02}", (state.hour + 1) % 24),
            ],
            DatePickerField::Minute => vec![
                format!("{:02}", if state.minute == 0 { 59 } else { state.minute - 1 }),
                format!("{:02}", state.minute),
                format!("{:02}", (state.minute + 1) % 60),
            ],
        };

        let mut spans = vec![];
        for (idx, val) in wheel_items.iter().enumerate() {
            let mut style = Style::default().fg(Color::DarkGray);
            let mut text = val.clone();
            
            if idx == 1 {
                style = style.fg(Color::White).add_modifier(Modifier::BOLD);
                if is_focused {
                    style = style.bg(Color::Rgb(45, 45, 65)).fg(Color::Cyan);
                    text = format!(" {} ◄", text);
                } else {
                    text = format!(" {}  ", text);
                }
            } else {
                text = format!(" {}  ", text);
            }
            spans.push(ListItem::new(Line::from(Span::styled(text, style)).alignment(Alignment::Center)));
        }

        let list = List::new(spans);
        f.render_widget(list, label_area[1]);
    }
}

fn get_month_name(m: u32) -> &'static str {
    match m {
        1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr", 5 => "May", 6 => "Jun",
        7 => "Jul", 8 => "Aug", 9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
        _ => "???"
    }
}
