use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::app::App;
use crate::state::ui::PrTab;
use crate::ui::layout::PR_SPLIT;

pub fn render_pull_requests(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);

    let constraints = if app.ui.maximized_panel {
        [Constraint::Length(0), Constraint::Min(0)]
    } else {
        [
            Constraint::Percentage(PR_SPLIT.0),
            Constraint::Percentage(PR_SPLIT.1),
        ]
    };
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints.as_ref())
        .split(area);

    if !app.ui.maximized_panel {
        render_pr_list(f, chunks[0], app);
    }
    render_pr_details(f, chunks[1], app);
}

fn render_pr_list(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let filter_label = app.ui.pr_filter.label();
    let total = app.repo.pull_requests.len();

    let title = if total > 0 {
        format!(
            " Pull Requests [{filter_label}] [{}/{total}] ",
            app.ui.selected_pr_idx + 1
        )
    } else {
        format!(" Pull Requests [{filter_label}] ")
    };

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.border);

    let inner = block.inner(area);
    f.render_widget(block, area);

    if app.repo.pull_requests.is_empty() {
        let mut lines = Vec::new();
        lines.push(Line::default());
        if let Some(ref err) = app.repo.pr_error {
            lines.push(Line::from(vec![
                Span::styled("  ⚠ ", theme.danger.add_modifier(Modifier::BOLD)),
                Span::styled(
                    "GitHub CLI (gh) Error:",
                    theme.danger.add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(Line::default());
            for err_line in err.lines().take(4) {
                lines.push(Line::from(vec![
                    Span::raw("    "),
                    Span::styled(err_line.to_string(), theme.muted),
                ]));
            }
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "    Check auth: `gh auth status`",
                theme.subtle,
            )));
            lines.push(Line::from(Span::styled(
                "    [r] Retry • [f] Filter • [Shift+B] Branches",
                theme.accent2,
            )));
        } else {
            lines.push(Line::from(Span::styled(
                "  No pull requests matching current filter.",
                theme.muted,
            )));
            lines.push(Line::default());
            lines.push(Line::from(Span::styled(
                "  [f] Cycle filter (Open / All / Merged / Closed)",
                theme.accent2,
            )));
            lines.push(Line::from(Span::styled("  [r] Refresh", theme.accent2)));
        }
        let p = Paragraph::new(lines).wrap(Wrap { trim: false });
        f.render_widget(p, inner);
        return;
    }

    // Viewport windowing: Each PR entry takes 2 lines, plus 1 divider line if space permits
    let entry_height = 2;
    let visible_entries = (inner.height as usize / entry_height).max(1);
    let selected = app.ui.selected_pr_idx;

    let start_idx = if total <= visible_entries || selected < visible_entries / 2 {
        0
    } else if selected + visible_entries / 2 >= total {
        total.saturating_sub(visible_entries)
    } else {
        selected - visible_entries / 2
    };
    let end_idx = (start_idx + visible_entries).min(total);

    let mut lines = Vec::new();
    for (i, pr) in app.repo.pull_requests[start_idx..end_idx]
        .iter()
        .enumerate()
    {
        let real_idx = start_idx + i;
        let is_selected = real_idx == selected;

        let (badge_str, badge_style) = match pr.state.to_uppercase().as_str() {
            "OPEN" => (
                " OPEN ",
                Style::default()
                    .bg(theme.diff_add_bg)
                    .fg(theme.diff_add_fg)
                    .add_modifier(Modifier::BOLD),
            ),
            "MERGED" => (
                " MERGED ",
                Style::default()
                    .bg(theme.surface)
                    .fg(theme.accent.fg.unwrap_or(Color::Magenta))
                    .add_modifier(Modifier::BOLD),
            ),
            _ => (
                " CLOSED ",
                Style::default()
                    .bg(theme.diff_del_bg)
                    .fg(theme.diff_del_fg)
                    .add_modifier(Modifier::BOLD),
            ),
        };

        let pointer = if is_selected { "▸ " } else { "  " };
        let pointer_style = if is_selected {
            theme.accent.add_modifier(Modifier::BOLD)
        } else {
            theme.subtle
        };

        let ci_badge = if let Some(checks) = app.repo.pr_checks.get(&pr.number) {
            match crate::models::compute_ci_status(checks) {
                crate::models::CiStatus::Passing => {
                    Span::styled("✓ ", theme.success.add_modifier(Modifier::BOLD))
                }
                crate::models::CiStatus::Failing => {
                    Span::styled("✗ ", theme.danger.add_modifier(Modifier::BOLD))
                }
                crate::models::CiStatus::Pending => {
                    Span::styled("● ", theme.warning.add_modifier(Modifier::BOLD))
                }
                crate::models::CiStatus::Unknown => Span::styled("· ", theme.dim),
            }
        } else {
            Span::styled("· ", theme.dim)
        };

        let line1_spans = vec![
            Span::styled(pointer, pointer_style),
            Span::styled(badge_str, badge_style),
            Span::raw(" "),
            ci_badge,
            Span::styled(format!("#{} ", pr.number), theme.accent2),
            Span::styled(
                pr.title.clone(),
                if is_selected {
                    theme.base.add_modifier(Modifier::BOLD)
                } else {
                    theme.base
                },
            ),
        ];

        let line2_spans = vec![
            Span::raw("    "),
            Span::styled(format!("@{}", pr.author.login), theme.soft),
            Span::styled(" • ", theme.subtle),
            Span::styled(pr.head_ref_name.clone(), theme.accent2),
            Span::styled(" ➔ ", theme.subtle),
            Span::styled(pr.base_ref_name.clone(), theme.muted),
        ];

        let mut line1 = Line::from(line1_spans);
        let mut line2 = Line::from(line2_spans);

        if is_selected {
            line1 = line1.style(Style::default().bg(theme.select_soft));
            line2 = line2.style(Style::default().bg(theme.select_soft));
        }

        lines.push(line1);
        lines.push(line2);
    }

    let p = Paragraph::new(lines);
    f.render_widget(p, inner);
}

fn render_pr_details(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();

    if app.repo.pull_requests.is_empty() {
        let block = Block::default()
            .title(" PR Details ")
            .borders(Borders::ALL)
            .border_style(theme.border);
        let inner = block.inner(area);
        f.render_widget(block, area);

        let p = Paragraph::new(vec![
            Line::default(),
            Line::from(Span::styled("  No pull request selected.", theme.subtle)),
        ]);
        f.render_widget(p, inner);
        return;
    }

    let selected_pr = &app.repo.pull_requests[app.ui.selected_pr_idx];

    // Layout: Top tab bar + metadata, bottom content
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(0)].as_ref())
        .split(area);

    // 1. Tab Bar & Metadata Header
    let is_desc = app.ui.pr_tab == PrTab::Description;
    let is_diff = app.ui.pr_tab == PrTab::Diff;
    let is_checks = app.ui.pr_tab == PrTab::Checks;
    let is_comments = app.ui.pr_tab == PrTab::Comments;

    let tab_desc_style = if is_desc {
        Style::default()
            .bg(theme.surface)
            .fg(theme.accent.fg.unwrap_or(Color::Magenta))
            .add_modifier(Modifier::BOLD)
    } else {
        theme.subtle
    };

    let tab_diff_style = if is_diff {
        Style::default()
            .bg(theme.surface)
            .fg(theme.accent.fg.unwrap_or(Color::Magenta))
            .add_modifier(Modifier::BOLD)
    } else {
        theme.subtle
    };

    let tab_checks_style = if is_checks {
        Style::default()
            .bg(theme.surface)
            .fg(theme.accent.fg.unwrap_or(Color::Magenta))
            .add_modifier(Modifier::BOLD)
    } else {
        theme.subtle
    };

    let tab_comments_style = if is_comments {
        Style::default()
            .bg(theme.surface)
            .fg(theme.accent.fg.unwrap_or(Color::Magenta))
            .add_modifier(Modifier::BOLD)
    } else {
        theme.subtle
    };

    let checks_count = app
        .repo
        .pr_checks
        .get(&selected_pr.number)
        .map(|c| c.len())
        .unwrap_or(0);
    let checks_badge_label = if checks_count > 0 {
        let checks = app.repo.pr_checks.get(&selected_pr.number).unwrap();
        match crate::models::compute_ci_status(checks) {
            crate::models::CiStatus::Passing => format!(" [ 3: Checks (✓ {checks_count}) ] "),
            crate::models::CiStatus::Failing => " [ 3: Checks (✗ failed) ] ".to_string(),
            crate::models::CiStatus::Pending => " [ 3: Checks (● pending) ] ".to_string(),
            crate::models::CiStatus::Unknown => format!(" [ 3: Checks ({checks_count}) ] "),
        }
    } else {
        " [ 3: Checks ] ".to_string()
    };

    let comments_count = app
        .repo
        .pr_comments
        .get(&selected_pr.number)
        .map(|c| c.len())
        .unwrap_or(0);
    let comments_badge_label = if comments_count > 0 {
        format!(" [ 4: Discussion ({comments_count}) ] ")
    } else {
        " [ 4: Discussion ] ".to_string()
    };

    let header_block = Block::default()
        .title(" Pull Request Overview ")
        .borders(Borders::ALL)
        .border_style(theme.border);
    let header_inner = header_block.inner(chunks[0]);
    f.render_widget(header_block, chunks[0]);

    let action_hint = if is_checks {
        " (Tab: tabs │ d/u: select check │ Enter: open log │ a: approve │ o: web)"
    } else if is_comments {
        " (Tab: tabs │ u/d: scroll discussion │ m: add comment │ a: approve │ o: web)"
    } else {
        " (Tab: tabs │ a: approve │ x: request changes │ m: comment │ c: checkout │ o: web)"
    };

    let meta_lines = vec![
        Line::from(vec![
            Span::styled(" [ 1: Description ] ", tab_desc_style),
            Span::styled(" [ 2: Diff ] ", tab_diff_style),
            Span::styled(checks_badge_label, tab_checks_style),
            Span::styled(comments_badge_label, tab_comments_style),
            Span::styled(action_hint, theme.subtle),
        ]),
        Line::from(vec![
            Span::styled(
                format!(" #{} ", selected_pr.number),
                theme.accent2.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                selected_pr.title.clone(),
                theme.base.add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled(format!(" by @{} ", selected_pr.author.login), theme.soft),
            Span::styled(" • ", theme.subtle),
            Span::styled(selected_pr.head_ref_name.clone(), theme.accent2),
            Span::styled(" ➔ ", theme.subtle),
            Span::styled(selected_pr.base_ref_name.clone(), theme.muted),
            Span::styled(" • URL: ", theme.subtle),
            Span::styled(selected_pr.url.clone(), theme.subtle),
        ]),
    ];

    let meta_p = Paragraph::new(meta_lines);
    f.render_widget(meta_p, header_inner);

    // 2. Content Area (Markdown Description, Unified Diff, or CI Checks)
    let visible_rows = chunks[1].height.saturating_sub(2) as usize;
    let (total_lines, type_title, content_widget) = if is_desc {
        let md_content = if selected_pr.body.trim().is_empty() {
            "*No description provided for this pull request.*"
        } else {
            &selected_pr.body
        };
        let md_lines = crate::ui::markdown::render_markdown(md_content, &theme);
        let len = md_lines.len();
        let p = Paragraph::new(md_lines)
            .scroll((app.ui.pr_scroll_y, 0))
            .wrap(Wrap { trim: false });
        (len, "Description (Markdown)", p)
    } else if is_diff {
        let diff_text = app
            .repo
            .pr_diff
            .as_deref()
            .unwrap_or("Loading diff from GitHub...");
        let diff_lines = crate::ui::components::format_diff_with_gutters(diff_text, &theme);
        let len = diff_lines.len();
        let p = Paragraph::new(diff_lines)
            .scroll((app.ui.pr_scroll_y, 0))
            .wrap(Wrap { trim: false });
        (len, "Changes Diff", p)
    } else if is_checks {
        let checks = app.repo.pr_checks.get(&selected_pr.number);
        let mut check_lines = Vec::new();
        check_lines.push(Line::default());
        if let Some(list) = checks {
            if list.is_empty() {
                check_lines.push(Line::from(Span::styled(
                    "  No CI checks reported for this pull request.",
                    theme.muted,
                )));
            } else {
                for (idx, c) in list.iter().enumerate() {
                    let is_focused = idx == app.ui.pr_check_idx;
                    let pointer = if is_focused { "▸ " } else { "  " };
                    let (icon, icon_style) = match c.bucket.as_str() {
                        "pass" => (" ✓ ", theme.success.add_modifier(Modifier::BOLD)),
                        "fail" | "cancel" => (" ✗ ", theme.danger.add_modifier(Modifier::BOLD)),
                        "pending" => (" ● ", theme.warning.add_modifier(Modifier::BOLD)),
                        _ => (" · ", theme.dim),
                    };
                    let desc = if c.description.is_empty() {
                        c.state.clone()
                    } else {
                        format!("{} - {}", c.state, c.description)
                    };
                    let mut line_style = Style::default();
                    if is_focused {
                        line_style = line_style.bg(theme.select_soft);
                    }
                    check_lines.push(
                        Line::from(vec![
                            Span::styled(
                                pointer,
                                if is_focused {
                                    theme.accent.add_modifier(Modifier::BOLD)
                                } else {
                                    theme.subtle
                                },
                            ),
                            Span::styled(icon, icon_style),
                            Span::styled(
                                format!("{:<28} ", c.name),
                                theme.base.add_modifier(Modifier::BOLD),
                            ),
                            Span::styled(desc, theme.soft),
                        ])
                        .style(line_style),
                    );
                    if !c.link.is_empty() {
                        let link_hint = if is_focused {
                            " [Enter: open log] "
                        } else {
                            ""
                        };
                        check_lines.push(
                            Line::from(vec![
                                Span::raw("     ↳ URL: "),
                                Span::styled(
                                    &c.link,
                                    if is_focused {
                                        theme.nav.add_modifier(Modifier::UNDERLINED)
                                    } else {
                                        theme.subtle
                                    },
                                ),
                                Span::styled(link_hint, theme.accent2),
                            ])
                            .style(line_style),
                        );
                    }
                    check_lines.push(Line::default());
                }
            }
        } else {
            check_lines.push(Line::from(Span::styled(
                "  Loading checks from GitHub...",
                theme.subtle,
            )));
        }
        let len = check_lines.len();
        let p = Paragraph::new(check_lines)
            .scroll((app.ui.pr_scroll_y, 0))
            .wrap(Wrap { trim: false });
        (len, "CI Checks", p)
    } else {
        let comments = app.repo.pr_comments.get(&selected_pr.number);
        let mut comment_lines = Vec::new();
        comment_lines.push(Line::default());
        if let Some(list) = comments {
            if list.is_empty() {
                comment_lines.push(Line::from(Span::styled(
                    "  No discussion comments yet on this pull request.",
                    theme.muted,
                )));
            } else {
                for (idx, c) in list.iter().enumerate() {
                    let header_line = Line::from(vec![
                        Span::styled(
                            format!(" #{} 💬 @{} ", idx + 1, c.author.login),
                            theme.accent.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(format!(" • {} ", c.created_at), theme.subtle),
                    ]);
                    comment_lines.push(header_line);
                    comment_lines.push(Line::default());
                    let rendered = crate::ui::markdown::render_markdown(&c.body, &theme);
                    comment_lines.extend(rendered);
                    comment_lines.push(Line::default());
                    comment_lines.push(Line::from(Span::styled(
                        "────────────────────────────────────────",
                        theme.dim,
                    )));
                    comment_lines.push(Line::default());
                }
            }
        } else {
            comment_lines.push(Line::from(Span::styled(
                "  Loading comments from GitHub...",
                theme.subtle,
            )));
        }
        let len = comment_lines.len();
        let p = Paragraph::new(comment_lines)
            .scroll((app.ui.pr_scroll_y, 0))
            .wrap(Wrap { trim: false });
        (len, "Discussion", p)
    };

    let scroll_badge = crate::ui::components::scroll_indicator(
        app.ui.pr_scroll_y as usize,
        visible_rows,
        total_lines,
    );
    let zoom_hint = if app.ui.maximized_panel {
        "[Z: restore] "
    } else {
        "[Z: zoom] "
    };
    let content_title = if is_checks {
        format!(
            " {} {} {}[d/u: navigate • Enter: open log] ",
            type_title, scroll_badge, zoom_hint
        )
    } else {
        format!(
            " {} {} {}[u/d: scroll] ",
            type_title, scroll_badge, zoom_hint
        )
    };

    let content_block = Block::default()
        .title(content_title)
        .borders(Borders::ALL)
        .border_style(theme.active_border);
    let content_inner = content_block.inner(chunks[1]);
    f.render_widget(content_block, chunks[1]);
    f.render_widget(content_widget, content_inner);
}
