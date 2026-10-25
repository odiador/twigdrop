use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::git::files::FileStatus;
use crate::models::GutterStatus;
use crate::state::ui::{AppMode, FilePanel, PreviewState};

pub fn render_directory_searcher(f: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
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
        theme.nav.fg.unwrap_or(Color::Blue)
    } else {
        theme.border.fg.unwrap_or(Color::DarkGray)
    };

    let title_str = if app.ui.files_search_active {
        format!(
            " 📂 Search: {}_ [Enter: done • Esc: cancel] ",
            app.ui.files_search_query
        )
    } else if !app.ui.files_search_query.is_empty() {
        format!(
            " 📂 Files [Filter: \"{}\"] [Esc: clear] ",
            app.ui.files_search_query
        )
    } else {
        let mut badges = Vec::new();
        if app.ui.files_changed_only {
            badges.push("CHANGED ONLY");
        }
        if app.ui.show_hidden_files {
            badges.push("+DOTFILES");
        }
        let badge_str = if badges.is_empty() {
            String::new()
        } else {
            format!(" [{}]", badges.join(" • "))
        };
        format!(" 📂 Files{badge_str} (.: dotfiles • c: changed • Space: stage • /: search) ")
    };

    let block = Block::default()
        .title(Line::from(title_str).alignment(Alignment::Left))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(sidebar_border_color));

    let mut items = vec![];
    for (display_idx, &tree_idx) in app.ui.filtered_file_indices.iter().enumerate() {
        let entry = &app.repo.file_tree[tree_idx];
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
            FileStatus::Modified => theme.warning.fg.unwrap_or(Color::Yellow),
            FileStatus::Added => theme.merged.fg.unwrap_or(Color::Green),
            FileStatus::Staged => theme.info.fg.unwrap_or(Color::Blue),
            FileStatus::Untracked => theme.unique.fg.unwrap_or(Color::Magenta),
            FileStatus::Ignored => theme.dim.fg.unwrap_or(Color::Gray),
            FileStatus::Deleted => theme.danger.fg.unwrap_or(Color::Red),
            FileStatus::Conflict => theme.conflict.fg.unwrap_or(Color::Red),
            FileStatus::Normal => theme.base.fg.unwrap_or(Color::White),
        };

        let mut style = Style::default().fg(status_color);

        let status_bg = match entry.status {
            FileStatus::Modified => Some(theme.tint_modified),
            FileStatus::Added => Some(theme.tint_added),
            FileStatus::Conflict => Some(theme.tint_conflict),
            FileStatus::Staged => Some(theme.tint_staged),
            _ => None,
        };

        if let Some(bg) = status_bg {
            style = style.bg(bg);
        }

        if display_idx == app.ui.selected_file_idx {
            let bg = if app.ui.active_panel == FilePanel::Directory {
                Color::White
            } else {
                theme.select_soft
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

    if items.is_empty() {
        let msg = if app.ui.files_changed_only {
            "No changed files in workspace"
        } else if !app.ui.files_search_query.is_empty() {
            "No files matching search"
        } else {
            "No files found"
        };
        items.push(ListItem::new(Line::from(vec![
            Span::raw("  "),
            Span::styled(msg, theme.dim),
        ])));
    }

    let list = List::new(items).block(block);
    f.render_widget(list, sidebar_area);

    if let Some((pa, state)) = preview_area {
        render_code_preview(f, app, pa, state);
    }
}

pub fn render_code_preview(f: &mut Frame, app: &App, area: Rect, state: &PreviewState) {
    let theme = app.theme();
    let border_color = if app.ui.active_panel == FilePanel::Preview {
        theme.nav.fg.unwrap_or(Color::Blue)
    } else {
        theme.border.fg.unwrap_or(Color::DarkGray)
    };

    let (header_area, body_area) = if area.height >= 10 {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);
        (Some(chunks[0]), chunks[1])
    } else {
        (None, area)
    };

    if let Some(ha) = header_area {
        let header_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color));
        let inner = header_block.inner(ha);
        f.render_widget(header_block, ha);

        let parts: Vec<&str> = state.file_path.split('/').collect();
        let mut breadcrumb_spans = vec![Span::styled("📁 ", theme.accent2)];

        for (idx, part) in parts.iter().enumerate() {
            if idx + 1 == parts.len() {
                breadcrumb_spans.push(Span::styled(
                    (*part).to_string(),
                    theme.base.add_modifier(ratatui::style::Modifier::BOLD),
                ));
            } else {
                breadcrumb_spans.push(Span::styled((*part).to_string(), theme.soft));
                breadcrumb_spans.push(Span::styled(" › ", theme.subtle));
            }
        }

        breadcrumb_spans.push(Span::styled(
            format!(
                "  [Ln {}/{} │ {} lines] ",
                state.cursor_y + 1,
                state.lines.len(),
                state.lines.len()
            ),
            theme.peach,
        ));

        breadcrumb_spans.push(Span::styled(
            " (Tab: focus directory │ j/k: scroll) ",
            theme.subtle,
        ));

        let p = Paragraph::new(Line::from(breadcrumb_spans));
        f.render_widget(p, inner);
    }

    let block_title = format!(" Preview: {} ", state.file_path);

    let block = Block::default()
        .title(block_title)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color));

    let mut final_lines = Vec::new();
    let visible_rows = body_area.height.saturating_sub(2) as usize;
    let start_idx = state.scroll_y;
    const OVERSCAN_LINES: usize = 5;
    let end_idx = (start_idx + visible_rows + OVERSCAN_LINES).min(state.highlighted_lines.len());

    for i in start_idx..end_idx {
        let h_line = &state.highlighted_lines[i];
        let mut spans = vec![Span::styled(
            format!("{:>4} │ ", i + 1),
            Style::default().fg(Color::DarkGray),
        )];
        let gutter_diff = state.line_diffs.get(&i);
        spans.push(match gutter_diff {
            Some(GutterStatus::Added) => Span::styled("+ ", Style::default().fg(theme.diff_add_fg)),
            Some(GutterStatus::Modified) => Span::styled("| ", Style::default().fg(Color::Cyan)),
            Some(GutterStatus::Deleted) => {
                Span::styled("~ ", Style::default().fg(theme.diff_del_fg))
            }
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
            line_style = theme.search_hit;
        } else if is_cursor {
            line_style = line_style.bg(theme.cursor_unfocused);
        } else if is_selected {
            line_style = line_style.bg(theme.select_range);
        } else if let Some(diff) = gutter_diff {
            match diff {
                GutterStatus::Added => line_style = line_style.bg(theme.diff_add_bg),
                GutterStatus::Deleted => line_style = line_style.bg(theme.diff_del_bg),
                GutterStatus::Modified => line_style = line_style.bg(theme.tint_modified),
            }
        }

        for span in &h_line.spans {
            let mut s = span.style;
            if is_cursor && app.ui.active_panel == FilePanel::Preview {
                s = theme.search_hit;
            } else if is_cursor {
                s = s.bg(theme.cursor_unfocused);
            } else if is_selected {
                s = s.bg(theme.select_range);
            }
            spans.push(Span::styled(span.content.clone(), s));
        }
        final_lines.push(Line::from(spans).style(line_style));
    }
    f.render_widget(Clear, body_area);
    f.render_widget(
        Paragraph::new(Text::from(final_lines)).block(block),
        body_area,
    );
}
