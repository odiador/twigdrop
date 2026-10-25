pub mod ai;
pub mod branches;
pub mod deletion;
pub mod files;
pub mod preview;
pub mod pull_requests;
pub mod stash_rebase;

pub use ai::AIState;
pub use branches::INSPECTOR_COMMIT_LIMIT;

use crate::events::{Event, GitEvent, TaskEvent};
use crate::models::{Branch, ConflictBlock};
use crate::state::ui::RebaseAction;
use crate::state::{AppMode, FilePanel, FrameCache, PrimaryMode, RepositoryState, UiState};
use std::cell::RefCell;
use std::sync::Arc;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tokio::sync::mpsc;

pub struct App {
    pub repo: RepositoryState,
    pub ui: UiState,
    pub ai_state: AIState,
    /// Per-frame render cache (scroll position, mouse hit rows).
    /// Behind a `RefCell` so `ui::render` only needs `&App`.
    pub frame_cache: RefCell<FrameCache>,

    pub config: crate::utils::config::Config,

    pub ps: Arc<SyntaxSet>,
    pub ts: Arc<ThemeSet>,
    pub event_tx: Option<mpsc::Sender<Event>>,

    pub shared_primary_mode: Arc<std::sync::RwLock<PrimaryMode>>,
    pub trigger_tx: mpsc::Sender<()>,
    pub ai_trigger_tx: mpsc::Sender<(String, String, String)>,
    pub conflict_trigger_tx: mpsc::Sender<(String, ConflictBlock)>,
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
        let primary_mode = match config.last_primary_mode {
            1 => PrimaryMode::Files,
            2 => PrimaryMode::Commits,
            3 => PrimaryMode::Stashes,
            4 => PrimaryMode::PullRequests,
            _ => PrimaryMode::Branches,
        };

        let shared_primary_mode = Arc::new(std::sync::RwLock::new(primary_mode));

        let mut app = Self {
            repo: RepositoryState::new(current_branch, branches),
            ui: UiState::new(
                primary_mode,
                u16::try_from(config.default_sidebar_width).unwrap_or(30),
            ),
            ai_state: AIState {
                db: None,
                ai_analysis: None,
                ai_pending: false,
            },
            frame_cache: RefCell::new(FrameCache::default()),
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
        } else if app.ui.primary_mode == PrimaryMode::Commits {
            app.repo.commit_tree = crate::git::commands::get_commit_tree(repo_path);
        } else if app.ui.primary_mode == PrimaryMode::Stashes {
            app.load_stashes(repo_path);
            app.load_stash_detail(repo_path);
        } else if app.ui.primary_mode == PrimaryMode::PullRequests {
            app.load_pull_requests(repo_path);
        }

        app.refresh_filtered_branches();
        app
    }

    pub fn toggle_primary_mode(&mut self) {
        let next_mode = match self.ui.primary_mode {
            PrimaryMode::Branches => PrimaryMode::Files,
            PrimaryMode::Files => PrimaryMode::Commits,
            PrimaryMode::Commits => PrimaryMode::Stashes,
            PrimaryMode::Stashes => PrimaryMode::PullRequests,
            PrimaryMode::PullRequests => PrimaryMode::Branches,
        };
        self.set_primary_mode(next_mode);
    }

    pub fn set_primary_mode(&mut self, mode: PrimaryMode) {
        if self.ui.primary_mode != mode && matches!(self.ui.current_mode(), AppMode::CodePreview(_))
        {
            self.ui.pop_modal();
            self.ui.active_panel = FilePanel::Directory;
        }
        self.ui.primary_mode = mode;
        self.config.last_primary_mode = match mode {
            PrimaryMode::Branches => 0,
            PrimaryMode::Files => 1,
            PrimaryMode::Commits => 2,
            PrimaryMode::Stashes => 3,
            PrimaryMode::PullRequests => 4,
        };

        if let Ok(mut w) = self.shared_primary_mode.write() {
            *w = mode;
        }

        crate::utils::config::save_config(&self.config);
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
                        entry.status = self
                            .repo
                            .git_file_statuses
                            .get(&rel_path)
                            .cloned()
                            .unwrap_or(crate::git::files::FileStatus::Normal);
                    }
                }
            },
            Event::Task(task_event) => match task_event {
                TaskEvent::HighlightingComplete(path, lines) => {
                    let updated = self.ui.modal_stack.iter_mut().any(|modal| {
                        if let AppMode::CodePreview(ref mut state) = modal.mode
                            && state.file_path == path
                        {
                            state.highlighted_lines = lines.clone();
                            return true;
                        }
                        false
                    });
                    if !updated
                        && let AppMode::CodePreview(ref mut state) = self.ui.mode
                        && state.file_path == path
                    {
                        state.highlighted_lines = lines;
                    }
                }
                TaskEvent::AiAnalysisComplete(analysis) => {
                    self.ai_state.ai_pending = false;
                    if analysis.starts_with("Commit Msg Suggestion:\n") {
                        if matches!(self.ui.current_mode(), AppMode::InteractiveRebase) {
                            let msg = analysis
                                .replace("Commit Msg Suggestion:\n", "")
                                .trim()
                                .to_string();
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
                TaskEvent::ConflictResolved {
                    file_path,
                    original_block,
                    resolved_content,
                } => {
                    let _ = crate::actions::commands::apply_resolution_to_file(
                        ".",
                        &file_path,
                        &original_block,
                        &resolved_content,
                    );
                    self.ui
                        .push_modal(AppMode::Message(format!("Fixed conflict in {file_path}")));
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
                    self.ai_state.ai_pending = false;
                    self.ui
                        .push_modal(AppMode::Message(format!("Task Error: {err}")));
                }
            },
            Event::Resize => {
                self.ui.needs_clear = true;
            }
            _ => {}
        }
    }

    pub fn locale(&self) -> &'static crate::i18n::Locale {
        self.config.locale()
    }

    /// Active color scheme from config (`App::theme`, never `dark_default`
    /// directly in render paths).
    pub fn theme(&self) -> crate::ui::theme::Theme {
        self.config.theme_mode.resolve()
    }

    /// Background work in flight (AI request or delete animation).
    /// Drives the status-bar spinner.
    pub fn is_busy(&self) -> bool {
        self.ai_state.ai_pending
            || self.ui.rebase_state.ai_analyzing
            || self.ui.snap_animation.is_some()
    }
}
