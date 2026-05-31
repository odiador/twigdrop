use crate::events::{Event, TaskEvent};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use std::sync::Arc;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tokio::sync::mpsc;

pub fn spawn_highlight_task(
    tx: mpsc::Sender<Event>,
    file_path: String,
    lines: Vec<String>,
    ps: Arc<SyntaxSet>,
    ts: Arc<ThemeSet>,
) {
    tokio::task::spawn_blocking(move || {
        let extension = std::path::Path::new(&file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");

        let syntax = ps
            .find_syntax_by_extension(extension)
            .or_else(|| ps.find_syntax_for_file(&file_path).unwrap_or(None))
            .unwrap_or_else(|| ps.find_syntax_plain_text());

        let theme = &ts.themes["base16-ocean.dark"];
        let mut h = syntect::easy::HighlightLines::new(syntax, theme);

        let mut highlighted_lines = Vec::with_capacity(lines.len());

        for line in &lines {
            let line_with_ending = format!("{}\n", line);
            let ranges = h.highlight_line(&line_with_ending, &ps).unwrap_or_default();
            let mut spans = Vec::new();

            for (style, text) in ranges {
                let color = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                let content = text.trim_end_matches(['\n', '\r']);
                if !content.is_empty() || text.is_empty() {
                    spans.push(Span::styled(
                        content.to_string(),
                        Style::default().fg(color),
                    ));
                }
            }
            highlighted_lines.push(Line::from(spans));
        }

        let _ = tx.blocking_send(Event::Task(TaskEvent::HighlightingComplete(
            file_path,
            highlighted_lines,
        )));
    });
}
