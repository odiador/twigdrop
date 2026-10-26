use crate::models::{BranchStatus, GutterStatus};
use crate::ui::animations::SnapAnimation;
use std::collections::HashSet;
use std::time::Instant;

pub use crate::state::frame_cache::FrameCache;
pub use crate::state::modals::{
    CommandAction, CommandPaletteState, DatePickerField, DatePickerState, FolderProfileState,
    MainMenuState, ModalState, QuickActionsState, SettingsState,
};
pub use crate::state::rebase::{RebaseAction, RebaseCommit, RebaseState};

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct BlameEntry {
    pub commit: String,
    pub author: String,
    pub date: String,
}

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
    pub line_blames: std::collections::HashMap<usize, BlameEntry>,
    pub show_blame: bool,
}

impl PreviewState {
    pub fn jump_hunk(&mut self, forward: bool) -> Option<usize> {
        if self.line_diffs.is_empty() {
            return None;
        }

        let mut hunk_starts: Vec<usize> = self
            .line_diffs
            .keys()
            .copied()
            .filter(|&line| line == 0 || !self.line_diffs.contains_key(&(line - 1)))
            .collect();
        hunk_starts.sort_unstable();

        if hunk_starts.is_empty() {
            return None;
        }

        let target = if forward {
            hunk_starts
                .iter()
                .copied()
                .find(|&line| line > self.cursor_y)
                .unwrap_or(hunk_starts[0])
        } else {
            let current_start = hunk_starts
                .iter()
                .copied()
                .rfind(|&line| line <= self.cursor_y);

            if let Some(s) = current_start {
                if self.cursor_y > s && self.line_diffs.contains_key(&self.cursor_y) {
                    s
                } else {
                    hunk_starts
                        .iter()
                        .copied()
                        .rfind(|&line| line < s)
                        .unwrap_or(*hunk_starts.last().unwrap())
                }
            } else {
                *hunk_starts.last().unwrap()
            }
        };

        self.cursor_y = target;
        if target < self.scroll_y {
            self.scroll_y = target.saturating_sub(3);
        } else if target >= self.scroll_y + 20 {
            self.scroll_y = target.saturating_sub(10);
        }

        Some(target)
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum FilePanel {
    #[default]
    Directory,
    Preview,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum CommitPanel {
    #[default]
    Tree,
    Files,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum BranchSortMode {
    #[default]
    Recent,
    PrunableFirst,
    Alphabetical,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum PrTab {
    #[default]
    Description,
    Diff,
    Checks,
    Comments,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PrReviewKind {
    Approve,
    RequestChanges,
    Comment,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct PrReviewModalState {
    pub pr_number: u64,
    pub pr_title: String,
    pub kind: PrReviewKind,
    pub body: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Copy, Default)]
pub enum PrFilter {
    #[default]
    Open,
    All,
    Merged,
    Closed,
}

impl PrFilter {
    #[must_use]
    pub const fn as_gh_arg(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::All => "all",
            Self::Merged => "merged",
            Self::Closed => "closed",
        }
    }

    #[must_use]
    pub const fn next(self) -> Self {
        match self {
            Self::Open => Self::All,
            Self::All => Self::Merged,
            Self::Merged => Self::Closed,
            Self::Closed => Self::Open,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Open => "Open",
            Self::All => "All",
            Self::Merged => "Merged",
            Self::Closed => "Closed",
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct ConflictResolverState {
    pub files: Vec<String>,
    pub selected_file_idx: usize,
    pub hunks: Vec<crate::git::conflicts::ConflictHunk>,
    pub selected_hunk_idx: usize,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Normal,
    Help,
    Manage,
    Filter,
    Settings,
    Diff,
    CodePreview(PreviewState),
    ConfirmDelete(Vec<String>),
    CreateBranch(String),
    RenameBranch(String, String),
    CommitAction(String),
    InteractiveRebase,
    Shell(String),
    QuickActions,
    MainMenu,
    Message(String),
    Switcher,
    DatePicker(DatePickerState),
    CommitFiles(String, Vec<crate::git::files::FileEntry>),
    CommandPalette(CommandPaletteState),
    FolderProfile(FolderProfileState),
    PrReview(PrReviewModalState),
    ConflictResolver(ConflictResolverState),
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum PrimaryMode {
    Branches,
    Files,
    Commits,
    Stashes,
    PullRequests,
}

/// Canonical view: exactly one visible at a time.
/// Spec: `docs/ARCHITECTURE.md` §3.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum View {
    Branches,
    Files,
    Commits,
    Stashes,
    PullRequests,
}

impl View {
    /// All primary views, in sidebar order.
    pub const ALL: [Self; 5] = [
        Self::Branches,
        Self::Files,
        Self::Commits,
        Self::Stashes,
        Self::PullRequests,
    ];

    #[must_use]
    pub const fn from_primary(mode: PrimaryMode) -> Self {
        match mode {
            PrimaryMode::Branches => Self::Branches,
            PrimaryMode::Files => Self::Files,
            PrimaryMode::Commits => Self::Commits,
            PrimaryMode::Stashes => Self::Stashes,
            PrimaryMode::PullRequests => Self::PullRequests,
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Branches => "Branches",
            Self::Files => "Files",
            Self::Commits => "Commit Tree",
            Self::Stashes => "Stashes",
            Self::PullRequests => "Pull Requests",
        }
    }
}

/// App-switcher entry: a primary view or the help overlay.
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SwitchTarget {
    View(View),
    Help,
}

impl SwitchTarget {
    /// Single source for the switcher list (render + keyboard).
    pub const ALL: [Self; 6] = [
        Self::View(View::Files),
        Self::View(View::Branches),
        Self::View(View::Commits),
        Self::View(View::Stashes),
        Self::View(View::PullRequests),
        Self::Help,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::View(view) => view.label(),
            Self::Help => "Help",
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
                | Self::FolderProfile(_)
                | Self::PrReview(_)
                | Self::ConflictResolver(_)
        )
    }
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
    pub selected_pr_idx: usize,
    pub pr_tab: PrTab,
    pub pr_filter: PrFilter,
    pub pr_scroll_y: u16,
    pub pr_check_idx: usize,

    // Scrolling
    pub info_scroll: u16,

    // Filtering and Searching
    pub current_filter: Option<BranchStatus>,
    pub search_query: String,
    pub filtered_indices: Vec<usize>,
    pub bulk_selected: HashSet<String>,
    pub branches_active_only: bool,
    pub files_changed_only: bool,
    pub show_hidden_files: bool,
    pub files_search_query: String,
    pub files_search_active: bool,
    pub filtered_file_indices: Vec<usize>,

    // UI State
    pub sidebar_width: u16,
    pub needs_clear: bool,
    pub show_terminal: bool,
    pub active_panel: FilePanel,
    pub diff_panel: FilePanel,
    pub commit_panel: CommitPanel,
    pub branch_sort_mode: BranchSortMode,
    pub show_inspector_drawer: bool,
    pub show_nav_sidebar: bool,
    pub nav_sidebar_selected: usize,
    pub maximized_panel: bool,

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

    // App Switcher (views only — modals never enter history)
    pub mode_history: Vec<View>,
    pub switcher_index: usize,

    // Transient toast notification (expires automatically).
    pub toast: Option<crate::state::toast::Toast>,

    // Animations
    pub snap_animation: Option<SnapAnimation>,
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
            selected_pr_idx: 0,
            pr_tab: PrTab::Description,
            pr_filter: PrFilter::Open,
            pr_scroll_y: 0,
            pr_check_idx: 0,
            info_scroll: 0,
            current_filter: None,
            search_query: String::new(),
            filtered_indices: Vec::new(),
            bulk_selected: HashSet::new(),
            branches_active_only: false,
            files_changed_only: false,
            show_hidden_files: false,
            files_search_query: String::new(),
            files_search_active: false,
            filtered_file_indices: Vec::new(),
            sidebar_width,
            needs_clear: false,
            show_terminal: false,
            active_panel: FilePanel::Directory,
            diff_panel: FilePanel::Directory,
            commit_panel: CommitPanel::Tree,
            branch_sort_mode: BranchSortMode::Recent,
            show_inspector_drawer: false,
            show_nav_sidebar: false,
            nav_sidebar_selected: 0,
            maximized_panel: false,
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
            mode_history: Vec::from(View::ALL),
            switcher_index: 0,
            snap_animation: None,
            toast: None,
        }
    }

    pub fn track_history(&mut self, view: View) {
        self.mode_history.retain(|v| *v != view);
        self.mode_history.insert(0, view);
    }

    /// Switcher list order: most-recently-used views first, then the
    /// remaining targets. Render and keyboard share this (single source).
    #[must_use]
    pub fn switcher_order(&self) -> Vec<SwitchTarget> {
        let mut ordered: Vec<SwitchTarget> = self
            .mode_history
            .iter()
            .map(|&view| SwitchTarget::View(view))
            .collect();
        for target in SwitchTarget::ALL {
            if !ordered.contains(&target) {
                ordered.push(target);
            }
        }
        ordered
    }

    /// Show a transient toast, replacing any previous one.
    pub fn notify(
        &mut self,
        kind: crate::state::toast::ToastKind,
        text: String,
        undo: Option<crate::state::toast::UndoRestore>,
    ) {
        self.toast = Some(crate::state::toast::Toast::new(kind, text, undo));
    }

    /// Drop the toast once expired. Called at `draw` entry.
    pub fn expire_toast(&mut self) {
        if self.toast.as_ref().is_some_and(|t| !t.is_live()) {
            self.toast = None;
        }
    }

    /// Live toast with undo data (the `u` binding target).
    pub fn undoable_toast(&self) -> Option<&crate::state::toast::Toast> {
        self.toast.as_ref().filter(|t| t.can_undo())
    }

    pub fn push_modal(&mut self, mode: AppMode) {
        if self.current_mode() == &mode {
            return;
        }
        self.track_history(self.view());
        self.modal_stack.push(ModalState { mode });
    }

    pub fn pop_modal(&mut self) -> Option<ModalState> {
        let popped = self.modal_stack.pop();
        self.track_history(self.view());
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
        assert_eq!(UiState::new(PrimaryMode::Commits, 30).view(), View::Commits);
        assert_eq!(UiState::new(PrimaryMode::Stashes, 30).view(), View::Stashes);
        assert_eq!(
            UiState::new(PrimaryMode::PullRequests, 30).view(),
            View::PullRequests
        );
    }

    #[test]
    fn test_overlay_set() {
        assert!(AppMode::Manage.is_overlay());
        assert!(AppMode::Help.is_overlay());
        assert!(!AppMode::Normal.is_overlay());
        assert!(!AppMode::Diff.is_overlay());
        assert!(!AppMode::Message(String::new()).is_overlay());
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
    fn test_switcher_targets_cover_views() {
        use SwitchTarget as T;
        assert_eq!(SwitchTarget::ALL.len(), 6);
        for view in View::ALL {
            assert!(SwitchTarget::ALL.contains(&T::View(view)), "{view:?}");
        }
        assert!(!SwitchTarget::View(View::Branches).label().is_empty());
        assert_eq!(SwitchTarget::Help.label(), "Help");
    }

    #[test]
    fn test_switcher_order_mru_first() {
        use SwitchTarget as T;
        let mut state = UiState::new(PrimaryMode::Branches, 30);
        state.track_history(View::Commits);
        state.track_history(View::Files);
        let order = state.switcher_order();
        assert_eq!(order[0], T::View(View::Files));
        assert_eq!(order[1], T::View(View::Commits));
        assert_eq!(order.len(), SwitchTarget::ALL.len());
        // Current view is always reachable.
        for view in View::ALL {
            assert!(state.switcher_order().contains(&T::View(view)));
        }
    }

    #[test]
    fn test_preview_state_jump_hunk() {
        let mut diffs = std::collections::HashMap::new();
        diffs.insert(5, GutterStatus::Modified);
        diffs.insert(6, GutterStatus::Modified);
        diffs.insert(20, GutterStatus::Added);
        diffs.insert(50, GutterStatus::Deleted);

        let mut state = PreviewState {
            file_path: "test.rs".to_string(),
            lines: vec!["line".to_string(); 60],
            highlighted_lines: Vec::new(),
            cursor_y: 0,
            scroll_y: 0,
            selection_start: None,
            selection_end: None,
            line_diffs: diffs,
            line_blames: std::collections::HashMap::new(),
            show_blame: false,
        };

        // Forward jumps: 0 -> 5 -> 20 -> 50 -> wrap to 5
        assert_eq!(state.jump_hunk(true), Some(5));
        assert_eq!(state.cursor_y, 5);
        assert_eq!(state.jump_hunk(true), Some(20));
        assert_eq!(state.cursor_y, 20);
        assert_eq!(state.jump_hunk(true), Some(50));
        assert_eq!(state.cursor_y, 50);
        assert_eq!(state.jump_hunk(true), Some(5));
        assert_eq!(state.cursor_y, 5);

        // Backward jumps from 5 wraps to 50
        assert_eq!(state.jump_hunk(false), Some(50));
        assert_eq!(state.cursor_y, 50);
        assert_eq!(state.jump_hunk(false), Some(20));
        assert_eq!(state.cursor_y, 20);

        // When inside a multi-line hunk (line 6 inside 5..6): jumps to start of hunk (5)
        state.cursor_y = 6;
        assert_eq!(state.jump_hunk(false), Some(5));
        assert_eq!(state.cursor_y, 5);
    }
}
