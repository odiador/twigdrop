use crate::models::{Branch, BranchStatus};
use std::time::Instant;

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Normal,
    Help,
    Manage,
    Filter,
    Diff,
    Message(String),
}

pub struct App {
    pub branches: Vec<Branch>,
    pub current_branch: String,
    pub selected: usize,
    pub mode: AppMode,
    pub branch_info: String,
    pub info_scroll: u16,
    pub manage_selected: usize,
    pub filter_selected: usize,
    pub current_filter: Option<BranchStatus>,
    pub list_start_index: usize,
    pub filtered_indices: Vec<usize>,
    
    // Double click support
    pub last_click_time: Instant,
    pub last_click_row: Option<usize>,
    pub needs_clear: bool,
}

impl App {
    pub fn new(branches: Vec<Branch>, current_branch: String) -> Self {
        let mut app = Self {
            branches,
            current_branch,
            selected: 0,
            mode: AppMode::Normal,
            branch_info: String::new(),
            info_scroll: 0,
            manage_selected: 0,
            filter_selected: 0,
            current_filter: None,
            list_start_index: 0,
            filtered_indices: vec![],
            last_click_time: Instant::now(),
            last_click_row: None,
            needs_clear: false,
        };
        app.refresh_filtered_branches();
        app
    }

    pub fn refresh_filtered_branches(&mut self) {
        self.filtered_indices = if let Some(filter) = &self.current_filter {
            self.branches
                .iter()
                .enumerate()
                .filter(|(_, b)| b.status.contains(filter))
                .map(|(i, _)| i)
                .collect()
        } else {
            (0..self.branches.len()).collect()
        };

        // Clamp selected index
        let max = self.filtered_indices.len().saturating_sub(1);
        if self.selected > max {
            self.selected = max;
        }
    }

    pub fn get_filtered_branches(&self) -> Vec<&Branch> {
        self.filtered_indices
            .iter()
            .map(|&i| &self.branches[i])
            .collect()
    }

    pub fn toggle_help(&mut self) {
        if self.mode == AppMode::Help {
            self.mode = AppMode::Normal;
        } else {
            self.mode = AppMode::Help;
        }
    }

    pub fn next(&mut self) {
        let max = self.filtered_indices.len().saturating_sub(1);
        if self.selected < max {
            self.selected += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
}
