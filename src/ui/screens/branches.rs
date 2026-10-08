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
use crate::ui::theme::Theme;

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

pub fn render_main_list(f: &mut Frame, area: Rect, app: &mut App) {
    use crate::ui::layout::{INSPECTOR_SPLIT, SIDEBAR_SPLIT};
    let (table_area, maybe_inspector, maybe_sidebar) = if app.ui.show_inspector_drawer {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(INSPECTOR_SPLIT.0), Constraint::Percentage(INSPECTOR_SPLIT.1)])
            .split(area);
        (chunks[0], Some(chunks[1]), None)
    } else if app.ui.show_nav_sidebar {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(SIDEBAR_SPLIT.0), Constraint::Percentage(SIDEBAR_SPLIT.1)])
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

    let loc = app.locale();
    // Frozen Catppuccin default: visual-noop vs previous Rgb() literals.
    let theme = Theme::dark_default();
    // Narrow terminals collapse the author column (UI_GUIDE §3).
    let narrow = table_area.width < crate::ui::theme::NARROW_WIDTH;

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
        f.render_widget(empty, table_area);
        return;
    }

    // Scroll math lives in `UiState::branch_viewport` (pure + tested);
    // positions below are per-frame mouse hit data rebuilt each render.
    let (start, branch_items_to_show) =
        app.ui.sync_branch_viewport(branches_len, inner_height);

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

        app.ui
            .branch_screen_positions
            .push((actual_idx, table_area.y + 3 + i as u16));

        let (icons, color) = get_status_icons(&b.status);
        let (merge_text, merge_color) =
            crate::ui::components::get_merge_status_display(&b.merge_status, loc);

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

        let branch_name = format!("{}{}", b.name, current_tag);
        let author_str = format!("{} by {}", b.commit_date, b.author);

        let mut row_style = theme.base;
        let mut branch_style = Style::default().fg(color);
        if is_current {
            branch_style = branch_style.add_modifier(Modifier::BOLD).fg(theme.success.fg.unwrap_or(Color::Green));
        }

        if selected {
            row_style = row_style.bg(theme.highlight.bg.unwrap_or(Color::DarkGray));
        }

        let checkbox_prefix = if selected {
            format!("▎{}", checkbox)
        } else {
            format!(" {}", checkbox)
        };

        let checkbox_style = if is_protected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else if is_bulk_selected {
            theme.success.add_modifier(Modifier::BOLD)
        } else {
            theme.faint
        };

        let mut cells = vec![
            Cell::from(checkbox_prefix).style(checkbox_style),
            Cell::from(Line::from(vec![
                Span::styled(format!("{:<4} ", icons), Style::default().fg(color)),
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
        .header(
            Row::new(header_cells)
            .style(theme.header)
            .bottom_margin(1),
        )
        .block(
            Block::default()
                .title(if !app.ui.search_query.is_empty() {
                    Line::from(vec![
                        Span::styled(format!(" {} ", loc.branches.title_branches), theme.accent.add_modifier(Modifier::BOLD)),
                        Span::styled(" ", theme.warning.add_modifier(Modifier::BOLD)),
                        Span::styled(&app.ui.search_query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
                        Span::styled("▌ ", theme.accent),
                        Span::styled(format!("{} • ", loc.branches.esc_to_clear), theme.muted),
                        Span::styled(format!("{}: {} ", loc.branches.sort_label, sort_hint), theme.muted),
                    ])
                } else {
                    Line::from(vec![
                        Span::styled(format!(" {} ", loc.branches.title_branches), theme.accent.add_modifier(Modifier::BOLD)),
                        Span::styled(format!("{} • ", loc.branches.type_to_filter), theme.subtle),
                        Span::styled(format!("{}: {} • ", loc.branches.sort_label, sort_hint), theme.muted),
                        Span::styled(format!("{} • {} ", loc.branches.inspector_hint, loc.branches.sidebar_hint), theme.muted),
                    ])
                })
                .title(
                    Line::from(vec![
                        Span::styled(
                            loc.format_range(if branches_len == 0 { 0 } else { start + 1 }, start + branch_items_to_show, branches_len),
                            theme.muted,
                        ),
                    ])
                    .alignment(Alignment::Right),
                )
                .borders(Borders::ALL)
                .border_style(theme.border),
        );

    if app.ui.current_mode() == &AppMode::Diff {
        super::diff::render_diff_overlay(f, table_area, app);
    } else {
        f.render_widget(table, table_area);
    }
}

pub fn render_filter(f: &mut Frame, app: &App) {
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area);
    let (mx, my) = crate::ui::theme::MODAL_MD;
    let inner = crate::ui::components::centered_rect(mx, my, full_area);
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
    let theme = Theme::dark_default();
    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = theme.base;
        if i == app.ui.filter_selected {
            style = theme.highlight;
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let block = Block::default()
        .title(Line::from(" Filter by Status ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(theme.border);
    let list = List::new(items).block(block);
    f.render_widget(list, inner);
}

pub fn render_confirm_delete(f: &mut Frame, names: &[String], locale: &crate::i18n::Locale) {
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area);

    let (mx, my) = crate::ui::theme::MODAL_MD;
    let inner = crate::ui::components::centered_rect(mx, my, full_area);

    f.render_widget(Clear, inner);

    let branch_list = if names.len() > 3 {
        format!("{} branches (including {})", names.len(), names[0])
    } else {
        names.join(", ")
    };

    let theme = Theme::dark_default();
    let block = Block::default()
        .title(Line::from(locale.branches.confirm_unpushed_title).alignment(Alignment::Center))
        .borders(Borders::ALL)
        .border_style(theme.danger.add_modifier(Modifier::BOLD))
        .style(Style::default().bg(theme.danger_surface));

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            locale.branches.confirm_unique_msg,
            theme.warning.add_modifier(Modifier::BOLD),
        ))
        .alignment(Alignment::Center),
        Line::from(""),
        Line::from(Span::styled(branch_list, theme.accent2))
            .alignment(Alignment::Center),
        Line::from(""),
        Line::from(locale.branches.confirm_data_loss)
            .alignment(Alignment::Center),
        Line::from(""),
        Line::from(locale.branches.confirm_prompt)
            .alignment(Alignment::Center),
    ];
    let p = Paragraph::new(text).block(block);
    f.render_widget(p, inner);
}

pub fn render_create_branch(f: &mut Frame, input: &str) {
    let full_area = f.area();
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area);
    let (mx, my) = crate::ui::theme::MODAL_SM;
    let inner = crate::ui::components::centered_rect(mx, my, full_area);

    f.render_widget(Clear, inner);

    let theme = Theme::dark_default();
    let block = Block::default()
        .title(Line::from(" Create New Branch ").alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_style(theme.accent2);

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
    let loc = app.locale();
    let theme = Theme::dark_default();

    // 1. Dim background
    crate::ui::components::apply_dimmed_backdrop(f.buffer_mut(), full_area);

    // 2. Centered Spotlight floating window (MODAL_SM token)
    let inner = crate::ui::components::centered_rect(
        crate::ui::theme::MODAL_SM.0,
        crate::ui::theme::MODAL_SM.1,
        full_area,
    );
    f.render_widget(Clear, inner);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(crate::ui::layout::SPOTLIGHT_HEADER_H),
            Constraint::Min(crate::ui::layout::SPOTLIGHT_LIST_MIN_H),
            Constraint::Length(crate::ui::layout::SPOTLIGHT_FOOTER_H),
        ])
        .split(inner);

    let b_name = app
        .get_filtered_branches()
        .get(app.ui.selected_branch_idx)
        .map(|b| b.name.as_str())
        .unwrap_or(loc.common.none);

    let is_protected = crate::actions::commands::is_protected_branch(b_name);

    let header_block = Block::default()
        .title(Line::from(vec![
            Span::styled(
                format!(" {} ", loc.branches.manage_title),
                theme.accent.add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("({}) ", b_name),
                theme.muted,
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
        .border_style(theme.active_border)
        .style(Style::default().bg(theme.surface_alt));

    let header_text = Line::from(vec![
        Span::styled(format!("  {}", loc.branches.manage_target_prefix), theme.muted),
        Span::styled(
            b_name,
            theme.accent2.add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            loc.branches.manage_instruction,
            theme.subtle,
        ),
    ]);
    f.render_widget(Paragraph::new(header_text).block(header_block), chunks[0]);

    let highlight_bg = theme.highlight.bg.unwrap_or(Color::DarkGray);
    let list_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM)
        .border_style(Style::default().fg(highlight_bg))
        .style(Style::default().bg(theme.surface));

    let success = theme.success.fg.unwrap_or(Color::Green);
    let danger = theme.danger.fg.unwrap_or(Color::Red);
    let actions = [
        (
            loc.branches.act_checkout,
            loc.branches.act_checkout_desc,
            success,
        ),
        (
            loc.branches.act_diff_ai,
            loc.branches.act_diff_ai_desc,
            theme.info.fg.unwrap_or(Color::Blue),
        ),
        (
            loc.branches.act_delete,
            loc.branches.act_delete_desc,
            danger,
        ),
        (
            loc.branches.act_rename,
            loc.branches.act_rename_desc,
            theme.warning.fg.unwrap_or(Color::Yellow),
        ),
        (
            loc.branches.act_stash,
            loc.branches.act_stash_desc,
            theme.unique.fg.unwrap_or(Color::Magenta),
        ),
        (
            loc.branches.act_help,
            loc.branches.act_help_desc,
            theme.muted.fg.unwrap_or(Color::Gray),
        ),
        (
            loc.branches.act_cancel,
            loc.branches.act_cancel_desc,
            theme.subtle.fg.unwrap_or(Color::DarkGray),
        ),
    ];

    let mut items = vec![];
    for (i, (tag, desc, tag_color)) in actions.iter().enumerate() {
        let is_selected = i == app.ui.manage_selected;
        let row_style = if is_selected {
            theme.highlight
        } else {
            theme.base
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
                theme.base
            },
        );

        let item_line = Line::from(vec![
            Span::styled(
                prefix,
                theme.accent.add_modifier(Modifier::BOLD),
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
            theme.nav.add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}   ", loc.common.select),
            theme.muted,
        ),
        Span::styled(
            "↵",
            theme.success.add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            format!(" {}   ", loc.common.execute),
            theme.muted,
        ),
        Span::styled(
            "Esc",
            theme.danger.add_modifier(Modifier::BOLD),
        ),
        Span::styled(format!(" {}", loc.common.cancel), theme.muted),
    ]);
    f.render_widget(
        Paragraph::new(footer_hints).style(Style::default().bg(theme.surface_alt)),
        chunks[2],
    );
}
