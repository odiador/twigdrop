use crate::models::{Branch, BranchStatus, MergeStatus};
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PrimaryMode {
    Branches,
    Files,
}

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Normal,
    Help,
    Manage,
    Filter,
    Diff,
    StashDetail,
    Message(String),
}

pub struct MergeUpdate {
    pub branch_name: String,
    pub status: MergeStatus,
}

pub struct AIUpdate {
    pub analysis: String,
}

#[derive(Default)]
pub struct BranchState {
    pub branches: Vec<Branch>,
    pub selected: usize,
    pub manage_selected: usize,
    pub filter_selected: usize,
    pub current_filter: Option<BranchStatus>,
    pub list_start_index: usize,
    pub filtered_indices: Vec<usize>,
    pub bulk_selected: HashSet<String>,
}

#[derive(Default)]
pub struct FileState {
    pub file_tree: Vec<crate::git::files::FileEntry>,
    pub file_selected: usize,
    pub file_scroll: usize,
    pub git_file_statuses: HashMap<String, crate::git::files::FileStatus>,
}

#[derive(Default)]
pub struct StashState {
    pub stashes: Vec<crate::git::stash::StashEntry>,
    pub stash_selected: usize,
    pub stash_files: Vec<String>,
    pub stash_diff: String,
}

pub struct AIState {
    pub ai_provider: Option<Box<dyn crate::ai::AIProvider>>,
    pub db: Option<crate::db::Database>,
    pub ai_analysis: Option<String>,
    pub ai_rx: mpsc::Receiver<AIUpdate>,
    pub ai_trigger_tx: mpsc::Sender<(String, String)>,
}

pub struct App {
    pub branch_state: BranchState,
    pub file_state: FileState,
    pub stash_state: StashState,
    pub ai_state: AIState,

    pub current_branch: String,
    pub primary_mode: PrimaryMode,
    pub mode: AppMode,
    pub branch_info: String,
    pub info_scroll: u16,

    // UI & System State
    pub last_click_time: Instant,
    pub last_click_row: Option<usize>,
    pub needs_clear: bool,
    pub alt_pressed: bool,
    pub shift_pressed: bool,
    pub config: crate::utils::config::Config,

    // Background updates
    pub rx: mpsc::Receiver<MergeUpdate>,
    pub trigger_tx: mpsc::Sender<()>,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        branches: Vec<Branch>,
        current_branch: String,
        rx: mpsc::Receiver<MergeUpdate>,
        trigger_tx: mpsc::Sender<()>,
        ai_rx: mpsc::Receiver<AIUpdate>,
        ai_trigger_tx: mpsc::Sender<(String, String)>,
    ) -> Self {
        let mut app = Self {
            branch_state: BranchState {
                branches,
                ..Default::default()
            },
            file_state: FileState::default(),
            stash_state: StashState::default(),
            ai_state: AIState {
                ai_provider: None,
                db: None,
                ai_analysis: None,
                ai_rx,
                ai_trigger_tx,
            },
            current_branch,
            primary_mode: PrimaryMode::Branches,
            mode: AppMode::Normal,
            branch_info: String::new(),
            info_scroll: 0,
            last_click_time: Instant::now(),
            last_click_row: None,
            needs_clear: false,
            alt_pressed: false,
            shift_pressed: false,
            config: crate::utils::config::load_config(),
            rx,
            trigger_tx,
        };
        app.refresh_filtered_branches();
        app
    }

    pub fn refresh_branches(&mut self, path: &str) {
        self.branch_state.branches = crate::git::build_branches(path);
        self.refresh_filtered_branches();
        let _ = self.trigger_tx.try_send(());
    }

    pub fn update_from_channel(&mut self) {
        while let Ok(update) = self.rx.try_recv() {
            if let Some(branch) = self
                .branch_state
                .branches
                .iter_mut()
                .find(|b| b.name == update.branch_name)
            {
                branch.merge_status = update.status;
            }
        }
        while let Ok(update) = self.ai_state.ai_rx.try_recv() {
            self.ai_state.ai_analysis = Some(update.analysis);
        }
    }

    pub fn refresh_filtered_branches(&mut self) {
        self.branch_state.filtered_indices = if let Some(filter) = &self.branch_state.current_filter
        {
            self.branch_state
                .branches
                .iter()
                .enumerate()
                .filter(|(_, b)| b.status.contains(filter))
                .map(|(i, _)| i)
                .collect()
        } else {
            (0..self.branch_state.branches.len()).collect()
        };

        // Clamp selected index
        let max = self.branch_state.filtered_indices.len().saturating_sub(1);
        if self.branch_state.selected > max {
            self.branch_state.selected = max;
        }
    }

    pub fn get_filtered_branches(&self) -> Vec<&Branch> {
        self.branch_state
            .filtered_indices
            .iter()
            .map(|&i| &self.branch_state.branches[i])
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
        match self.primary_mode {
            PrimaryMode::Branches => {
                let max = self.branch_state.filtered_indices.len().saturating_sub(1);
                if self.branch_state.selected < max {
                    self.branch_state.selected += 1;
                }
            }
            PrimaryMode::Files => {
                if self.file_state.file_selected < self.file_state.file_tree.len().saturating_sub(1)
                {
                    self.file_state.file_selected += 1;
                }
            }
        }
    }

    pub fn previous(&mut self) {
        match self.primary_mode {
            PrimaryMode::Branches => {
                if self.branch_state.selected > 0 {
                    self.branch_state.selected -= 1;
                }
            }
            PrimaryMode::Files => {
                if self.file_state.file_selected > 0 {
                    self.file_state.file_selected -= 1;
                }
            }
        }
    }

    pub fn load_file_tree(&mut self, path: &str) {
        if self.file_state.file_tree.is_empty() {
            self.file_state.git_file_statuses = crate::git::files::get_git_file_statuses(path);
            self.file_state.file_tree =
                crate::git::files::build_file_tree(path, "", 0, &self.file_state.git_file_statuses);
            self.file_state.file_selected = 0;
            self.file_state.file_scroll = 0;
        }
    }

    pub fn toggle_file_dir(&mut self, path_str: &str) {
        if self.file_state.file_selected >= self.file_state.file_tree.len() {
            return;
        }

        let is_dir = self.file_state.file_tree[self.file_state.file_selected].is_dir;
        let is_open = self.file_state.file_tree[self.file_state.file_selected].is_open;
        let depth = self.file_state.file_tree[self.file_state.file_selected].depth;
        let rel_path = self.file_state.file_tree[self.file_state.file_selected]
            .path
            .to_string_lossy()
            .to_string();

        if is_dir {
            if is_open {
                // Close: remove all children
                self.file_state.file_tree[self.file_state.file_selected].is_open = false;
                let i = self.file_state.file_selected + 1;
                while i < self.file_state.file_tree.len()
                    && self.file_state.file_tree[i].depth > depth
                {
                    self.file_state.file_tree.remove(i);
                }
            } else {
                // Open: insert children
                self.file_state.file_tree[self.file_state.file_selected].is_open = true;
                let children = crate::git::files::build_file_tree(
                    path_str,
                    &rel_path,
                    depth + 1,
                    &self.file_state.git_file_statuses,
                );
                for (j, child) in children.into_iter().enumerate() {
                    self.file_state
                        .file_tree
                        .insert(self.file_state.file_selected + 1 + j, child);
                }
            }
        }
    }

    pub fn load_stashes(&mut self, path: &str) {
        self.stash_state.stashes = crate::git::stash::get_stashes(path);
        self.stash_state.stash_selected = 0;
    }

    pub fn load_stash_detail(&mut self, path: &str) {
        if let Some(stash) = self
            .stash_state
            .stashes
            .get(self.stash_state.stash_selected)
        {
            self.stash_state.stash_files = crate::git::stash::get_stash_files(path, &stash.id);
            self.stash_state.stash_diff = crate::git::stash::get_stash_diff(path, &stash.id);
        }
    }

    pub fn setup_ai(&mut self, path: &str) {
        dotenv::dotenv().ok();
        let db_path = std::path::PathBuf::from(path)
            .join(".git")
            .join("twigdrop.db");
        self.ai_state.db = crate::db::Database::new(db_path).ok();

        let provider_type = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
        let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "llama3".to_string());

        if provider_type == "openai" {
            if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                self.ai_state.ai_provider =
                    Some(Box::new(crate::ai::openai::OpenAIProvider::new(key, model)));
            }
        } else {
            let url = std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            self.ai_state.ai_provider =
                Some(Box::new(crate::ai::ollama::OllamaProvider::new(url, model)));
        }
    }

    pub async fn trigger_ai_analysis(&mut self, path: &str, branch_name: &str) {
        let hash = match crate::git::commands::run_git(path, &["rev-parse", branch_name]) {
            Ok(h) => h.trim().to_string(),
            Err(_) => return,
        };

        // 1. Check Cache
        if let Some(db) = &self.ai_state.db
            && let Ok(Some((cached_hash, summary, cleanup))) = db.get_analysis(branch_name)
            && cached_hash == hash
        {
            self.ai_state.ai_analysis = Some(format!(
                "--- CACHED ANALYSIS ---\n\nSummary:\n{}\n\nRecommendation:\n{}",
                summary, cleanup
            ));
            return;
        }

        // 2. Run AI Analysis
        if let Some(provider) = &self.ai_state.ai_provider {
            self.ai_state.ai_analysis = Some("Analyzing with AI...".to_string());

            let diff = crate::git::get_branch_info(path, branch_name);
            let summary_res = provider.summarize_diff(&diff).await;
            let cleanup_res = provider.recommend_cleanup(branch_name).await;

            match (summary_res, cleanup_res) {
                (Ok(s), Ok(c)) => {
                    if let Some(db) = &self.ai_state.db {
                        let _ = db.save_analysis(branch_name, &hash, &s, &c);
                    }
                    self.ai_state.ai_analysis =
                        Some(format!("Summary:\n{}\n\nRecommendation:\n{}", s, c));
                }
                _ => {
                    self.ai_state.ai_analysis = Some("AI Analysis failed.".to_string());
                }
            }
        } else {
            self.ai_state.ai_analysis = Some("No AI Provider configured.".to_string());
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(&idx) = self
            .branch_state
            .filtered_indices
            .get(self.branch_state.selected)
        {
            let name = self.branch_state.branches[idx].name.clone();
            if self.branch_state.bulk_selected.contains(&name) {
                self.branch_state.bulk_selected.remove(&name);
            } else {
                self.branch_state.bulk_selected.insert(name);
            }
        }
    }
}
