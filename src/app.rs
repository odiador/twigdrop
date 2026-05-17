use crate::models::{Branch, BranchStatus, MergeStatus};
use std::time::Instant;
use tokio::sync::mpsc;

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Normal,
    Help,
    Manage,
    Filter,
    Diff,
    DirectorySearcher,
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
    pub bulk_selected: std::collections::HashSet<String>,

    // Directory Searcher
    pub file_tree: Vec<crate::git::files::FileEntry>,
    pub file_selected: usize,
    pub file_scroll: usize,
    pub git_file_statuses: std::collections::HashMap<String, crate::git::files::FileStatus>,

    // AI & Storage
    pub ai_provider: Option<Box<dyn crate::ai::AIProvider>>,
    pub db: Option<crate::db::Database>,
    pub ai_analysis: Option<String>,

    // Stash Detail
    pub stashes: Vec<crate::git::stash::StashEntry>,
    pub stash_selected: usize,
    pub stash_files: Vec<String>,
    pub stash_diff: String,

    // Double click support
    pub last_click_time: Instant,
    pub last_click_row: Option<usize>,
    pub needs_clear: bool,
    pub alt_pressed: bool,
    pub config: crate::utils::config::Config,

    // Background updates
    pub rx: mpsc::Receiver<MergeUpdate>,
    pub trigger_tx: mpsc::Sender<()>,
    pub ai_rx: mpsc::Receiver<AIUpdate>,
    pub ai_trigger_tx: mpsc::Sender<(String, String)>,
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
            bulk_selected: std::collections::HashSet::new(),
            file_tree: vec![],
            file_selected: 0,
            file_scroll: 0,
            git_file_statuses: std::collections::HashMap::new(),
            ai_provider: None,
            db: None,
            ai_analysis: None,
            stashes: vec![],
            stash_selected: 0,
            stash_files: vec![],
            stash_diff: String::new(),
            last_click_time: Instant::now(),
            last_click_row: None,
            needs_clear: false,
            alt_pressed: false,
            config: crate::utils::config::load_config(),
            rx,
            trigger_tx,
            ai_rx,
            ai_trigger_tx,
        };
        app.refresh_filtered_branches();
        app
    }

    pub fn refresh_branches(&mut self, path: &str) {
        self.branches = crate::git::build_branches(path);
        self.refresh_filtered_branches();
        let _ = self.trigger_tx.try_send(());
    }

    pub fn update_from_channel(&mut self) {
        while let Ok(update) = self.rx.try_recv() {
            if let Some(branch) = self
                .branches
                .iter_mut()
                .find(|b| b.name == update.branch_name)
            {
                branch.merge_status = update.status;
            }
        }
        while let Ok(update) = self.ai_rx.try_recv() {
            self.ai_analysis = Some(update.analysis);
        }
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

    pub fn load_file_tree(&mut self, path: &str) {
        self.git_file_statuses = crate::git::files::get_git_file_statuses(path);
        self.file_tree = crate::git::files::build_file_tree(path, "", 0, &self.git_file_statuses);
        self.file_selected = 0;
        self.file_scroll = 0;
    }

    pub fn toggle_file_dir(&mut self, path_str: &str) {
        if self.file_selected >= self.file_tree.len() {
            return;
        }

        let is_dir = self.file_tree[self.file_selected].is_dir;
        let is_open = self.file_tree[self.file_selected].is_open;
        let depth = self.file_tree[self.file_selected].depth;
        let rel_path = self.file_tree[self.file_selected]
            .path
            .to_string_lossy()
            .to_string();

        if is_dir {
            if is_open {
                // Close: remove all children
                self.file_tree[self.file_selected].is_open = false;
                let i = self.file_selected + 1;
                while i < self.file_tree.len() && self.file_tree[i].depth > depth {
                    self.file_tree.remove(i);
                }
            } else {
                // Open: insert children
                self.file_tree[self.file_selected].is_open = true;
                let children = crate::git::files::build_file_tree(
                    path_str,
                    &rel_path,
                    depth + 1,
                    &self.git_file_statuses,
                );
                for (j, child) in children.into_iter().enumerate() {
                    self.file_tree.insert(self.file_selected + 1 + j, child);
                }
            }
        }
    }

    pub fn load_stashes(&mut self, path: &str) {
        self.stashes = crate::git::stash::get_stashes(path);
        self.stash_selected = 0;
    }

    pub fn load_stash_detail(&mut self, path: &str) {
        if let Some(stash) = self.stashes.get(self.stash_selected) {
            self.stash_files = crate::git::stash::get_stash_files(path, &stash.id);
            self.stash_diff = crate::git::stash::get_stash_diff(path, &stash.id);
        }
    }

    pub fn setup_ai(&mut self, path: &str) {
        dotenv::dotenv().ok();
        let db_path = std::path::PathBuf::from(path)
            .join(".git")
            .join("twigdrop.db");
        self.db = crate::db::Database::new(db_path).ok();

        let provider_type = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
        let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "llama3".to_string());

        if provider_type == "openai" {
            if let Ok(key) = std::env::var("OPENAI_API_KEY") {
                self.ai_provider =
                    Some(Box::new(crate::ai::openai::OpenAIProvider::new(key, model)));
            }
        } else {
            let url = std::env::var("OLLAMA_URL")
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            self.ai_provider = Some(Box::new(crate::ai::ollama::OllamaProvider::new(url, model)));
        }
    }

    pub async fn trigger_ai_analysis(&mut self, path: &str, branch_name: &str) {
        let hash = crate::git::commands::run_git(path, &["rev-parse", branch_name])
            .trim()
            .to_string();

        // 1. Check Cache
        let cached = self
            .db
            .as_ref()
            .and_then(|db| db.get_analysis(branch_name).ok().flatten())
            .filter(|(h, _, _)| h == &hash);

        if let Some((_, summary, cleanup)) = cached {
            self.ai_analysis = Some(format!(
                "--- CACHED ANALYSIS ---\n\nSummary:\n{}\n\nRecommendation:\n{}",
                summary, cleanup
            ));
            return;
        }

        // 2. Run AI Analysis
        if let Some(provider) = &self.ai_provider {
            self.ai_analysis = Some("Analyzing with AI...".to_string());

            let diff = crate::git::get_branch_info(path, branch_name);
            let summary_res = provider.summarize_diff(&diff).await;
            let cleanup_res = provider.recommend_cleanup(branch_name).await;

            match (summary_res, cleanup_res) {
                (Ok(s), Ok(c)) => {
                    if let Some(db) = &self.db {
                        let _ = db.save_analysis(branch_name, &hash, &s, &c);
                    }
                    self.ai_analysis = Some(format!("Summary:\n{}\n\nRecommendation:\n{}", s, c));
                }
                _ => {
                    self.ai_analysis = Some("AI Analysis failed.".to_string());
                }
            }
        } else {
            self.ai_analysis = Some("No AI Provider configured.".to_string());
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(&idx) = self.filtered_indices.get(self.selected) {
            let name = self.branches[idx].name.clone();
            if self.bulk_selected.contains(&name) {
                self.bulk_selected.remove(&name);
            } else {
                self.bulk_selected.insert(name);
            }
        }
    }
}
