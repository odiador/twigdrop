#![allow(dead_code)]
use crate::models::{Branch, Commit, MergeStatus};
use crate::git::files::FileStatus;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug)]
pub enum GitEvent {
    BranchesUpdated(Vec<Branch>),
    MergeStatusUpdated { branch: String, status: MergeStatus },
    FileStatusesUpdated(HashMap<String, FileStatus>),
    CommitsUpdated(Vec<Commit>),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum TaskEvent {
    AiAnalysisComplete(String),
    ConflictResolved { file_path: String, original_block: String, resolved_content: String },
    AiModelsFetched(Vec<String>),
    HighlightingComplete(String, Vec<ratatui::text::Line<'static>>),
    TaskFailed(String),
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Event {
    Key(ratatui::crossterm::event::KeyEvent),
    Mouse(ratatui::crossterm::event::MouseEvent),
    Resize(u16, u16),
    Git(GitEvent),
    Task(TaskEvent),
    Tick,
    // Add custom application-level events here as needed
    Action(crate::actions::ActionId),
}
