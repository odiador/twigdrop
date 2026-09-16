use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::state::ui::FilePanel;
use crate::ui::layout::calculate_diff_layout;

pub fn render_diff_overlay(f: &mut Frame, area: Rect, app: &App) {
    let diff_layout = calculate_diff_layout(area);
    f.render_widget(Clear, diff_layout.inner_area);

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
    f.render_widget(files_list, diff_layout.files_list_area);

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
    f.render_widget(ai_p, diff_layout.ai_area);

    // 3. File Preview (The Diff)
    if let Some(state) = &app.repo.diff_preview {
        let border_color = if app.ui.diff_panel == FilePanel::Preview {
            Color::Green
        } else {
            Color::Rgb(74, 79, 106)
        };
        let mut final_lines = Vec::new();
        let visible_rows = diff_layout.preview_area.height.saturating_sub(2) as usize;
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
        f.render_widget(diff_preview, diff_layout.preview_area);
    } else {
        let fallback_diff = Paragraph::new("Select a file to see diff").block(
            Block::default()
                .title(" Diff Preview ")
                .borders(Borders::ALL),
        );
        f.render_widget(fallback_diff, diff_layout.preview_area);
    }
}

pub fn render_diff(f: &mut Frame, app: &App) {
    let area = f.area();
    if let Some(ref state) = app.repo.diff_preview {
        super::files::render_code_preview(f, app, area, state);
    } else {
        let block = Block::default().title(" Diff View ").borders(Borders::ALL);
        f.render_widget(Paragraph::new("No diff available.").block(block), area);
    }
}
