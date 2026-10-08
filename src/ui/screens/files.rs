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
use crate::ui::theme::Theme;

pub fn render_directory_searcher(f: &mut Frame, area: Rect, app: &App) {
    let theme = Theme::dark_default();
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

        if i == app.ui.selected_file_idx {
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

    let list = List::new(items).block(block);
    f.render_widget(list, sidebar_area);

    if let Some((pa, state)) = preview_area {
        render_code_preview(f, app, pa, state);
    }
}

pub fn render_code_preview(f: &mut Frame, app: &App, area: Rect, state: &PreviewState) {
    let theme = Theme::dark_default();
    let border_color = if app.ui.active_panel == FilePanel::Preview {
        theme.nav.fg.unwrap_or(Color::Blue)
    } else {
        theme.border.fg.unwrap_or(Color::DarkGray)
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
            line_style = theme.search_hit;
        } else if is_cursor {
            line_style = line_style.bg(theme.cursor_unfocused);
        } else if is_selected {
            line_style = line_style.bg(theme.select_range);
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
    f.render_widget(Clear, area);
    f.render_widget(Paragraph::new(Text::from(final_lines)).block(block), area);
}
