use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};

use crate::app::App;

pub fn render_inspector_drawer(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .title(" ◀ Inspector [←/Esc] ")
        .title_style(
            Style::default()
                .fg(Color::Rgb(137, 220, 235))
                .add_modifier(Modifier::BOLD),
        )
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(69, 71, 90)));

    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(7), // Branch overview
            Constraint::Min(5),    // Commits vs main
            Constraint::Length(5), // Background workers status
        ])
        .split(inner);

    // 1. Branch Overview
    let mut overview_lines = vec![];
    if let Some(&idx) = app.ui.filtered_indices.get(app.ui.selected_branch_idx)
        && let Some(b) = app.repo.branches.get(idx)
    {
        let is_current = b.name == app.repo.current_branch;
        let is_protected = crate::actions::commands::is_protected_branch(&b.name);
        let tag = if is_current {
            " (HEAD)"
        } else if is_protected {
            " [PROTECTED]"
        } else {
            ""
        };

        overview_lines.push(Line::from(vec![
            Span::styled(
                &b.name,
                Style::default()
                    .fg(Color::Rgb(137, 180, 250))
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                tag,
                Style::default().fg(if is_current {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ),
        ]));

        overview_lines.push(Line::from(vec![
            Span::styled("Author: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&b.author, Style::default().fg(Color::White)),
        ]));

        overview_lines.push(Line::from(vec![
            Span::styled("Updated: ", Style::default().fg(Color::DarkGray)),
            Span::styled(&b.age, Style::default().fg(Color::Rgb(250, 179, 135))),
            Span::styled(format!(" ({})", b.commit_date), Style::default().fg(Color::DarkGray)),
        ]));

        overview_lines.push(Line::from(vec![
            Span::styled("Sync: ", Style::default().fg(Color::DarkGray)),
            Span::styled(
                format!("↑{} ↓{}", b.ahead_count, b.behind_count),
                Style::default().fg(Color::Rgb(137, 220, 235)),
            ),
        ]));
    } else {
        overview_lines.push(Line::from("No branch selected"));
    }

    f.render_widget(
        Paragraph::new(overview_lines).block(
            Block::default()
                .title(" Overview ")
                .title_style(Style::default().fg(Color::Gray))
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb(49, 50, 68))),
        ),
        chunks[0],
    );

    // 2. Recent Commits vs main
    let mut commit_lines = vec![];
    if let Some(&idx) = app.ui.filtered_indices.get(app.ui.selected_branch_idx)
        && let Some(b) = app.repo.branches.get(idx)
        && !b.name.starts_with('*')
    {
        // Query recent 3 commits on this branch
        let log_out = crate::git::commands::run_git(
            ".",
            &["log", "-n", "4", "--format=%h %s", &b.name],
        )
        .unwrap_or_default();

        if log_out.trim().is_empty() {
            commit_lines.push(Line::from("No commits found"));
        } else {
            for line in log_out.lines().take(4) {
                let parts: Vec<&str> = line.splitn(2, ' ').collect();
                if parts.len() == 2 {
                    commit_lines.push(Line::from(vec![
                        Span::styled(parts[0].to_string(), Style::default().fg(Color::Rgb(137, 220, 235))),
                        Span::raw(" "),
                        Span::styled(parts[1].to_string(), Style::default().fg(Color::Rgb(205, 214, 244))),
                    ]));
                } else {
                    commit_lines.push(Line::from(line.to_string()));
                }
            }
        }
    } else {
        commit_lines.push(Line::from("Special working tree row"));
    }

    f.render_widget(
        Paragraph::new(commit_lines).block(
            Block::default()
                .title(" Recent Commits ")
                .title_style(Style::default().fg(Color::Gray))
                .borders(Borders::BOTTOM)
                .border_style(Style::default().fg(Color::Rgb(49, 50, 68))),
        ),
        chunks[1],
    );

    // 3. Live Background Workers (Claude Code style)
    let workers_lines = vec![
        Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Green)),
            Span::styled("Merge Analyzer: ", Style::default().fg(Color::Gray)),
            Span::styled("Idle (Up to date)", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Rgb(137, 220, 235))),
            Span::styled("File Watcher: ", Style::default().fg(Color::Gray)),
            Span::styled("Active", Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("● ", Style::default().fg(Color::Rgb(203, 166, 247))),
            Span::styled("AI Engine: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("Ready ({})", app.config.ai_provider),
                Style::default().fg(Color::White),
            ),
        ]),
    ];

    f.render_widget(
        Paragraph::new(workers_lines).block(
            Block::default()
                .title(" Background Workers ")
                .title_style(Style::default().fg(Color::Gray)),
        ),
        chunks[2],
    );
}
