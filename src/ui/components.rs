use crate::models::{BranchStatus, MergeStatus};
use ratatui::style::Color;

pub fn get_status_icons(status: &[BranchStatus]) -> (String, Color) {
    let mut icons = String::new();
    let mut color = Color::Rgb(161, 229, 193);

    if status.contains(&BranchStatus::HasUniqueCommits) { color = Color::Rgb(245, 194, 231); }
    else if status.contains(&BranchStatus::Gone) { color = Color::Rgb(140, 143, 161); }
    else if status.contains(&BranchStatus::Ahead) { color = Color::Rgb(249, 226, 175); }
    else if status.contains(&BranchStatus::Behind) { color = Color::Rgb(180, 190, 254); }
    else if status.contains(&BranchStatus::Merged) || status.contains(&BranchStatus::Stashed) { color = Color::Rgb(161, 229, 193); }
    else if status.contains(&BranchStatus::Local) { color = Color::Rgb(180, 190, 254); }

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
            let filled = if total_f > 0.0 { ((safe_f / total_f) * bar_len as f32).round() as usize } else { 0 };
            let mut bar = String::new();
            for _ in 0..filled { bar.push('█'); }
            for _ in filled..bar_len { bar.push('░'); }
            (format!("{} {}/{}", bar, safe, total), Color::Rgb(249, 226, 175))
        }
    }
}
