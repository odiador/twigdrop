use super::App;

pub struct AIState {
    pub db: Option<crate::db::Database>,
    pub ai_analysis: Option<String>,
    /// True between AI request dispatch and completion/failure.
    /// Drives the status-bar spinner together with `snap_animation`.
    pub ai_pending: bool,
}

impl App {
    /// Mark an AI request as dispatched (spinner on).
    pub fn mark_ai_dispatched(&mut self) {
        self.ai_state.ai_pending = true;
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
}
