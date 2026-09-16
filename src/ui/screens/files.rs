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
