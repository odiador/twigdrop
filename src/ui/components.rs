use crate::models::{BranchStatus, MergeStatus};
use crate::ui::theme::Theme;
use ratatui::style::Color;

pub fn get_status_icons(status: &[BranchStatus]) -> (String, Color) {
    let theme = Theme::dark_default();
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

pub fn get_merge_status_display(status: &MergeStatus, locale: &crate::i18n::Locale) -> (String, Color) {
    let theme = Theme::dark_default();
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
                format!("{} {}/{}", bar, safe, total),
                Theme::dark_default().warning.fg.unwrap_or(Color::Yellow),
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

pub fn apply_dimmed_backdrop(buf: &mut ratatui::buffer::Buffer, area: ratatui::layout::Rect) {
    let theme = Theme::dark_default();
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            cell.set_fg(theme.backdrop_fg);
            cell.set_bg(theme.backdrop_bg);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::buffer::Buffer;
    use ratatui::layout::Rect;

    #[test]
    fn test_apply_dimmed_backdrop() {
        let area = Rect::new(0, 0, 5, 5);
        let mut buf = Buffer::empty(area);
        buf[(2, 2)].set_char('A').set_fg(Color::White).set_bg(Color::Black);

        apply_dimmed_backdrop(&mut buf, area);

        assert_eq!(buf[(2, 2)].symbol(), "A");
        assert_eq!(buf[(2, 2)].fg, Theme::dark_default().backdrop_fg);
        assert_eq!(buf[(2, 2)].bg, Theme::dark_default().backdrop_bg);
    }
}


