use crate::models::Branch;

#[derive(PartialEq)]
pub enum AppMode {
    Normal,
    Help,
    Manage,
    Diff,
    Message(String),
}

pub struct App {
    pub branches: Vec<Branch>,
    pub selected: usize,
    pub marked: Vec<bool>,
    pub mode: AppMode,
    pub branch_info: String,
    pub info_scroll: u16,
    pub manage_selected: usize,
    pub list_start_index: usize, // Para rastrear el scroll y arreglar el mouse
}

impl App {
    pub fn new(branches: Vec<Branch>) -> Self {
        let len = branches.len();
        Self {
            branches,
            selected: 0,
            marked: vec![false; len],
            mode: AppMode::Normal,
            branch_info: String::new(),
            info_scroll: 0,
            manage_selected: 0,
            list_start_index: 0,
        }
    }

    pub fn toggle_help(&mut self) {
        if self.mode == AppMode::Help {
            self.mode = AppMode::Normal;
        } else {
            self.mode = AppMode::Help;
        }
    }

    pub fn next(&mut self) {
        if self.selected < self.branches.len().saturating_sub(1) {
            self.selected += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn toggle(&mut self) {
        if self.marked.len() > self.selected {
            self.marked[self.selected] = !self.marked[self.selected];
        }
    }
}
