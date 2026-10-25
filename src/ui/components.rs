use crate::models::{BranchStatus, MergeStatus};
use crate::ui::theme::Theme;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

pub fn get_status_icons(status: &[BranchStatus], theme: &Theme) -> (String, Color) {
    let mut icons = String::new();
    let mut color = theme.merged.fg.unwrap_or(Color::Green);

    if status.contains(&BranchStatus::HasUniqueCommits) {
        color = theme.unique.fg.unwrap_or(color);
    } else if status.contains(&BranchStatus::Gone) {
        color = theme.dim.fg.unwrap_or(color);
    } else if status.contains(&BranchStatus::Ahead) {
        color = theme.warning.fg.unwrap_or(color);
    } else if status.contains(&BranchStatus::Behind) {
        color = theme.nav.fg.unwrap_or(color);
    } else if status.contains(&BranchStatus::Merged) || status.contains(&BranchStatus::Stashed) {
        color = theme.merged.fg.unwrap_or(color);
    } else if status.contains(&BranchStatus::Local) {
        color = theme.nav.fg.unwrap_or(color);
    }

    for s in status {
        match s {
            BranchStatus::HasUniqueCommits => icons.push('▲'),
            BranchStatus::Gone => icons.push('⨯'),
            BranchStatus::Ahead => icons.push('↑'),
            BranchStatus::Behind => icons.push('↓'),
            BranchStatus::Merged => icons.push('✓'),
            BranchStatus::Local => icons.push('L'),
            BranchStatus::Stashed => icons.push('S'),
            BranchStatus::Safe => icons.push('●'),
            BranchStatus::RemoteTracked => icons.push('R'),
            BranchStatus::RemoteUntracked => icons.push('U'),
        }
    }
    (icons, color)
}

pub fn get_merge_status_display(
    status: &MergeStatus,
    locale: &crate::i18n::Locale,
    theme: &Theme,
) -> (String, Color) {
    let dim = theme.dim.fg.unwrap_or(Color::Gray);
    let warning = theme.warning.fg.unwrap_or(Color::Yellow);
    let merged = theme.merged.fg.unwrap_or(Color::Green);
    let unique = theme.unique.fg.unwrap_or(Color::Magenta);
    match status {
        MergeStatus::NotAnalyzed => (locale.branches.merge_not_analyzed.to_string(), dim),
        MergeStatus::Checking => (locale.branches.merge_checking.to_string(), warning),
        MergeStatus::Clean => (locale.branches.merge_clean.to_string(), merged),
        MergeStatus::Conflict(_) => (locale.branches.merge_conflict.to_string(), unique),
        MergeStatus::SafeLimit(safe, total) => {
            let total_f = *total as f32;
            let safe_f = *safe as f32;
            let bar_len = 10;
            let filled = if total_f > 0.0 {
                ((safe_f / total_f) * bar_len as f32).round() as usize
            } else {
                0
            };
            let mut bar = String::new();
            for _ in 0..filled {
                bar.push('█');
            }
            for _ in filled..bar_len {
                bar.push('░');
            }
            (
                format!("{bar} {safe}/{total}"),
                theme.warning.fg.unwrap_or(Color::Yellow),
            )
        }
    }
}

pub fn centered_rect(
    percent_x: u16,
    percent_y: u16,
    r: ratatui::layout::Rect,
) -> ratatui::layout::Rect {
    let popup_layout = ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Vertical)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
            ratatui::layout::Constraint::Percentage(percent_y),
            ratatui::layout::Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    ratatui::layout::Layout::default()
        .direction(ratatui::layout::Direction::Horizontal)
        .constraints([
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
            ratatui::layout::Constraint::Percentage(percent_x),
            ratatui::layout::Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn apply_dimmed_backdrop(
    buf: &mut ratatui::buffer::Buffer,
    area: ratatui::layout::Rect,
    theme: &Theme,
) {
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            cell.set_fg(theme.backdrop_fg);
            cell.set_bg(theme.backdrop_bg);
        }
    }
}

pub fn parse_hunk_header(line: &str) -> Option<(usize, usize)> {
    if !line.starts_with("@@") {
        return None;
    }
    let parts: Vec<&str> = line.split("@@").collect();
    if parts.len() < 3 {
        return None;
    }
    let range_str = parts[1].trim();
    let mut tokens = range_str.split_whitespace();
    let old_token = tokens.next()?;
    let new_token = tokens.next()?;

    let parse_start = |token: &str, prefix: char| -> Option<usize> {
        let s = token.strip_prefix(prefix)?;
        let start_str = s.split(',').next()?;
        start_str.parse::<usize>().ok()
    };

    let old_start = parse_start(old_token, '-')?;
    let new_start = parse_start(new_token, '+')?;
    Some((old_start, new_start))
}

pub fn format_diff_with_gutters<'a>(diff: &str, theme: &Theme) -> Vec<Line<'a>> {
    if diff.trim().is_empty() {
        return vec![Line::from(Span::styled(
            " (No diff for this commit/file) ",
            theme.dim,
        ))];
    }

    let mut lines = Vec::new();
    let mut old_lineno = 1usize;
    let mut new_lineno = 1usize;
    let mut in_hunk = false;

    let green = theme.merged.fg.unwrap_or(Color::Green);
    let red = theme.danger.fg.unwrap_or(Color::Red);
    let cyan = theme.nav.fg.unwrap_or(Color::Cyan);

    for line in diff.lines() {
        if line.starts_with("@@") {
            if let Some((old_start, new_start)) = parse_hunk_header(line) {
                old_lineno = old_start;
                new_lineno = new_start;
                in_hunk = true;
            }
            lines.push(Line::from(vec![
                Span::styled("───────┼── ", theme.dim),
                Span::styled(
                    line.to_string(),
                    Style::default().fg(cyan).add_modifier(Modifier::BOLD),
                ),
            ]));
        } else if !in_hunk
            || line.starts_with("diff --git")
            || line.starts_with("index ")
            || line.starts_with("--- ")
            || line.starts_with("+++ ")
        {
            lines.push(Line::from(vec![
                Span::styled("       │   ", theme.dim),
                Span::styled(line.to_string(), theme.dim),
            ]));
        } else if let Some(stripped) = line.strip_prefix('+') {
            lines.push(Line::from(vec![
                Span::styled("     ", theme.dim),
                Span::styled(format!("{:>4} ", new_lineno), green),
                Span::styled("│ ", theme.dim),
                Span::styled("+ ", green),
                Span::styled(stripped.to_string(), green),
            ]));
            new_lineno += 1;
        } else if let Some(stripped) = line.strip_prefix('-') {
            lines.push(Line::from(vec![
                Span::styled(format!("{:>4} ", old_lineno), red),
                Span::styled("     ", theme.dim),
                Span::styled("│ ", theme.dim),
                Span::styled("- ", red),
                Span::styled(stripped.to_string(), red),
            ]));
            old_lineno += 1;
        } else if let Some(stripped) = line.strip_prefix(' ') {
            lines.push(Line::from(vec![
                Span::styled(format!("{:>4} ", old_lineno), theme.dim),
                Span::styled(format!("{:>4} ", new_lineno), theme.dim),
                Span::styled("│   ", theme.dim),
                Span::styled(stripped.to_string(), theme.base),
            ]));
            old_lineno += 1;
            new_lineno += 1;
        } else {
            lines.push(Line::from(vec![
                Span::styled("       │   ", theme.dim),
                Span::styled(line.to_string(), theme.dim),
            ]));
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn test_apply_dimmed_backdrop() {
        let theme = Theme::dark_default();
        let area = Rect::new(0, 0, 5, 5);
        let mut buf = Buffer::empty(area);
        buf[(2, 2)]
            .set_char('A')
            .set_fg(Color::White)
            .set_bg(Color::Black);

        apply_dimmed_backdrop(&mut buf, area, &theme);

        assert_eq!(buf[(2, 2)].symbol(), "A");
        assert_eq!(buf[(2, 2)].fg, theme.backdrop_fg);
        assert_eq!(buf[(2, 2)].bg, theme.backdrop_bg);
    }

    #[test]
    fn test_light_theme_differs_from_dark() {
        let dark = Theme::dark_default();
        let light = Theme::light();
        assert_ne!(dark.base, light.base);
        assert_ne!(dark.surface, light.surface);
        assert_ne!(dark.success, light.success);
    }

    #[test]
    fn test_parse_hunk_header_variants() {
        assert_eq!(
            parse_hunk_header("@@ -42,10 +42,12 @@ format:"),
            Some((42, 42))
        );
        assert_eq!(parse_hunk_header("@@ -1 +1 @@"), Some((1, 1)));
        assert_eq!(parse_hunk_header("@@ -0,0 +1,5 @@"), Some((0, 1)));
        assert_eq!(parse_hunk_header("not a hunk"), None);
        assert_eq!(parse_hunk_header("@@ malformed @@"), None);
    }

    #[test]
    fn test_format_diff_with_gutters() {
        let theme = Theme::dark_default();
        let empty_diff = format_diff_with_gutters("", &theme);
        assert_eq!(empty_diff.len(), 1);

        let diff_sample = "diff --git a/foo b/foo\n@@ -1,2 +1,2 @@\n-old\n+new\n context";
        let lines = format_diff_with_gutters(diff_sample, &theme);
        assert_eq!(lines.len(), 5);
    }
}
