use crate::models::{BranchStatus, GutterStatus};
use crate::ui::animations::SnapAnimation;
use std::collections::HashSet;
use std::time::Instant;

#[derive(Clone, Debug, PartialEq)]
pub struct PreviewState {
    pub file_path: String,
    pub lines: Vec<String>,
    pub highlighted_lines: Vec<ratatui::text::Line<'static>>,
    pub cursor_y: usize,
    pub scroll_y: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub line_diffs: std::collections::HashMap<usize, GutterStatus>,
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum FilePanel {
    #[default]
    Directory,
    Preview,
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum BranchSortMode {
    #[default]
    Recent,
    PrunableFirst,
    Alphabetical,
}

#[derive(PartialEq, Debug, Clone)]
pub enum DatePickerField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
}

#[derive(PartialEq, Debug, Clone)]
pub struct DatePickerState {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub active_field: DatePickerField,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Normal,
    Help,
    Manage,
    Filter,
    StashDetail,
    Settings,
    Search,
    Diff,
    CodePreview(PreviewState),
    ConfirmDelete(Vec<String>),
    CreateBranch(String),
    CommitAction(String),
    InteractiveRebase,
    Shell(String),
    QuickActions,
    MainMenu,
    Message(String),
    Switcher,
    BranchesView,
    FilesView,
    CommitsView,
    DatePicker(DatePickerState),
    CommitFiles(String, Vec<crate::git::files::FileEntry>),
    CommandPalette(CommandPaletteState),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PrimaryMode {
    Branches,
    Files,
    Commits,
    Stashes,
}

/// Canonical view: exactly one visible at a time. Replaces the legacy
/// `AppMode::{BranchesView, FilesView, CommitsView, StashDetail-as-view}`
/// quartet (kept as deprecated aliases until handlers/ui migrate).
/// Spec: `docs/ARCHITECTURE.md` §3.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum View {
    Branches,
    Files,
    Commits,
    Stashes,
}

impl View {
    #[must_use]
    pub const fn from_primary(mode: PrimaryMode) -> Self {
        match mode {
            PrimaryMode::Branches => Self::Branches,
            PrimaryMode::Files => Self::Files,
            PrimaryMode::Commits => Self::Commits,
            PrimaryMode::Stashes => Self::Stashes,
        }
    }
}

impl AppMode {
    /// Spotlight-style overlays that dim + trap focus. Replaces the
    /// hard-coded `is_overlay_modal` list in `ui::draw`.
    #[must_use]
    pub const fn is_overlay(&self) -> bool {
        matches!(
            self,
            Self::CommandPalette(_)
                | Self::MainMenu
                | Self::Manage
                | Self::Help
                | Self::ConfirmDelete(_)
        )
    }

    /// Legacy view markers kept for compat. Prefer `View` for new code.
    #[must_use]
    pub const fn as_view(&self) -> Option<View> {
        match self {
            Self::BranchesView => Some(View::Branches),
            Self::FilesView => Some(View::Files),
            Self::CommitsView => Some(View::Commits),
            // Debt: StashDetail doubles as the Stashes view.
            Self::StashDetail => Some(View::Stashes),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum CommandAction {
    CheckoutBranch,
    CreateBranch,
    OpenSettings,
    InteractiveRebase,
    StashChanges,
    PopStash,
    Fetch,
    Pull,
    Push,
    CommitChanges,
}

#[derive(Default, PartialEq, Debug, Clone)]
pub struct CommandPaletteState {
    pub query: String,
    pub selected: usize,
    pub actions: Vec<(String, CommandAction)>,
}

#[derive(Default, Clone)]
pub struct RebaseCommit {
    pub hash: String,
    pub original_message: String,
    pub new_message: Option<String>,
    pub action: RebaseAction,
}

#[derive(Default, Clone, PartialEq)]
pub enum RebaseAction {
    #[default]
    Pick,
    Reword,
    Drop,
    Squash,
}

#[derive(Default)]
pub struct RebaseState {
    pub commits: Vec<RebaseCommit>,
    pub selected: usize,
    pub editing: bool,
    pub input: String,
    pub ai_analyzing: bool,
}

#[derive(Default)]
pub struct SettingsState {
    pub selected: usize,
    pub editing: bool,
    pub selecting: bool,
    pub choice_idx: usize,
    pub choices: Vec<String>,
    pub input: String,
}

#[derive(Default)]
pub struct MainMenuState {
    pub selected: usize,
}

#[derive(Default)]
pub struct QuickActionsState {
    pub selected: usize,
    pub actions: Vec<String>,
}

pub struct ModalState {
    pub mode: AppMode,
}

pub struct UiState {
    pub primary_mode: PrimaryMode,
    pub mode: AppMode,
    pub modal_stack: Vec<ModalState>,

    // Selection and Navigation
    pub selected_branch_idx: usize,
    pub selected_file_idx: usize,
    pub selected_stash_idx: usize,
    pub manage_selected: usize,
    pub filter_selected: usize,
    pub diff_file_selected: usize,
    pub selected_commit_idx: usize,
    pub selected_commit_file_idx: usize,

    // Scrolling
    pub info_scroll: u16,
    pub list_start_index: usize,

    // Filtering and Searching
    pub current_filter: Option<BranchStatus>,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
    pub bulk_selected: HashSet<String>,

    // UI State
    pub sidebar_width: u16,
    pub needs_clear: bool,
    pub show_terminal: bool,
    pub active_panel: FilePanel,
    pub diff_panel: FilePanel,
    pub branch_sort_mode: BranchSortMode,
    pub show_inspector_drawer: bool,
    pub show_nav_sidebar: bool,
    pub nav_sidebar_selected: usize,

    // Interaction State (modifiers are matched per-KeyEvent, never stored)
    pub last_click_time: Instant,
    pub last_click_row: Option<usize>,

    // Persistence for tree
    pub open_paths: HashSet<String>,

    // Specific View States
    pub rebase_state: RebaseState,
    pub settings_state: SettingsState,
    pub main_menu_state: MainMenuState,
    pub quick_actions_state: QuickActionsState,

    // App Switcher
    pub mode_history: Vec<AppMode>,
    pub switcher_index: usize,

    // Animations
    pub snap_animation: Option<SnapAnimation>,
    pub branch_screen_positions: Vec<(usize, u16)>,
}

impl UiState {
    pub fn new(primary_mode: PrimaryMode, sidebar_width: u16) -> Self {
        Self {
            primary_mode,
            mode: AppMode::Normal,
            modal_stack: Vec::new(),
            selected_branch_idx: 0,
            selected_file_idx: 0,
            selected_stash_idx: 0,
            manage_selected: 0,
            filter_selected: 0,
            diff_file_selected: 0,
            selected_commit_idx: 0,
            selected_commit_file_idx: 0,
            info_scroll: 0,
            list_start_index: 0,
            current_filter: None,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            bulk_selected: HashSet::new(),
            sidebar_width,
            needs_clear: false,
            show_terminal: false,
            active_panel: FilePanel::Directory,
            diff_panel: FilePanel::Directory,
            branch_sort_mode: BranchSortMode::Recent,
            show_inspector_drawer: false,
            show_nav_sidebar: false,
            nav_sidebar_selected: 0,
            last_click_time: Instant::now(),
            last_click_row: None,
            open_paths: HashSet::new(),
            rebase_state: RebaseState::default(),
            settings_state: SettingsState::default(),
            main_menu_state: MainMenuState::default(),
            quick_actions_state: QuickActionsState {
                selected: 0,
                actions: vec![
                    "git pull".to_string(),
                    "git push".to_string(),
                    "git fetch --all".to_string(),
                    "git status".to_string(),
                    "git remote -v".to_string(),
                    "git branch -a".to_string(),
                    "git commit --amend --no-edit".to_string(),
                    "git log -n 5".to_string(),
                ],
            },
            mode_history: vec![
                AppMode::BranchesView,
                AppMode::FilesView,
                AppMode::CommitsView,
                AppMode::StashDetail, // We use StashDetail as the view for Stashes mode
                AppMode::Diff,
                AppMode::Search,
                AppMode::Filter,
                AppMode::Settings,
                AppMode::QuickActions,
            ],
            switcher_index: 0,
            snap_animation: None,
            branch_screen_positions: Vec::new(),
        }
    }

    pub fn track_history(&mut self, mode: AppMode) {
        if matches!(
            mode,
            AppMode::BranchesView
                | AppMode::FilesView
                | AppMode::CommitsView
                | AppMode::StashDetail
                | AppMode::Diff
                | AppMode::Search
                | AppMode::Filter
                | AppMode::Settings
                | AppMode::QuickActions
        ) {
            self.mode_history.retain(|m| m != &mode);
            self.mode_history.insert(0, mode);
        }
    }

    pub fn push_modal(&mut self, mode: AppMode) {
        if self.current_mode() == &mode {
            return;
        }
        self.track_history(mode.clone());
        self.modal_stack.push(ModalState { mode });
    }

    pub fn pop_modal(&mut self) -> Option<ModalState> {
        let popped = self.modal_stack.pop();

        let new_current = self.current_mode().clone();
        if new_current == AppMode::Normal {
            let view = match self.primary_mode {
                PrimaryMode::Branches => AppMode::BranchesView,
                PrimaryMode::Files => AppMode::FilesView,
                PrimaryMode::Commits => AppMode::CommitsView,
                PrimaryMode::Stashes => AppMode::StashDetail,
            };
            self.track_history(view);
        } else {
            self.track_history(new_current);
        }

        self.needs_clear = true;
        popped
    }

    pub fn current_mode(&self) -> &AppMode {
        if let Some(modal) = self.modal_stack.last() {
            &modal.mode
        } else {
            &self.mode
        }
    }

    pub fn current_mode_mut(&mut self) -> &mut AppMode {
        if let Some(modal) = self.modal_stack.last_mut() {
            &mut modal.mode
        } else {
            &mut self.mode
        }
    }

    /// Canonical visible view derived from `primary_mode`.
    /// New code should branch on this instead of `AppMode::*View`.
    #[must_use]
    pub const fn view(&self) -> View {
        View::from_primary(self.primary_mode)
    }

    /// Pure centered-viewport math: `(start, visible_count)` for `selected`
    /// inside `total` rows with `height` visible slots. Keeps the selection
    /// vertically centered and clamps at both ends. `render_main_list` must
    /// use this instead of inline scroll math (tested below).
    #[must_use]
    pub const fn branch_viewport(selected: usize, total: usize, height: usize) -> (usize, usize) {
        if height == 0 || total == 0 {
            return (0, 0);
        }
        let mut start = 0;
        if total > height {
            let half = height / 2;
            if selected > half {
                start = selected - half;
            }
            let end = start + height;
            if end > total {
                start = total.saturating_sub(height);
            }
        }
        let visible = if height < total.saturating_sub(start) {
            height
        } else {
            total.saturating_sub(start)
        };
        (start, visible)
    }

    /// Sync scroll cache for the branches table. Returns `(start, visible)`.
    /// Also resets per-frame mouse hit positions (rebuilt during render —
    /// see `ARCHITECTURE.md` debt note on `draw(&mut App)`).
    pub fn sync_branch_viewport(&mut self, total: usize, height: usize) -> (usize, usize) {
        let (start, visible) =
            Self::branch_viewport(self.selected_branch_idx, total, height);
        self.list_start_index = start;
        self.branch_screen_positions.clear();
        (start, visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_state_defaults() {
        let state = UiState::new(PrimaryMode::Branches, 30);
        assert_eq!(state.branch_sort_mode, BranchSortMode::Recent);
        assert!(!state.show_inspector_drawer);
        assert!(!state.show_nav_sidebar);
    }

    #[test]
    fn test_drawer_state_toggling() {
        let mut state = UiState::new(PrimaryMode::Branches, 30);
        state.show_inspector_drawer = true;
        assert!(state.show_inspector_drawer);
        state.show_inspector_drawer = false;
        assert!(!state.show_inspector_drawer);
    }

    #[test]
    fn test_view_from_primary() {
        assert_eq!(
            UiState::new(PrimaryMode::Branches, 30).view(),
            View::Branches
        );
        assert_eq!(UiState::new(PrimaryMode::Files, 30).view(), View::Files);
        assert_eq!(
            UiState::new(PrimaryMode::Commits, 30).view(),
            View::Commits
        );
        assert_eq!(
            UiState::new(PrimaryMode::Stashes, 30).view(),
            View::Stashes
        );
    }

    #[test]
    fn test_overlay_set() {
        assert!(AppMode::Manage.is_overlay());
        assert!(AppMode::Help.is_overlay());
        assert!(!AppMode::Normal.is_overlay());
        assert!(!AppMode::Diff.is_overlay());
        assert_eq!(AppMode::BranchesView.as_view(), Some(View::Branches));
        assert_eq!(AppMode::StashDetail.as_view(), Some(View::Stashes));
        assert_eq!(AppMode::Manage.as_view(), None);
    }

    #[test]
    fn test_branch_viewport_centering() {
        use UiState as U;
        assert_eq!(U::branch_viewport(0, 0, 10), (0, 0));
        assert_eq!(U::branch_viewport(0, 5, 0), (0, 0));
        // Fits entirely.
        assert_eq!(U::branch_viewport(2, 5, 10), (0, 5));
        // Pinned to top.
        assert_eq!(U::branch_viewport(0, 87, 20), (0, 20));
        // Centered in the middle.
        assert_eq!(U::branch_viewport(43, 87, 20), (33, 20));
        // Clamped at the end.
        assert_eq!(U::branch_viewport(86, 87, 20), (67, 20));
        assert_eq!(U::branch_viewport(100, 87, 20), (67, 20));
    }

    #[test]
    fn test_sync_branch_viewport_resets_hits() {
        let mut state = UiState::new(PrimaryMode::Branches, 30);
        state.branch_screen_positions.push((0, 3));
        let (start, visible) = state.sync_branch_viewport(87, 20);
        assert_eq!((start, visible), (0, 20));
        assert_eq!(state.list_start_index, start);
        assert!(state.branch_screen_positions.is_empty());
    }
}

