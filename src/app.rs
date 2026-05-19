use crate::models::{Branch, BranchStatus, ConflictBlock, MergeStatus, GutterStatus};
use crate::ui::animations::SnapAnimation;
use std::collections::{HashMap, HashSet};
use std::time::Instant;
use tokio::sync::mpsc;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};

use std::sync::Arc;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum PrimaryMode {
    Branches,
    Files,
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FilePanel {
    Directory,
    Preview,
}

#[derive(Clone, Debug)]
pub struct PreviewState {
    pub file_path: String,
    pub lines: Vec<String>,
    pub highlighted_lines: Vec<Line<'static>>,
    pub cursor_y: usize,
    pub scroll_y: usize,
    pub selection_start: Option<usize>,
    pub selection_end: Option<usize>,
    pub line_diffs: HashMap<usize, GutterStatus>,
}

impl PartialEq for PreviewState {
    fn eq(&self, other: &Self) -> bool {
        self.file_path == other.file_path && 
        self.cursor_y == other.cursor_y && 
        self.scroll_y == other.scroll_y &&
        self.selection_start == other.selection_start &&
        self.selection_end == other.selection_end
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum AppMode {
    Normal,
    Help,
    Manage,
    Filter,
    Diff,
    StashDetail,
    Settings,
    Search,
    CodePreview(PreviewState),
    ConfirmDelete(Vec<String>),
    Message(String),
}

pub struct MergeUpdate {
    pub branch_name: String,
    pub status: MergeStatus,
}

pub struct AIUpdate {
    pub analysis: String,
}

pub struct ConflictResolutionUpdate {
    pub file_path: String,
    pub resolved_content: String,
    pub original_block: String,
}

pub struct FileStatusUpdate {
    pub statuses: HashMap<String, crate::git::files::FileStatus>,
}

pub struct FileState {
    pub file_tree: Vec<crate::git::files::FileEntry>,
    pub file_selected: usize,
    pub file_scroll: usize,
    pub git_file_statuses: HashMap<String, crate::git::files::FileStatus>,
    pub sidebar_width: u16,
    pub active_panel: FilePanel,
    pub status_rx: mpsc::Receiver<FileStatusUpdate>,
    pub open_paths: HashSet<String>,
}

impl FileState {
    pub fn new(status_rx: mpsc::Receiver<FileStatusUpdate>) -> Self {
        Self {
            file_tree: Vec::new(),
            file_selected: 0,
            file_scroll: 0,
            git_file_statuses: HashMap::new(),
            sidebar_width: 30,
            active_panel: FilePanel::Directory,
            status_rx,
            open_paths: HashSet::new(),
        }
    }
}

#[derive(Default)]
pub struct StashState {
    pub stashes: Vec<crate::git::stash::StashEntry>,
    pub stash_selected: usize,
    pub stash_files: Vec<String>,
    pub stash_diff: String,
}

pub struct AIState {
    pub ai_worker: Option<crate::ai::AIWorker>,
    pub db: Option<crate::db::Database>,
    pub ai_analysis: Option<String>,
    pub ai_rx: mpsc::Receiver<AIUpdate>,
    pub ai_trigger_tx: mpsc::Sender<(String, String)>,
    pub conflict_resolution_rx: mpsc::Receiver<ConflictResolutionUpdate>,
    pub conflict_trigger_tx: mpsc::Sender<(String, ConflictBlock)>,
}

#[derive(Default)]
pub struct SettingsState {
    pub selected: usize,
    pub editing: bool,
    pub input: String,
}

pub struct App {
    pub branch_state: BranchState,
    pub file_state: FileState,
    pub stash_state: StashState,
    pub ai_state: AIState,
    pub settings_state: SettingsState,

    pub current_branch: String,
    pub primary_mode: PrimaryMode,
    pub mode: AppMode,

    // UI & System State
    pub last_click_time: Instant,
    pub last_click_row: Option<usize>,
    pub needs_clear: bool,
    pub alt_pressed: bool,
    pub shift_pressed: bool,
    pub config: crate::utils::config::Config,

    // Animations
    pub snap_animation: Option<SnapAnimation>,
    pub branch_screen_positions: Vec<(usize, u16)>, // (branch_index, screen_y)

    // Syntax Highlighting
    pub ps: SyntaxSet,
    pub ts: ThemeSet,

    // Background updates
    pub rx: mpsc::Receiver<MergeUpdate>,
    pub trigger_tx: mpsc::Sender<()>,
    pub shared_primary_mode: Arc<std::sync::RwLock<PrimaryMode>>,
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
    pub branch_info: String,
    pub info_scroll: u16,
    pub search_query: String,
}

impl App {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        repo_path: &str,
        branches: Vec<Branch>,
        current_branch: String,
        rx: mpsc::Receiver<MergeUpdate>,
        trigger_tx: mpsc::Sender<()>,
        ai_rx: mpsc::Receiver<AIUpdate>,
        ai_trigger_tx: mpsc::Sender<(String, String)>,
        conflict_resolution_rx: mpsc::Receiver<ConflictResolutionUpdate>,
        conflict_trigger_tx: mpsc::Sender<(String, ConflictBlock)>,
        file_status_rx: mpsc::Receiver<FileStatusUpdate>,
    ) -> Self {
        let config = crate::utils::config::load_config();
        let primary_mode = if config.last_primary_mode == 1 {
            PrimaryMode::Files
        } else {
            PrimaryMode::Branches
        };

        let shared_primary_mode = Arc::new(std::sync::RwLock::new(primary_mode));

        let mut app = Self {
            branch_state: BranchState {
                branches,
                ..Default::default()
            },
            file_state: FileState::new(file_status_rx),
            stash_state: StashState::default(),
            ai_state: AIState {
                ai_worker: None,
                db: None,
                ai_analysis: None,
                ai_rx,
                ai_trigger_tx,
                conflict_resolution_rx,
                conflict_trigger_tx,
            },
            settings_state: SettingsState::default(),
            current_branch,
            primary_mode,
            mode: AppMode::Normal,
            last_click_time: Instant::now(),
            last_click_row: None,
            needs_clear: false,
            alt_pressed: false,
            shift_pressed: false,
            config,
            snap_animation: None,
            branch_screen_positions: Vec::new(),
            ps: SyntaxSet::load_defaults_newlines(),
            ts: ThemeSet::load_defaults(),
            rx,
            trigger_tx,
            shared_primary_mode,
        };
        
        if app.primary_mode == PrimaryMode::Files {
            app.load_file_tree(repo_path); 
        }
        
        app.refresh_filtered_branches();
        app
    }

    pub fn toggle_primary_mode(&mut self) {
        self.primary_mode = match self.primary_mode {
            PrimaryMode::Branches => PrimaryMode::Files,
            PrimaryMode::Files => PrimaryMode::Branches,
        };
        self.config.last_primary_mode = match self.primary_mode {
            PrimaryMode::Branches => 0,
            PrimaryMode::Files => 1,
        };
        
        // Sync shared mode synchronously
        if let Ok(mut w) = self.shared_primary_mode.write() {
            *w = self.primary_mode;
        }

        crate::utils::config::save_config(&self.config);
    }

    pub fn refresh_branches(&mut self, path: &str) {
        self.branch_state.branches = crate::git::build_branches(path);
        self.refresh_filtered_branches();
        let _ = self.trigger_tx.try_send(());
    }

    pub fn update_from_channel(&mut self, path: &str) {
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
        while let Ok(update) = self.ai_state.conflict_resolution_rx.try_recv() {
            match crate::actions::commands::apply_resolution_to_file(path, &update.file_path, &update.original_block, &update.resolved_content) {
                Ok(_) => {
                    self.mode = AppMode::Message(format!("Fixed conflict in {}", update.file_path));
                }
                Err(e) => {
                    self.mode = AppMode::Message(format!("Error fixing conflict: {}", e));
                }
            }
        }
        while let Ok(update) = self.file_state.status_rx.try_recv() {
            self.update_file_statuses(update.statuses, path);
        }
    }

    pub fn update_file_statuses(&mut self, statuses: HashMap<String, crate::git::files::FileStatus>, repo_path: &str) {
        let mut tree_needs_refresh = false;
        
        // Detect additions, deletions OR renames
        if statuses.len() != self.file_state.git_file_statuses.len() {
            tree_needs_refresh = true;
        } else {
            // Check if any key in the new statuses is missing from our current knowledge
            for path in statuses.keys() {
                if !self.file_state.git_file_statuses.contains_key(path) {
                    tree_needs_refresh = true;
                    break;
                }
            }
        }

        if tree_needs_refresh {
            self.load_file_tree(repo_path);
        }

        self.file_state.git_file_statuses = statuses;
        for entry in self.file_state.file_tree.iter_mut() {
            let rel_path = entry.path.to_string_lossy().to_string();
            entry.status = self.file_state.git_file_statuses.get(&rel_path).cloned().unwrap_or(crate::git::files::FileStatus::Normal);
        }

        // If we are in code preview, check if the current file was modified
        let mut new_preview_data = None;
        if let AppMode::CodePreview(ref mut state) = self.mode {
            let current_path = state.file_path.clone();
            if let Some(status) = self.file_state.git_file_statuses.get(&current_path)
                && *status == crate::git::files::FileStatus::Modified {
                    state.line_diffs = crate::git::get_line_diffs(repo_path, &current_path);
                    
                    let full_path = std::path::Path::new(repo_path).join(&current_path);
                    if let Ok(metadata) = std::fs::metadata(&full_path)
                        && metadata.modified().is_ok() {
                            new_preview_data = Some((current_path, state.cursor_y, state.scroll_y));
                    }
            }
        }

        if let Some((path, cy, sy)) = new_preview_data
            && let Some(mut np) = self.create_preview_state(repo_path, &path)
            && let AppMode::CodePreview(ref mut state) = self.mode {
                np.cursor_y = cy;
                np.scroll_y = sy;
                *state = np;
        }
    }

    pub fn apply_snap_deletion(&mut self, path: &str) -> String {
        if let Some(ref anim) = self.snap_animation {
            let names: Vec<String> = anim.rows.iter().map(|r| r.branch_name.clone()).collect();
            let msg = crate::actions::bulk_delete_branches(path, &names);
            self.refresh_branches(path);
            self.branch_state.bulk_selected.clear();
            msg
        } else {
            String::new()
        }
    }

    pub fn refresh_filtered_branches(&mut self) {
        self.branch_state.filtered_indices = self.branch_state.branches
            .iter()
            .enumerate()
            .filter(|(_, b)| {
                let status_match = if let Some(filter) = &self.branch_state.current_filter {
                    b.status.contains(filter)
                } else {
                    true
                };
                
                let search_match = if !self.branch_state.search_query.is_empty() {
                    b.name.to_lowercase().contains(&self.branch_state.search_query.to_lowercase())
                } else {
                    true
                };
                
                status_match && search_match
            })
            .map(|(i, _)| i)
            .collect();

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
        self.file_state.git_file_statuses = crate::git::files::get_git_file_statuses(path);
        let mut new_tree = Vec::new();
        self.build_tree_recursive(path, "", 0, &mut new_tree);
        self.file_state.file_tree = new_tree;
    }

    fn build_tree_recursive(&self, root: &str, current_dir: &str, depth: usize, tree: &mut Vec<crate::git::files::FileEntry>) {
        let entries = crate::git::files::build_file_tree(
            root,
            current_dir,
            depth,
            &self.file_state.git_file_statuses,
        );

        for mut entry in entries {
            let path_str = entry.path.to_string_lossy().to_string();
            let is_open = self.file_state.open_paths.contains(&path_str);
            entry.is_open = is_open;
            
            tree.push(entry.clone());
            
            if entry.is_dir && is_open {
                self.build_tree_recursive(root, &path_str, depth + 1, tree);
            }
        }
    }

    pub fn toggle_file_dir(&mut self, _path_str: &str) {
        if self.file_state.file_selected >= self.file_state.file_tree.len() {
            return;
        }

        let entry = &self.file_state.file_tree[self.file_state.file_selected];
        if !entry.is_dir { return; }
        
        let path = entry.path.to_string_lossy().to_string();
        if self.file_state.open_paths.contains(&path) {
            self.file_state.open_paths.remove(&path);
        } else {
            self.file_state.open_paths.insert(path);
        }
        
        // Rebuild tree to reflect change
        let repo_path = _path_str.to_string();
        let mut new_tree = Vec::new();
        self.build_tree_recursive(&repo_path, "", 0, &mut new_tree);
        self.file_state.file_tree = new_tree;
    }

    pub fn load_stashes(&mut self, path: &str) {
        self.stash_state.stashes = crate::git::stash::get_stashes(path);
        self.stash_state.stash_selected = 0;
    }

    pub fn load_stash_detail(&mut self, path: &str) {
        if let Some(stash) = self.stash_state.stashes.get(self.stash_state.stash_selected) {
            self.stash_state.stash_files = crate::git::stash::get_stash_files(path, &stash.id);
            self.stash_state.stash_diff = crate::git::stash::get_stash_diff(path, &stash.id);
        }
    }

    pub fn setup_ai(&mut self, _path: &str) {
        dotenv::dotenv().ok();
        let db_path = crate::utils::config::get_config_path()
            .unwrap_or_else(|| std::path::PathBuf::from(".git"))
            .parent()
            .unwrap_or(&std::path::PathBuf::from("."))
            .join("twigdrop.db");

        self.ai_state.db = crate::db::Database::new(db_path).ok();

        let provider_type = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
        let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "llama3".to_string());
        let api_key = std::env::var("OPENAI_API_KEY").ok();
        let url = std::env::var("OLLAMA_URL").ok();

        self.ai_state.ai_worker = crate::ai::AIWorker::new(&provider_type, &model, api_key, url).ok();
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

    pub fn create_preview_state(&self, repo_path: &str, rel_path: &str) -> Option<PreviewState> {
        let full_path = std::path::Path::new(repo_path).join(rel_path);
        
        let file = std::fs::File::open(&full_path).ok()?;
        let reader = std::io::BufReader::new(file);
        use std::io::BufRead;
        
        let mut lines = Vec::new();
        let max_lines = 5000;
        for (i, line) in reader.lines().enumerate() {
            if i >= max_lines { break; }
            if let Ok(l) = line {
                lines.push(l);
            }
        }

        let line_diffs = crate::git::get_line_diffs(repo_path, rel_path);
        
        let mut state = PreviewState {
            file_path: rel_path.to_string(),
            lines,
            highlighted_lines: Vec::new(),
            cursor_y: 0,
            scroll_y: 0,
            selection_start: None,
            selection_end: None,
            line_diffs,
        };

        self.update_preview_highlighting(&mut state);
        
        Some(state)
    }

    pub fn update_preview_highlighting(&self, state: &mut PreviewState) {
        if state.lines.is_empty() { return; }
        
        let extension = std::path::Path::new(&state.file_path).extension().and_then(|s| s.to_str()).unwrap_or("");
        let syntax = self.ps.find_syntax_by_extension(extension)
            .or_else(|| self.ps.find_syntax_for_file(&state.file_path).unwrap_or(None))
            .unwrap_or_else(|| self.ps.find_syntax_plain_text());

        let theme = &self.ts.themes["base16-ocean.dark"];
        let mut h = syntect::easy::HighlightLines::new(syntax, theme);
        
        state.highlighted_lines.clear();
        for line in &state.lines {
            let line_with_ending = format!("{}\n", line);
            let ranges = h.highlight_line(&line_with_ending, &self.ps).unwrap_or_default();
            let mut spans = Vec::new();

            for (style, text) in ranges {
                let color = Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                let content = text.trim_end_matches(['\n', '\r']);
                if !content.is_empty() || text.is_empty() {
                    spans.push(Span::styled(content.to_string(), Style::default().fg(color)));
                }
            }
            state.highlighted_lines.push(Line::from(spans));
        }
    }
}
