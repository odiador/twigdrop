use super::App;
use crate::state::ui::PreviewState;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};

impl App {
    pub fn create_preview_state(&self, repo_path: &str, rel_path: &str) -> Option<PreviewState> {
        let full_path = std::path::Path::new(repo_path).join(rel_path);

        let file = std::fs::File::open(&full_path).ok()?;
        let reader = std::io::BufReader::new(file);
        use std::io::BufRead;

        let mut lines = Vec::new();
        let max_lines = 5000;
        for (i, line) in reader.lines().enumerate() {
            if i >= max_lines {
                break;
            }
            if let Ok(l) = line {
                lines.push(l);
            }
        }

        let line_diffs = crate::git::get_line_diffs(repo_path, rel_path);
        let line_blames = crate::git::blame::get_file_blame(repo_path, rel_path);

        let mut state = PreviewState {
            file_path: rel_path.to_string(),
            lines,
            highlighted_lines: Vec::new(),
            cursor_y: 0,
            scroll_y: 0,
            selection_start: None,
            selection_end: None,
            line_diffs,
            line_blames,
            show_blame: false,
        };

        self.update_preview_highlighting(&mut state);

        Some(state)
    }

    pub fn update_preview_highlighting(&self, state: &mut PreviewState) {
        if state.lines.is_empty() {
            return;
        }

        if let Some(tx) = &self.event_tx {
            crate::tasks::highlighting::spawn_highlight_task(
                tx.clone(),
                state.file_path.clone(),
                state.lines.clone(),
                self.ps.clone(),
                self.ts.clone(),
            );
        } else {
            let extension = std::path::Path::new(&state.file_path)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let syntax = self
                .ps
                .find_syntax_by_extension(extension)
                .or_else(|| {
                    self.ps
                        .find_syntax_for_file(&state.file_path)
                        .unwrap_or(None)
                })
                .unwrap_or_else(|| self.ps.find_syntax_plain_text());

            let theme = &self.ts.themes["base16-ocean.dark"];
            let mut h = syntect::easy::HighlightLines::new(syntax, theme);

            state.highlighted_lines.clear();
            for line in &state.lines {
                let line_with_ending = format!("{line}\n");
                let ranges = h
                    .highlight_line(&line_with_ending, &self.ps)
                    .unwrap_or_default();
                let mut spans = Vec::new();

                for (style, text) in ranges {
                    let color =
                        Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                    let content = text.trim_end_matches(['\n', '\r']);
                    if !content.is_empty() || text.is_empty() {
                        spans.push(Span::styled(
                            content.to_string(),
                            Style::default().fg(color),
                        ));
                    }
                }
                state.highlighted_lines.push(Line::from(spans));
            }
        }
    }

    pub fn update_diff_highlighting(&self, state: &mut PreviewState) {
        let theme = self.theme();
        state.highlighted_lines.clear();
        for line in &state.lines {
            let color = if line.starts_with('+') && !line.starts_with("+++") {
                theme.muted.fg.unwrap_or(Color::Green)
            } else if line.starts_with("+++") || line.starts_with("---") {
                theme.accent2.fg.unwrap_or(Color::Cyan)
            } else {
                theme.base.fg.unwrap_or(Color::White)
            };

            let spans = vec![Span::styled(line.to_string(), Style::default().fg(color))];

            if line.starts_with("@@") {
                state.highlighted_lines.push(Line::from(vec![Span::styled(
                    " ".repeat(100),
                    Style::default().bg(theme.surface),
                )]));
            }

            state.highlighted_lines.push(Line::from(spans));
        }
    }
}
