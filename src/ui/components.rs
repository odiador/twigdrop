use ratatui::style::Color;
use crate::models::BranchStatus;

pub fn get_status_icons(status: &[BranchStatus]) -> (String, Color) {
    let mut icons = String::new();
    let mut color = Color::Green;

    if status.contains(&BranchStatus::HasUniqueCommits) {
        color = Color::Red;
    } else if status.contains(&BranchStatus::Gone) {
        color = Color::DarkGray;
    } else if status.contains(&BranchStatus::Ahead) {
        color = Color::Yellow;
    } else if status.contains(&BranchStatus::Behind) {
        color = Color::Cyan;
    } else if status.contains(&BranchStatus::Merged) {
        color = Color::Blue;
    } else if status.contains(&BranchStatus::Stashed) {
        color = Color::Green;
    } else if status.contains(&BranchStatus::Local) {
        color = Color::Magenta;
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
        }
    }
    (icons, color)
}
