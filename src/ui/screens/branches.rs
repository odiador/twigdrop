use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
};

use crate::app::App;
use crate::models::BranchStatus;
use crate::state::ui::AppMode;
use crate::ui::components::get_status_icons;

/// Branches table columns: selector / branch / sync / merge-health / age / last-commit.
/// Fixed widths are single source here; branch + last-commit share the remainder.
const COL_SEL_W: u16 = 5;
const COL_SYNC_W: u16 = 12;
const COL_MERGE_W: u16 = 16;
const COL_AGE_W: u16 = 12;
/// Flexible columns (branch name, last commit) share of the table width.
const COL_FLEX_PCT: u16 = 30;
/// Branch column share in narrow mode (author column collapsed).
const COL_FLEX_NARROW_PCT: u16 = 40;

pub fn render_current_branch_hero(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let loc = app.locale();

    let current_b = app
        .repo
        .branches
        .iter()
        .find(|b| b.name == app.repo.current_branch);

    let filter_badge = if app.ui.branches_active_only {
        Span::styled(
            " [ACTIVE ONLY • Tab: all] ",
            theme.warning.add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(" [ALL BRANCHES • Tab: active only] ", theme.soft)
    };

    let hero_block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                " 🌿 Current Branch ",
                theme.accent.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("⎇ {} ", &app.repo.current_branch),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            filter_badge,
        ]))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Green));

    // Line 1: Sync status & Health & Uncommitted workspace changes
    let mut line1 = vec![Span::styled("Sync: ", theme.dim)];

    if let Some(b) = current_b {
        if b.ahead_count > 0 || b.behind_count > 0 {
            if b.ahead_count > 0 {
                line1.push(Span::styled(
                    format!("↑{} ahead ", b.ahead_count),
                    theme.warning.add_modifier(Modifier::BOLD),
                ));
            }
            if b.behind_count > 0 {
                line1.push(Span::styled(
                    format!("↓{} behind ", b.behind_count),
                    theme.accent2.add_modifier(Modifier::BOLD),
                ));
            }
        } else if b.status.contains(&BranchStatus::RemoteTracked) {
            line1.push(Span::styled("✓ Up to date with remote ", theme.merged));
        } else {
            line1.push(Span::styled("Local only (no upstream) ", theme.dim));
        }

        let (merge_text, merge_color) =
            crate::ui::components::get_merge_status_display(&b.merge_status, loc, &theme);
        line1.push(Span::styled("• Health: ", theme.dim));
        line1.push(Span::styled(
            format!("{merge_text} "),
            Style::default().fg(merge_color),
        ));
    }

    let staged = app
        .repo
        .git_file_statuses
        .values()
        .filter(|s| **s == crate::git::files::FileStatus::Staged)
        .count();
    let modified = app
        .repo
        .git_file_statuses
        .values()
        .filter(|s| **s == crate::git::files::FileStatus::Modified)
        .count();
    let untracked = app
        .repo
        .git_file_statuses
        .values()
        .filter(|s| **s == crate::git::files::FileStatus::Untracked)
        .count();
    let total_uncommitted = staged + modified + untracked;

    line1.push(Span::styled("• Working tree: ", theme.dim));
    if total_uncommitted == 0 {
        line1.push(Span::styled("clean ", theme.merged));
    } else {
        let mut work_parts = Vec::new();
        if staged > 0 {
            work_parts.push(format!("{staged} staged"));
        }
        if modified > 0 {
            work_parts.push(format!("{modified} modified"));
        }
        if untracked > 0 {
            work_parts.push(format!("{untracked} untracked"));
        }
        line1.push(Span::styled(work_parts.join(", "), theme.warning));
    }

    // Line 2: HEAD commit
    let mut line2 = vec![Span::styled("HEAD: ", theme.dim)];
    if let Some(commit) = &app.repo.head_commit {
        line2.push(Span::styled(
            format!("{} ", &commit.hash),
            Style::default().fg(Color::Yellow),
        ));
        line2.push(Span::styled(
            commit.message.clone(),
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ));
        line2.push(Span::styled(format!(" ({})", commit.date), theme.dim));
    } else {
        line2.push(Span::styled("No commit information", theme.dim));
    }

    let hero_widget = Paragraph::new(vec![Line::from(line1), Line::from(line2)]).block(hero_block);
    f.render_widget(hero_widget, area);
}

pub fn render_main_list(f: &mut Frame, area: Rect, app: &App) {
    use crate::ui::layout::{INSPECTOR_SPLIT, SIDEBAR_SPLIT};
    let (table_area, maybe_inspector, maybe_sidebar) = if app.ui.show_inspector_drawer {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(INSPECTOR_SPLIT.0),
                Constraint::Percentage(INSPECTOR_SPLIT.1),
            ])
            .split(area);
        (chunks[0], Some(chunks[1]), None)
    } else if app.ui.show_nav_sidebar {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(SIDEBAR_SPLIT.0),
                Constraint::Percentage(SIDEBAR_SPLIT.1),
            ])
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

    let (hero_area, actual_table_area) = if table_area.height >= 16 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(4), Constraint::Min(6)])
            .split(table_area);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, table_area)
    };

    if let Some(ha) = hero_area {
        render_current_branch_hero(f, ha, app);
    }

    let filtered_indices = app.ui.filtered_indices.clone();
    let branches_len = filtered_indices.len();
    let inner_height = actual_table_area.height.saturating_sub(4) as usize;

    if inner_height == 0 {
        return;
    }

    let loc = app.locale();
    // Frozen Catppuccin default: visual-noop vs previous Rgb() literals.
    let theme = app.theme();
    // Narrow terminals collapse the author column (UI_GUIDE §3).
    let narrow = actual_table_area.width < crate::ui::theme::NARROW_WIDTH;

    // Designed empty state (UI_GUIDE §3) — never a bare header.
    if branches_len == 0 {
        let empty = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                loc.branches.empty_title,
                theme.warning.add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            Line::from(""),
            Line::from(Span::styled(loc.branches.empty_hint, theme.muted))
                .alignment(Alignment::Center),
        ])
        .block(
            Block::default()
                .title(Line::from(vec![Span::styled(
                    format!(" {} ", loc.branches.title_branches),
                    theme.accent.add_modifier(Modifier::BOLD),
                )]))
                .borders(Borders::ALL)
                .border_style(theme.border),
        );
        f.render_widget(empty, actual_table_area);
        return;
    }

    // Scroll math lives in `UiState::branch_viewport` (pure + tested);
    // positions below are per-frame mouse hit data in `FrameCache`.
    let (start, branch_items_to_show) = app.frame_cache.borrow_mut().sync_branches(
        app.ui.selected_branch_idx,
        branches_len,
        inner_height,
    );

    let mut rows: Vec<Row> = vec![];

    for i in 0..branch_items_to_show {
        let branch_idx = start + i;
        if branch_idx >= branches_len {
            break;
        }

        let actual_idx = filtered_indices[branch_idx];
        let b = &app.repo.branches[actual_idx];
        let selected = branch_idx == app.ui.selected_branch_idx;
        let is_current = b.name == app.repo.current_branch;

        app.frame_cache.borrow_mut().branch_screen_positions.push((
            actual_idx,
            actual_table_area
                .y
                .saturating_add(3)
                .saturating_add(u16::try_from(i).unwrap_or(u16::MAX)),
        ));

        let (icons, color) = get_status_icons(&b.status, &theme);
        let (merge_text, merge_color) =
            crate::ui::components::get_merge_status_display(&b.merge_status, loc, &theme);

        let current_tag = if is_current { " (HEAD)" } else { "" };
        let unique_tag = if b.status.contains(&BranchStatus::HasUniqueCommits) {
            " [NEW COMMITS]"
        } else {
            ""
        };
        let is_protected = app.config.is_protected(&b.name);
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
            loc.branches.sync_synced.to_string()
        } else {
            loc.branches.sync_local.to_string()
        };

        let sync_color = if b.ahead_count > 0 {
            theme.warning.fg.unwrap_or(Color::Yellow)
        } else if b.behind_count > 0 {
            theme.accent2.fg.unwrap_or(Color::Cyan)
        } else if b.status.contains(&BranchStatus::RemoteTracked) {
            theme.success.fg.unwrap_or(Color::Green)
        } else {
            theme.muted.fg.unwrap_or(Color::Gray)
        };

        let branch_name = format!("{}{}{}", b.name, current_tag, unique_tag);
        let author_str = format!("{} by {}", b.commit_date, b.author);

        let mut row_style = theme.base;
        let mut branch_style = Style::default().fg(color);
        if is_current {
            branch_style = branch_style
                .add_modifier(Modifier::BOLD)
                .fg(theme.success.fg.unwrap_or(Color::Green));
        } else if b.status.contains(&BranchStatus::HasUniqueCommits) {
            branch_style = branch_style
                .add_modifier(Modifier::BOLD)
                .fg(theme.warning.fg.unwrap_or(Color::Yellow));
        }

        if selected {
            row_style = row_style.bg(theme.highlight.bg.unwrap_or(Color::DarkGray));
        }

        let checkbox_prefix = if selected {
            format!("▎{checkbox}")
        } else {
            format!(" {checkbox}")
        };

        let checkbox_style = if is_protected {
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        } else if is_bulk_selected {
            theme.success.add_modifier(Modifier::BOLD)
        } else {
            theme.faint
        };

        let mut cells = vec![
            Cell::from(checkbox_prefix).style(checkbox_style),
            Cell::from(Line::from(vec![
                Span::styled(format!("{icons:<4} "), Style::default().fg(color)),
                Span::styled(branch_name, branch_style),
            ])),
            Cell::from(sync_text).style(Style::default().fg(sync_color)),
            Cell::from(merge_text).style(Style::default().fg(merge_color)),
            Cell::from(b.age.clone()).style(theme.warning),
        ];
        // Narrow terminals (< NARROW_WIDTH) drop the author column (UI_GUIDE §3).
        if !narrow {
            cells.push(Cell::from(author_str).style(theme.soft));
        }

        rows.push(Row::new(cells).style(row_style));
    }

    let widths: Vec<Constraint> = if narrow {
        vec![
            Constraint::Length(COL_SEL_W),
            Constraint::Percentage(COL_FLEX_NARROW_PCT),
            Constraint::Length(COL_SYNC_W),
            Constraint::Length(COL_MERGE_W),
            Constraint::Length(COL_AGE_W),
        ]
    } else {
        vec![
            Constraint::Length(COL_SEL_W),
            Constraint::Percentage(COL_FLEX_PCT),
            Constraint::Length(COL_SYNC_W),
            Constraint::Length(COL_MERGE_W),
            Constraint::Length(COL_AGE_W),
            Constraint::Percentage(COL_FLEX_PCT),
        ]
    };

    let sort_hint = match app.ui.branch_sort_mode {
        crate::state::ui::BranchSortMode::Recent => loc.branches.sort_recent,
        crate::state::ui::BranchSortMode::PrunableFirst => loc.branches.sort_prunable,
        crate::state::ui::BranchSortMode::Alphabetical => loc.branches.sort_alphabetical,
    };

    let mut header_cells = vec![
        loc.branches.header_sel,
        loc.branches.header_branch,
        loc.branches.header_sync,
        loc.branches.header_merge_health,
        loc.branches.header_age,
    ];
    if !narrow {
        header_cells.push(loc.branches.header_last_commit);
    }
    let table = Table::new(rows, widths)
        .header(Row::new(header_cells).style(theme.header).bottom_margin(1))
        .block(
            Block::default()
                .title(if !app.ui.search_query.is_empty() {
                    Line::from(vec![
                        Span::styled(
                            format!(" {} ", loc.branches.title_branches),
                            theme.accent.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(" ", theme.warning.add_modifier(Modifier::BOLD)),
                        Span::styled(
                            &app.ui.search_query,
                            Style::default()
                                .fg(Color::White)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("▌ ", theme.accent),
                        Span::styled(format!("{} • ", loc.branches.esc_to_clear), theme.muted),
                        Span::styled(
                            format!("{}: {} ", loc.branches.sort_label, sort_hint),
                            theme.muted,
                        ),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(
                            format!(" {} ", loc.branches.title_branches),
                            theme.accent.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            if app.ui.branches_active_only {
                                "[ACTIVE ONLY] • "
                            } else {
                                ""
                            },
                            theme.warning.add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("Tab: toggle filter • ", theme.muted),
                        Span::styled(format!("{} • ", loc.branches.type_to_filter), theme.subtle),
                        Span::styled(
                            format!("{}: {} • ", loc.branches.sort_label, sort_hint),
                            theme.muted,
                        ),
                        Span::styled(
                            format!(
                                "{} • {} ",
                                loc.branches.inspector_hint, loc.branches.sidebar_hint
                            ),
                            theme.muted,
                        ),
                    ])
                })
                .title(
                    Line::from(vec![Span::styled(
                        loc.format_range(
                            if branches_len == 0 { 0 } else { start + 1 },
                            start + branch_items_to_show,
                            branches_len,
                        ),
                        theme.muted,
                    )])
                    .alignment(Alignment::Right),
                )
                .borders(Borders::ALL)
                .border_style(theme.border),
        );

    if app.ui.current_mode() == &AppMode::Diff {
        super::diff::render_diff_overlay(f, actual_table_area, app);
    } else {
        f.render_widget(table, actual_table_area);
    }
}
