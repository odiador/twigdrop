use crate::models::{Branch, ConflictBlock};
use crate::state::{RepositoryState, UiState, AppMode, PrimaryMode, PreviewState};
use crate::state::ui::RebaseAction;
use tokio::sync::mpsc;
use syntect::parsing::SyntaxSet;
use syntect::highlighting::ThemeSet;
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};
use std::sync::Arc;
use crate::events::{Event, TaskEvent, GitEvent};

pub struct App {
    pub repo: RepositoryState,
    pub ui: UiState,
    pub ai_state: AIState,

    pub config: crate::utils::config::Config,

    pub ps: Arc<SyntaxSet>,
    pub ts: Arc<ThemeSet>,
    pub event_tx: Option<mpsc::Sender<Event>>,

    pub shared_primary_mode: Arc<std::sync::RwLock<PrimaryMode>>,
    pub trigger_tx: mpsc::Sender<()>,
    pub ai_trigger_tx: mpsc::Sender<(String, String, String)>,
    pub conflict_trigger_tx: mpsc::Sender<(String, ConflictBlock)>,
}

pub struct AIState {
    pub ai_worker: Option<crate::ai::AIWorker>,
    pub db: Option<crate::db::Database>,
    pub ai_analysis: Option<String>,
}

impl App {
    pub fn new(
        repo_path: &str,
        branches: Vec<Branch>,
        current_branch: String,
        trigger_tx: mpsc::Sender<()>,
        ai_trigger_tx: mpsc::Sender<(String, String, String)>,
        conflict_trigger_tx: mpsc::Sender<(String, ConflictBlock)>,
    ) -> Self {
        let config = crate::utils::config::load_config();
        let primary_mode = if config.last_primary_mode == 1 {
            PrimaryMode::Files
        } else {
            PrimaryMode::Branches
        };

        let shared_primary_mode = Arc::new(std::sync::RwLock::new(primary_mode));

        let mut app = Self {
            repo: RepositoryState::new(current_branch, branches),
            ui: UiState::new(primary_mode, config.default_sidebar_width as u16),
            ai_state: AIState {
                ai_worker: None,
                db: None,
                ai_analysis: None,
            },
            config,
            ps: Arc::new(SyntaxSet::load_defaults_newlines()),
            ts: Arc::new(ThemeSet::load_defaults()),
            event_tx: None,
            trigger_tx,
            shared_primary_mode,
            ai_trigger_tx,
            conflict_trigger_tx,
        };
        
        if app.ui.primary_mode == PrimaryMode::Files {
            app.load_file_tree(repo_path); 
        }
        
        app.refresh_filtered_branches();
        app
    }

    pub fn toggle_primary_mode(&mut self) {
        self.ui.primary_mode = match self.ui.primary_mode {
            PrimaryMode::Branches => PrimaryMode::Files,
            PrimaryMode::Files => PrimaryMode::Branches,
        };
        self.config.last_primary_mode = match self.ui.primary_mode {
            PrimaryMode::Branches => 0,
            PrimaryMode::Files => 1,
        };
        
        if let Ok(mut w) = self.shared_primary_mode.write() {
            *w = self.ui.primary_mode;
        }

        crate::utils::config::save_config(&self.config);
    }

    pub fn refresh_branches(&mut self, path: &str) {
        self.repo.branches = crate::git::build_branches(path);
        self.refresh_filtered_branches();
        let _ = self.trigger_tx.try_send(());
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::Git(git_event) => match git_event {
                GitEvent::MergeStatusUpdated { branch, status } => {
                    if let Some(b) = self.repo.branches.iter_mut().find(|b| b.name == branch) {
                        b.merge_status = status;
                    }
                }
                GitEvent::FileStatusesUpdated(statuses) => {
                    self.repo.git_file_statuses = statuses;
                    for entry in self.repo.file_tree.iter_mut() {
                        let rel_path = entry.path.to_string_lossy().to_string().replace('\\', "/");
                        entry.status = self.repo.git_file_statuses.get(&rel_path).cloned().unwrap_or(crate::git::files::FileStatus::Normal);
                    }
                }
            },
            Event::Task(task_event) => match task_event {
                TaskEvent::HighlightingComplete(path, lines) => {
                    if let AppMode::CodePreview(ref mut state) = self.ui.mode {
                        if state.file_path == path {
                            state.highlighted_lines = lines;
                        }
                    }
                }
                TaskEvent::AiAnalysisComplete(analysis) => {
                    if analysis.starts_with("Commit Msg Suggestion:\n") {
                        if let AppMode::InteractiveRebase = self.ui.mode {
                            let msg = analysis.replace("Commit Msg Suggestion:\n", "").trim().to_string();
                            if self.ui.rebase_state.ai_analyzing {
                                self.ui.rebase_state.ai_analyzing = false;
                                let i = self.ui.rebase_state.selected;
                                self.ui.rebase_state.commits[i].new_message = Some(msg);
                                self.ui.rebase_state.commits[i].action = RebaseAction::Reword;
                            }
                        }
                    } else {
                        self.ai_state.ai_analysis = Some(analysis);
                    }
                }
                TaskEvent::ConflictResolved { file_path, original_block, resolved_content } => {
                    let _ = crate::actions::commands::apply_resolution_to_file(".", &file_path, &original_block, &resolved_content);
                    self.ui.mode = AppMode::Message(format!("Fixed conflict in {}", file_path));
                }
                TaskEvent::AiModelsFetched(models) => {
                    if self.ui.settings_state.selecting && self.ui.settings_state.selected == 3 {
                        self.ui.settings_state.choices = models;
                        if self.ui.settings_state.choices.is_empty() {
                            self.ui.settings_state.choices = vec!["No models found".to_string()];
                        }
                    }
                }
                TaskEvent::TaskFailed(err) => {
                    self.ui.mode = AppMode::Message(format!("Task Error: {}", err));
                }
            },
            Event::Resize => {
                self.ui.needs_clear = true;
            }
            _ => {}
        }
    }

    pub fn apply_snap_deletion(&mut self, path: &str) -> String {
        if let Some(ref anim) = self.ui.snap_animation {
            let names: Vec<String> = anim.rows.iter().map(|r| r.branch_name.clone()).collect();
            let msg = crate::actions::bulk_delete_branches(path, &names);
            self.refresh_branches(path);
            self.ui.bulk_selected.clear();
            msg
        } else {
            String::new()
        }
    }

    pub fn refresh_filtered_branches(&mut self) {
        self.ui.filtered_indices = self.repo.branches
            .iter()
            .enumerate()
            .filter(|(_, b)| {
                let status_match = if let Some(filter) = &self.ui.current_filter {
                    b.status.contains(filter)
                } else {
                    true
                };
                
                let search_match = if !self.ui.search_query.is_empty() {
                    b.name.to_lowercase().contains(&self.ui.search_query.to_lowercase())
                } else {
                    true
                };
                
                status_match && search_match
            })
            .map(|(i, _)| i)
            .collect();

        let max = self.ui.filtered_indices.len().saturating_sub(1);
        if self.ui.selected_branch_idx > max {
            self.ui.selected_branch_idx = max;
        }
    }

    pub fn get_filtered_branches(&self) -> Vec<&Branch> {
        self.ui
            .filtered_indices
            .iter()
            .map(|&i| &self.repo.branches[i])
            .collect()
    }

    pub fn next(&mut self) {
        match self.ui.primary_mode {
            PrimaryMode::Branches => {
                let max = self.ui.filtered_indices.len().saturating_sub(1);
                if self.ui.selected_branch_idx < max {
                    self.ui.selected_branch_idx += 1;
                }
            }
            PrimaryMode::Files => {
                if self.ui.selected_file_idx < self.repo.file_tree.len().saturating_sub(1)
                {
                    self.ui.selected_file_idx += 1;
                }
            }
        }
    }

    pub fn previous(&mut self) {
        match self.ui.primary_mode {
            PrimaryMode::Branches => {
                if self.ui.selected_branch_idx > 0 {
                    self.ui.selected_branch_idx -= 1;
                }
            }
            PrimaryMode::Files => {
                if self.ui.selected_file_idx > 0 {
                    self.ui.selected_file_idx -= 1;
                }
            }
        }
    }

    pub fn load_file_tree(&mut self, path: &str) {
        self.repo.git_file_statuses = crate::git::files::get_git_file_statuses(path);
        let mut new_tree = Vec::new();
        self.build_tree_recursive(path, "", 0, &mut new_tree);
        self.repo.file_tree = new_tree;
    }

    fn build_tree_recursive(&self, root: &str, current_dir: &str, depth: usize, tree: &mut Vec<crate::git::files::FileEntry>) {
        let entries = crate::git::files::build_file_tree(
            root,
            current_dir,
            depth,
            &self.repo.git_file_statuses,
        );

        for mut entry in entries {
            let path_str = entry.path.to_string_lossy().to_string();
            let is_open = self.ui.open_paths.contains(&path_str);
            entry.is_open = is_open;
            
            tree.push(entry.clone());
            
            if entry.is_dir && is_open {
                self.build_tree_recursive(root, &path_str, depth + 1, tree);
            }
        }
    }

    pub fn toggle_file_dir(&mut self, _path_str: &str) {
        if self.ui.selected_file_idx >= self.repo.file_tree.len() {
            return;
        }

        let entry = &self.repo.file_tree[self.ui.selected_file_idx];
        if !entry.is_dir { return; }
        
        let path = entry.path.to_string_lossy().to_string();
        if self.ui.open_paths.contains(&path) {
            self.ui.open_paths.remove(&path);
        } else {
            self.ui.open_paths.insert(path);
        }
        
        let repo_path = _path_str.to_string();
        let mut new_tree = Vec::new();
        self.build_tree_recursive(&repo_path, "", 0, &mut new_tree);
        self.repo.file_tree = new_tree;
    }

    pub fn load_stashes(&mut self, path: &str) {
        self.repo.stashes = crate::git::stash::get_stashes(path);
        self.ui.selected_stash_idx = 0;
    }

    pub fn load_rebase_commits(&mut self, path: &str) {
        let commits = crate::git::get_unpushed_commits(path);
        self.ui.rebase_state.commits = commits.into_iter().map(|c| crate::state::ui::RebaseCommit {
            hash: c.hash,
            original_message: c.message,
            new_message: None,
            action: RebaseAction::Pick,
        }).collect();
        self.ui.rebase_state.selected = 0;
        self.ui.rebase_state.editing = false;
        self.ui.rebase_state.input.clear();
    }

    pub fn load_stash_detail(&mut self, path: &str) {
        if let Some(stash) = self.repo.stashes.get(self.ui.selected_stash_idx) {
            self.repo.stash_files = crate::git::stash::get_stash_files(path, &stash.id);
            self.repo.stash_diff = crate::git::stash::get_stash_diff(path, &stash.id);
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
    }

    pub fn toggle_selection(&mut self) {
        if let Some(&idx) = self
            .ui
            .filtered_indices
            .get(self.ui.selected_branch_idx)
        {
            let name = self.repo.branches[idx].name.clone();
            if self.ui.bulk_selected.contains(&name) {
                self.ui.bulk_selected.remove(&name);
            } else {
                self.ui.bulk_selected.insert(name);
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

        if let Some(tx) = &self.event_tx {
            crate::tasks::highlighting::spawn_highlight_task(
                tx.clone(),
                state.file_path.clone(),
                state.lines.clone(),
                self.ps.clone(),
                self.ts.clone(),
            );
        } else {
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

    pub fn update_diff_highlighting(&self, state: &mut PreviewState) {
        state.highlighted_lines.clear();
        for line in &state.lines {
            let color = if line.starts_with('+') && !line.starts_with("+++") {
                Color::Rgb(161, 239, 173) // Light Green
            } else if line.starts_with('-') && !line.starts_with("---") {
                Color::Rgb(245, 194, 231) // Pinkish Red
            } else if line.starts_with("@@") {
                Color::Rgb(137, 180, 250) // Blue for hunk headers
            } else if line.starts_with("diff --git") {
                Color::Rgb(249, 226, 175) // Yellow for file headers
            } else if line.starts_with("index ") {
                Color::Rgb(148, 156, 187) // Gray for index info
            } else if line.starts_with("+++") || line.starts_with("---") {
                Color::Rgb(180, 190, 254) // Lavender for file paths
            } else {
                Color::Rgb(205, 214, 244) // Default text color
            };

            let spans = vec![Span::styled(line.to_string(), Style::default().fg(color))];
            
            if line.starts_with("@@") {
                state.highlighted_lines.push(Line::from(vec![Span::styled(" ".repeat(100), Style::default().bg(Color::Rgb(30, 30, 46)))]));
            }

            state.highlighted_lines.push(Line::from(spans));
        }
    }
}
