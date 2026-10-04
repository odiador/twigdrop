use crate::models::{BranchStatus, MergeStatus};
use ratatui::style::Color;

pub fn get_status_icons(status: &[BranchStatus]) -> (String, Color) {
    let mut icons = String::new();
    let mut color = Color::Rgb(161, 229, 193);

    if status.contains(&BranchStatus::HasUniqueCommits) {
        color = Color::Rgb(245, 194, 231);
    } else if status.contains(&BranchStatus::Gone) {
        color = Color::Rgb(140, 143, 161);
    } else if status.contains(&BranchStatus::Ahead) {
        color = Color::Rgb(249, 226, 175);
    } else if status.contains(&BranchStatus::Behind) {
        color = Color::Rgb(180, 190, 254);
    } else if status.contains(&BranchStatus::Merged) || status.contains(&BranchStatus::Stashed) {
        color = Color::Rgb(161, 229, 193);
    } else if status.contains(&BranchStatus::Local) {
        color = Color::Rgb(180, 190, 254);
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

pub fn get_merge_status_display(status: &MergeStatus) -> (String, Color) {
    match status {
        MergeStatus::NotAnalyzed => ("?".to_string(), Color::Rgb(140, 143, 161)),
        MergeStatus::Checking => ("∞ Checking".to_string(), Color::Rgb(249, 226, 175)),
        MergeStatus::Clean => ("✓ Clean".to_string(), Color::Rgb(161, 229, 193)),
        MergeStatus::Conflict(_) => ("⨯ Conflict".to_string(), Color::Rgb(245, 194, 231)),
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
                Color::Rgb(249, 226, 175),
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
    for y in area.top()..area.bottom() {
        for x in area.left()..area.right() {
            let cell = &mut buf[(x, y)];
            cell.set_fg(Color::Rgb(88, 91, 112));
            cell.set_bg(Color::Rgb(17, 17, 27));
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
        assert_eq!(buf[(2, 2)].fg, Color::Rgb(88, 91, 112));
        assert_eq!(buf[(2, 2)].bg, Color::Rgb(17, 17, 27));
    }
}


