use crate::state::ui::AppMode;
use crate::ui::theme::{MODAL_LG, MODAL_MD, MODAL_SM};
use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Spotlight pattern: header search bar / flexible list / 1-line footer hints.
/// Shared by command palette (`command_palette.rs`) and branch manage (`branches.rs`).
pub const SPOTLIGHT_HEADER_H: u16 = 3;
pub const SPOTLIGHT_LIST_MIN_H: u16 = 6;
pub const SPOTLIGHT_FOOTER_H: u16 = 1;

/// Help modal vertical slots: ASCII banner / 2-column body / footer.
pub const HELP_BANNER_H: u16 = 7;
pub const HELP_BODY_MIN_H: u16 = 12;
pub const HELP_FOOTER_H: u16 = 2;
/// Help modal two-column split (keys / views).
pub const HELP_COLS: (u16, u16) = (52, 48);

/// Commit modals (percent_x, percent_y) — same geometry as the legacy
/// two-step percentage layouts they replace.
pub const COMMIT_ACTION_MODAL: (u16, u16) = (50, 40);
pub const COMMIT_FILES_MODAL: (u16, u16) = (60, 60);
pub const REBASE_MODAL: (u16, u16) = (80, 80);
/// Generic small modal (message, switcher) geometry.
pub const SWITCHER_MODAL: (u16, u16) = (40, 50);
/// Settings modal geometry.
pub const SETTINGS_MODAL: (u16, u16) = (70, 70);
/// Command-palette spotlight geometry (same as legacy 55x52).
pub const PALETTE_MODAL: (u16, u16) = (55, 52);
/// Date-picker modal geometry (same as legacy 30/40/30 + 20/60/20).
pub const DATE_PICKER_MODAL: (u16, u16) = (60, 40);
/// Quick-actions modal geometry (same as legacy 25/50/25 + 30/40/30).
pub const QUICK_ACTIONS_MODAL: (u16, u16) = (40, 50);
/// Shell input row height.
pub const SHELL_INPUT_H: u16 = 3;
/// Shell input width (percent).
pub const SHELL_INPUT_W: u16 = 80;
/// Search bar height (single-line input + borders).
pub const SEARCH_BAR_H: u16 = 3;
/// Main-menu ASCII art slot heights: logo / gap / options / help / quit.
pub const MAIN_MENU_LOGO_H: u16 = 8;
pub const MAIN_MENU_GAP_H: u16 = 2;
pub const MAIN_MENU_OPTIONS_H: u16 = 6;
pub const MAIN_MENU_HELP_H: u16 = 5;
pub const MAIN_MENU_QUIT_H: u16 = 5;

/// Inspector drawer slots: overview / recent commits / workers.
pub const INSPECTOR_OVERVIEW_H: u16 = 7;
pub const INSPECTOR_COMMITS_MIN_H: u16 = 5;
pub const INSPECTOR_WORKERS_H: u16 = 5;

/// Sidebar slots: views / filter scopes / worktrees.
pub const SIDEBAR_VIEWS_H: u16 = 8;
pub const SIDEBAR_FILTERS_H: u16 = 6;
pub const SIDEBAR_WORKTREES_MIN_H: u16 = 4;

/// Branches view splits: table / inspector drawer / nav sidebar.
pub const INSPECTOR_SPLIT: (u16, u16) = (65, 35);
pub const SIDEBAR_SPLIT: (u16, u16) = (25, 75);
/// Stashes view splits: list / detail and files / diff.
pub const STASH_SPLIT: (u16, u16) = (30, 70);

/// Centered-rect sizes per modal (percent_x, percent_y). Single source for
/// both render and mouse hit-testing (`calculate_modal_rect` delegates here).
#[must_use]
pub const fn modal_size(mode: &AppMode) -> (u16, u16) {
    match mode {
        AppMode::Help => MODAL_LG,
        AppMode::Manage => MODAL_SM,
        AppMode::CreateBranch(_) => MODAL_SM,
        AppMode::CommandPalette(_) => PALETTE_MODAL,
        AppMode::CommitAction(_) => COMMIT_ACTION_MODAL,
        AppMode::CommitFiles(_, _) => COMMIT_FILES_MODAL,
        AppMode::InteractiveRebase => REBASE_MODAL,
        AppMode::Settings => SETTINGS_MODAL,
        AppMode::QuickActions => QUICK_ACTIONS_MODAL,
        AppMode::Switcher => SWITCHER_MODAL,
        AppMode::DatePicker(_) => DATE_PICKER_MODAL,
        _ => MODAL_MD,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffLayout {
    pub inner_area: Rect,
    pub files_list_area: Rect,
    pub ai_area: Rect,
    pub preview_area: Rect,
}

pub fn calculate_diff_layout(area: Rect) -> DiffLayout {
    let overlay_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(90),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        )
        .split(area)[1];

    let inner_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(90),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        )
        .split(overlay_area)[1];

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
        .split(inner_area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(main_chunks[0]);

    DiffLayout {
        inner_area,
        files_list_area: left_chunks[0],
        ai_area: left_chunks[1],
        preview_area: main_chunks[1],
    }
}

pub fn calculate_modal_rect(mode: &AppMode, area: Rect) -> Rect {
    let (mx, my) = modal_size(mode);
    crate::ui::components::centered_rect(mx, my, area)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modal_size_matches_render_geometry() {
        // Render functions and mouse hit-testing share this table —
        // changing a modal's size here changes both (see each render fn).
        assert_eq!(modal_size(&AppMode::Help), MODAL_LG);
        assert_eq!(modal_size(&AppMode::Manage), MODAL_SM);
        assert_eq!(modal_size(&AppMode::Filter), MODAL_MD);
        assert_eq!(
            modal_size(&AppMode::ConfirmDelete(vec![])),
            MODAL_MD
        );
        assert_eq!(
            modal_size(&AppMode::CreateBranch(String::new())),
            MODAL_SM
        );
        assert_eq!(modal_size(&AppMode::Settings), SETTINGS_MODAL);
        assert_eq!(modal_size(&AppMode::QuickActions), QUICK_ACTIONS_MODAL);
        assert_eq!(modal_size(&AppMode::Switcher), SWITCHER_MODAL);
        assert_eq!(modal_size(&AppMode::InteractiveRebase), REBASE_MODAL);
    }

    #[test]
    fn modal_rect_is_centered_and_bounded() {
        let area = Rect::new(0, 0, 200, 50);
        let r = calculate_modal_rect(&AppMode::Manage, area);
        assert!(r.width < area.width && r.height < area.height);
        assert_eq!(r.x, (area.width - r.width) / 2);
    }
}
