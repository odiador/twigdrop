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

/// Commits split: graph list / details.
const COMMITS_LIST_PCT: u16 = 60;
const COMMITS_DETAILS_PCT: u16 = 40;

pub fn render_commits(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(COMMITS_LIST_PCT),
                Constraint::Percentage(COMMITS_DETAILS_PCT),
            ]
            .as_ref(),
        )
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
                    '|' | '/' | '\\' | '_' => theme.graph.fg.unwrap_or(Color::Gray),
                    _ => Color::Gray,
                };
                graph_spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            }

            let mut all_spans = graph_spans;

            if !commit.hash.is_empty() {
                all_spans.push(Span::styled(
                    format!(" {} ", commit.hash),
                    Style::default().fg(Color::Yellow),
                ));

                if !commit.branch_info.is_empty() {
                    all_spans.push(Span::styled(
                        format!(" {} ", commit.branch_info),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                if !commit.date.is_empty() {
                    all_spans.push(Span::styled(
                        format!(" {} ", commit.date),
                        Style::default().fg(Color::Cyan),
                    ));
                }

                if !commit.author.is_empty() {
                    all_spans.push(Span::styled(format!(" [{}] ", commit.author), theme.muted2));
                }

                if !commit.message.is_empty() {
                    all_spans.push(Span::styled(
                        commit.message.clone(),
                        Style::default().fg(Color::White),
                    ));
                }
            }

            let mut line_style = Style::default();
            if is_selected && !commit.hash.is_empty() {
                let files_focused = app.ui.commit_panel == crate::state::ui::CommitPanel::Files;
                line_style = if files_focused {
                    Style::default().bg(theme.cursor_unfocused)
                } else {
                    theme.select_hi
                };
            }

            items.push(ListItem::new(Line::from(all_spans)).style(line_style));
        }
    }

    let tree_border_style = if app.ui.commit_panel == crate::state::ui::CommitPanel::Tree {
        Style::default().fg(Color::Cyan)
    } else {
        theme.border
    };

    let list_title = if app.ui.commit_panel == crate::state::ui::CommitPanel::Tree {
        " Git Commit Tree (All Branches) [Tab: focus files] "
    } else {
        " Git Commit Tree (All Branches) "
    };

    let list = List::new(items).block(
        Block::default()
            .title(list_title)
            .borders(Borders::ALL)
            .border_style(tree_border_style),
    );
    f.render_widget(list, list_area);

    if let Some(commit) = app.repo.commit_tree.get(app.ui.selected_commit_idx) {
        let total_files = app.repo.commit_files.len();
        let files_height = (total_files as u16 + 2).clamp(7, 12);
        let detail_constraints = if app.ui.maximized_panel {
            [
                Constraint::Length(0),
                Constraint::Length(0),
                Constraint::Min(6),
            ]
        } else {
            [
                Constraint::Length(5),
                Constraint::Length(files_height),
                Constraint::Min(6),
            ]
        };
        let detail_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(detail_constraints)
            .split(details_area);

        let stats_summary = app
            .repo
            .commit_stats
            .as_deref()
            .and_then(|s| s.lines().last())
            .unwrap_or("");

        let meta_lines = vec![
            Line::from(vec![
                Span::styled("Hash: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&commit.hash, Style::default().fg(Color::Yellow)),
                Span::styled("  Author: ", Style::default().fg(Color::DarkGray)),
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
            Line::from(vec![
                Span::styled("Stats: ", Style::default().fg(Color::DarkGray)),
                Span::styled(stats_summary, Style::default().fg(Color::Yellow)),
            ]),
        ];

        if !app.ui.maximized_panel {
            let meta_block = Block::default()
                .title(" Commit Details ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow));
            f.render_widget(
                Paragraph::new(meta_lines).block(meta_block),
                detail_chunks[0],
            );
        }

        let files_focused = app.ui.commit_panel == crate::state::ui::CommitPanel::Files;
        let files_border_color = if files_focused {
            theme.nav.fg.unwrap_or(Color::Yellow)
        } else {
            theme.border.fg.unwrap_or(Color::DarkGray)
        };

        let visible_rows = detail_chunks[1].height.saturating_sub(2) as usize;
        let visible_rows = visible_rows.max(1);
        let sel_idx = app.ui.selected_commit_file_idx;
        let start_idx = if sel_idx >= visible_rows {
            sel_idx + 1 - visible_rows
        } else {
            0
        };
        let end_idx = (start_idx + visible_rows).min(total_files);

        let files_title = if total_files == 0 {
            " Files (0) [Tab: switch] ".to_string()
        } else {
            format!(
                " Files ({}) [{}/{}] [Tab: switch] ",
                total_files,
                sel_idx + 1,
                total_files
            )
        };

        let files_block = Block::default()
            .title(files_title)
            .borders(Borders::ALL)
            .border_style(Style::default().fg(files_border_color));

        let mut file_items = vec![];
        if app.repo.commit_files.is_empty() {
            file_items.push(ListItem::new(" No files changed in this commit ").style(theme.dim));
        } else {
            for (i, entry) in app.repo.commit_files[start_idx..end_idx].iter().enumerate() {
                let actual_idx = start_idx + i;
                let is_selected = actual_idx == sel_idx;

                let (badge, badge_color) = match entry.status {
                    FileStatus::Added => ("[A]", theme.merged.fg.unwrap_or(Color::Green)),
                    FileStatus::Modified => ("[M]", theme.warning.fg.unwrap_or(Color::Yellow)),
                    FileStatus::Deleted => ("[D]", theme.danger.fg.unwrap_or(Color::Red)),
                    FileStatus::Staged => ("[S]", theme.info.fg.unwrap_or(Color::Blue)),
                    FileStatus::Conflict => ("[C]", theme.danger.fg.unwrap_or(Color::Red)),
                    _ => ("[·]", Color::Gray),
                };

                let pointer = if is_selected { "▸ " } else { "  " };

                let (row_bg, row_fg) = if is_selected {
                    if files_focused {
                        (Some(Color::Rgb(45, 65, 105)), Color::White)
                    } else {
                        (Some(Color::Rgb(40, 42, 60)), Color::Rgb(220, 224, 240))
                    }
                } else {
                    (None, Color::Rgb(180, 185, 205))
                };

                let mut item_style = Style::default().fg(row_fg);
                if let Some(bg) = row_bg {
                    item_style = item_style.bg(bg);
                }
                if is_selected {
                    item_style = item_style.add_modifier(Modifier::BOLD);
                }

                let line = Line::from(vec![
                    Span::styled(
                        pointer,
                        if is_selected {
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            item_style
                        },
                    ),
                    Span::styled(
                        format!("{badge} "),
                        Style::default()
                            .fg(badge_color)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(entry.path.to_string_lossy().to_string(), item_style),
                ])
                .style(item_style);

                file_items.push(ListItem::new(line).style(item_style));
            }
        }
        if !app.ui.maximized_panel {
            let files_list = List::new(file_items).block(files_block);
            f.render_widget(files_list, detail_chunks[1]);
        }

        let selected_file_name = app
            .repo
            .commit_files
            .get(app.ui.selected_commit_file_idx)
            .map(|f| f.path.to_string_lossy().to_string());

        let diff_content = app
            .repo
            .commit_file_diff
            .as_deref()
            .or(app.repo.commit_diff.as_deref())
            .unwrap_or("");

        let diff_lines = crate::ui::components::format_diff_with_gutters(diff_content, &theme);
        let diff_visible_rows = detail_chunks[2].height.saturating_sub(2) as usize;
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

        let diff_title = match &selected_file_name {
            Some(name) => format!(
                " Diff: {} {} {}[d/u: scroll] ",
                name, scroll_badge, zoom_hint
            ),
            None => format!(" Diff Preview {} {}[d/u: scroll] ", scroll_badge, zoom_hint),
        };

        let diff_block = Block::default()
            .title(diff_title)
            .borders(Borders::ALL)
            .border_style(if app.ui.maximized_panel {
                theme.active_border
            } else {
                theme.border
            });
        let diff_widget = Paragraph::new(diff_lines)
            .block(diff_block)
            .scroll((app.ui.info_scroll, 0))
            .wrap(ratatui::widgets::Wrap { trim: false });
        f.render_widget(diff_widget, detail_chunks[2]);
    } else {
        let details_block = Block::default()
            .title(" Commit Details ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow));
        f.render_widget(
            Paragraph::new("No commit selected").block(details_block),
            details_area,
        );
    }
}

pub fn render_commit_action(f: &mut Frame, app: &App, hash: &str) {
    let theme = app.theme();
    let (mx, my) = crate::ui::layout::COMMIT_ACTION_MODAL;
    let inner = crate::ui::components::centered_rect(mx, my, f.area());
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
            style = theme.select_hi.add_modifier(Modifier::BOLD);
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
            .title(format!(" Manage Commit: {hash} "))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Yellow)),
    );
    f.render_widget(list, inner);
}

pub fn render_interactive_rebase(f: &mut Frame, app: &App) {
    let theme = app.theme();
    let (mx, my) = crate::ui::layout::REBASE_MODAL;
    let area = crate::ui::components::centered_rect(mx, my, f.area());
    f.render_widget(Clear, area);

    let block = Block::default()
        .title(" Interactive Rebase ")
        .borders(Borders::ALL)
        .border_style(theme.accent);

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
                style = theme.highlight;
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
    let theme = app.theme();
    let (mx, my) = crate::ui::layout::COMMIT_FILES_MODAL;
    let inner = crate::ui::components::centered_rect(mx, my, f.area());

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
                style = theme.select_hi.add_modifier(Modifier::BOLD);
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
        .title(format!(" Files in commit {hash} "))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    let list = List::new(items).block(block);
    f.render_widget(list, inner);

    // Footer for this modal
    let footer_area = Rect::new(inner.x + 1, inner.y + inner.height - 1, inner.width - 2, 1);
    f.render_widget(
        Paragraph::new(app.locale().footer.commit_files)
            .style(Style::default().fg(Color::DarkGray)),
        footer_area,
    );
}
