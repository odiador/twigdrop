//! Markdown rendering engine for terminal display in Ratatui.
//!
//! Converts markdown text (common in PR descriptions, release notes, and documentation)
//! into styled Ratatui `Line` vectors with visual hierarchy, headers, lists, code blocks,
//! quotes, and inline markup (bold, italic, code, links).

use crate::ui::theme::Theme;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
};

/// Parses inline markdown formatting (bold, italic, inline code, links, strikethrough)
/// and returns a sequence of styled `Span`s.
#[must_use]
pub fn parse_inline(text: &str, theme: &Theme, base_style: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut chars = text.char_indices().peekable();
    let mut current_text = String::new();

    while let Some(&(i, c)) = chars.peek() {
        // 1. Inline code: `code`
        if c == '`' {
            if !current_text.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut current_text), base_style));
            }
            chars.next(); // consume opening `
            let mut code = String::new();
            let mut closed = false;
            for (_, c2) in chars.by_ref() {
                if c2 == '`' {
                    closed = true;
                    break;
                }
                code.push(c2);
            }
            if closed {
                let code_style = Style::default()
                    .fg(theme.accent2.fg.unwrap_or(ratatui::style::Color::Cyan))
                    .bg(theme.surface)
                    .add_modifier(Modifier::BOLD);
                spans.push(Span::styled(format!(" {} ", code), code_style));
            } else {
                current_text.push('`');
                current_text.push_str(&code);
            }
            continue;
        }

        let remaining = &text[i..];

        // 2. Links: [text](url)
        if c == '['
            && let Some(close_bracket) = remaining.find(']')
            && let Some(after_bracket) = remaining[close_bracket..].strip_prefix("](")
            && let Some(close_paren) = after_bracket.find(')')
        {
            let link_text = &remaining[1..close_bracket];
            let link_url = &after_bracket[..close_paren];
            if !current_text.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut current_text), base_style));
            }
            let link_style = Style::default()
                .fg(theme.accent2.fg.unwrap_or(ratatui::style::Color::Cyan))
                .add_modifier(Modifier::UNDERLINED);
            spans.push(Span::styled(link_text.to_string(), link_style));
            spans.push(Span::styled(format!(" ({})", link_url), theme.subtle));

            let total_len = close_bracket + 2 + close_paren + 1;
            let char_count = remaining[..total_len].chars().count();
            for _ in 0..char_count {
                chars.next();
            }
            continue;
        }

        // 3. Bold: ** or __
        if remaining.starts_with("**") || remaining.starts_with("__") {
            let delim = &remaining[..2];
            if let Some(end_idx) = remaining[2..].find(delim) {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), base_style));
                }
                let bold_text = &remaining[2..2 + end_idx];
                let bold_style = base_style.add_modifier(Modifier::BOLD);
                spans.push(Span::styled(bold_text.to_string(), bold_style));

                let total_len = 2 + end_idx + 2;
                let char_count = remaining[..total_len].chars().count();
                for _ in 0..char_count {
                    chars.next();
                }
                continue;
            }
        }

        // 4. Strikethrough: ~~
        if let Some(after_tilde) = remaining.strip_prefix("~~")
            && let Some(end_idx) = after_tilde.find("~~")
        {
            if !current_text.is_empty() {
                spans.push(Span::styled(std::mem::take(&mut current_text), base_style));
            }
            let strike_text = &after_tilde[..end_idx];
            let strike_style = base_style.add_modifier(Modifier::CROSSED_OUT);
            spans.push(Span::styled(strike_text.to_string(), strike_style));

            let total_len = 2 + end_idx + 2;
            let char_count = remaining[..total_len].chars().count();
            for _ in 0..char_count {
                chars.next();
            }
            continue;
        }

        // 5. Italic: * or _
        if (c == '*' || c == '_') && remaining.len() > 1 && !remaining[1..].starts_with(c) {
            let delim = c;
            if let Some(end_idx) = remaining[1..].find(delim) {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), base_style));
                }
                let italic_text = &remaining[1..1 + end_idx];
                let italic_style = base_style.add_modifier(Modifier::ITALIC);
                spans.push(Span::styled(italic_text.to_string(), italic_style));

                let total_len = 1 + end_idx + 1;
                let char_count = remaining[..total_len].chars().count();
                for _ in 0..char_count {
                    chars.next();
                }
                continue;
            }
        }

        chars.next();
        current_text.push(c);
    }

    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, base_style));
    }

    spans
}

/// Renders a multi-line markdown string into a styled `Vec<Line<'static>>` suitable
/// for Ratatui Paragraph rendering.
#[must_use]
pub fn render_markdown(text: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    let mut in_code_block = false;
    let mut code_lang = String::new();
    let mut last_was_empty = false;

    for raw_line in text.lines() {
        let line = raw_line.trim_end();

        // 1. Code blocks (```lang or ```)
        if line.trim_start().starts_with("```") {
            if in_code_block {
                lines.push(Line::from(vec![
                    Span::styled("  └", theme.subtle),
                    Span::styled("─".repeat(40), theme.subtle),
                ]));
                in_code_block = false;
                code_lang.clear();
            } else {
                let lang = line.trim_start().trim_start_matches('`').trim();
                code_lang = lang.to_string();
                let lang_display = if code_lang.is_empty() {
                    "code".to_string()
                } else {
                    code_lang.clone()
                };
                lines.push(Line::from(vec![
                    Span::styled("  ┌─ ", theme.subtle),
                    Span::styled(lang_display, theme.accent2.add_modifier(Modifier::BOLD)),
                    Span::styled(" ", theme.subtle),
                    Span::styled("─".repeat(30), theme.subtle),
                ]));
                in_code_block = true;
            }
            last_was_empty = false;
            continue;
        }

        if in_code_block {
            let spans = vec![
                Span::styled("  │ ", theme.subtle),
                Span::styled(line.to_string(), theme.base),
            ];
            let mut l = Line::from(spans);
            l = l.style(Style::default().bg(theme.surface));
            lines.push(l);
            last_was_empty = false;
            continue;
        }

        // Empty lines
        if line.trim().is_empty() {
            if !last_was_empty {
                lines.push(Line::default());
                last_was_empty = true;
            }
            continue;
        }
        last_was_empty = false;

        // 2. Horizontal rules
        let trimmed = line.trim();
        if trimmed == "---"
            || trimmed == "***"
            || trimmed == "___"
            || (trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-'))
        {
            lines.push(Line::from(Span::styled(
                "  ──────────────────────────────────────────",
                theme.border,
            )));
            continue;
        }

        // 3. Headings
        if let Some(rest) = trimmed.strip_prefix("# ") {
            if !lines.is_empty() {
                lines.push(Line::default());
            }
            lines.push(Line::from(vec![
                Span::styled("  # ", theme.accent.add_modifier(Modifier::BOLD)),
                Span::styled(rest.to_string(), theme.base.add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(Span::styled(
                "  ══════════════════════════════════════════",
                theme.accent.add_modifier(Modifier::DIM),
            )));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            if !lines.is_empty() {
                lines.push(Line::default());
            }
            lines.push(Line::from(vec![
                Span::styled("  ## ", theme.accent2.add_modifier(Modifier::BOLD)),
                Span::styled(rest.to_string(), theme.base.add_modifier(Modifier::BOLD)),
            ]));
            lines.push(Line::from(Span::styled(
                "  ──────────────────────────────────────────",
                theme.border_soft,
            )));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("### ") {
            lines.push(Line::from(vec![
                Span::styled("  ### ", theme.peach.add_modifier(Modifier::BOLD)),
                Span::styled(rest.to_string(), theme.base.add_modifier(Modifier::BOLD)),
            ]));
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("#### ") {
            lines.push(Line::from(vec![
                Span::styled("  #### ", theme.subtle),
                Span::styled(rest.to_string(), theme.muted.add_modifier(Modifier::BOLD)),
            ]));
            continue;
        }

        // 4. Blockquotes (> ...)
        if let Some(rest) = trimmed.strip_prefix('>') {
            let quote_content = rest.trim_start();
            let mut spans = vec![Span::styled("  │ ", theme.accent)];
            spans.extend(parse_inline(
                quote_content,
                theme,
                theme.muted.add_modifier(Modifier::ITALIC),
            ));
            lines.push(Line::from(spans));
            continue;
        }

        // 5. Task lists: - [ ] or - [x]
        if let Some(rest) = trimmed
            .strip_prefix("- [ ] ")
            .or_else(|| trimmed.strip_prefix("* [ ] "))
        {
            let mut spans = vec![Span::styled("  [ ] ", theme.subtle)];
            spans.extend(parse_inline(rest, theme, theme.base));
            lines.push(Line::from(spans));
            continue;
        }
        if let Some(rest) = trimmed
            .strip_prefix("- [x] ")
            .or_else(|| trimmed.strip_prefix("* [x] "))
            .or_else(|| trimmed.strip_prefix("- [X] "))
            .or_else(|| trimmed.strip_prefix("* [X] "))
        {
            let mut spans = vec![Span::styled(
                "  [✓] ",
                theme.success.add_modifier(Modifier::BOLD),
            )];
            spans.extend(parse_inline(rest, theme, theme.muted));
            lines.push(Line::from(spans));
            continue;
        }

        // 6. Bullet lists: - or * or +
        if let Some(rest) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| trimmed.strip_prefix("+ "))
        {
            let indent_spaces = line.len() - line.trim_start().len();
            let indent_str = " ".repeat(indent_spaces + 2);
            let mut spans = vec![Span::styled(format!("{}• ", indent_str), theme.accent2)];
            spans.extend(parse_inline(rest, theme, theme.base));
            lines.push(Line::from(spans));
            continue;
        }

        // 7. Numbered lists: 1. or 2. etc.
        let mut is_num_list = false;
        if let Some(dot_idx) = trimmed.find(". ") {
            let prefix = &trimmed[..dot_idx];
            if !prefix.is_empty() && prefix.chars().all(|c| c.is_ascii_digit()) {
                let rest = &trimmed[dot_idx + 2..];
                let indent_spaces = line.len() - line.trim_start().len();
                let indent_str = " ".repeat(indent_spaces + 2);
                let mut spans = vec![Span::styled(
                    format!("{}{}. ", indent_str, prefix),
                    theme.accent2,
                )];
                spans.extend(parse_inline(rest, theme, theme.base));
                lines.push(Line::from(spans));
                is_num_list = true;
            }
        }
        if is_num_list {
            continue;
        }

        // 8. Regular text paragraph line
        let mut spans = vec![Span::raw("  ")];
        spans.extend(parse_inline(trimmed, theme, theme.base));
        lines.push(Line::from(spans));
    }

    if in_code_block {
        lines.push(Line::from(vec![
            Span::styled("  └", theme.subtle),
            Span::styled("─".repeat(40), theme.subtle),
        ]));
    }

    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inline_bold_and_code() {
        let theme = Theme::dark_default();
        let spans = parse_inline("This is **bold** and `code` here", &theme, Style::default());
        assert_eq!(spans.len(), 5);
        assert_eq!(spans[0].content, "This is ");
        assert_eq!(spans[1].content, "bold");
        assert!(spans[1].style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(spans[2].content, " and ");
        assert_eq!(spans[3].content, " code ");
        assert_eq!(spans[4].content, " here");
    }

    #[test]
    fn test_parse_inline_links() {
        let theme = Theme::dark_default();
        let spans = parse_inline(
            "Check [Twigdrop](https://github.com/odiador/twigdrop) now",
            &theme,
            Style::default(),
        );
        assert_eq!(spans.len(), 4);
        assert_eq!(spans[0].content, "Check ");
        assert_eq!(spans[1].content, "Twigdrop");
        assert!(spans[1].style.add_modifier.contains(Modifier::UNDERLINED));
        assert_eq!(spans[2].content, " (https://github.com/odiador/twigdrop)");
        assert_eq!(spans[3].content, " now");
    }

    #[test]
    fn test_render_markdown_headings_and_tasks() {
        let theme = Theme::dark_default();
        let md = r#"# Pull Request 42
A simple description.

## Changes
- [x] Implemented markdown renderer
- [ ] Add tests

```rust
fn test() {
    println!("hello");
}
```
"#;
        let lines = render_markdown(md, &theme);
        assert!(!lines.is_empty());
        // Verify we have headings, dividers, tasks, and code block
        let text_repr: Vec<String> = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect();

        assert!(text_repr.iter().any(|l| l.contains("# Pull Request 42")));
        assert!(
            text_repr
                .iter()
                .any(|l| l.contains("[✓] Implemented markdown renderer"))
        );
        assert!(text_repr.iter().any(|l| l.contains("[ ] Add tests")));
        assert!(text_repr.iter().any(|l| l.contains("fn test()")));
    }

    #[test]
    fn test_render_markdown_blockquotes_and_lists() {
        let theme = Theme::dark_default();
        let md = r#"> Important note from reviewer
1. First point
2. Second point
* An unordered bullet
---
~~old text~~ and *italic text*
"#;
        let lines = render_markdown(md, &theme);
        let text_repr: Vec<String> = lines
            .iter()
            .map(|l| {
                l.spans
                    .iter()
                    .map(|s| s.content.as_ref())
                    .collect::<String>()
            })
            .collect();

        assert!(
            text_repr
                .iter()
                .any(|l| l.contains("│") && l.contains("Important note"))
        );
        assert!(
            text_repr
                .iter()
                .any(|l| l.contains("1. ") && l.contains("First point"))
        );
        assert!(
            text_repr
                .iter()
                .any(|l| l.contains("• ") && l.contains("An unordered bullet"))
        );
        assert!(text_repr.iter().any(|l| l.contains("────")));
        assert!(
            text_repr
                .iter()
                .any(|l| l.contains("old text") && l.contains("italic text"))
        );
    }
}
