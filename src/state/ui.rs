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
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PrimaryMode {
    Branches,
    Files,
    Commits,
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

    // Interaction State
    pub alt_pressed: bool,
    pub shift_pressed: bool,
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
            alt_pressed: false,
            shift_pressed: false,
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
                AppMode::Diff,
                AppMode::StashDetail,
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
                | AppMode::Diff
                | AppMode::StashDetail
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
}
