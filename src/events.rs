use crate::models::{Branch, Commit, MergeStatus};
use crate::git::files::FileStatus;
use std::collections::HashMap;
use crate::actions::ActionId;

#[derive(Debug)]
pub enum GitEvent {
    BranchesUpdated(Vec<Branch>),
    MergeStatusUpdated { branch: String, status: MergeStatus },
    FileStatusesUpdated(HashMap<String, FileStatus>),
    CommitsUpdated(Vec<Commit>),
}

#[derive(Debug)]
pub enum TaskEvent {
    AiAnalysisComplete(String),
    ConflictResolved { file_path: String, original_block: String, resolved_content: String },
    AiModelsFetched(Vec<String>),
    HighlightingComplete(String, Vec<ratatui::text::Line<'static>>),
    TaskFailed(String),
}

#[derive(Debug, Clone)]
pub enum Event {
    Key(ratatui::crossterm::event::KeyEvent),
    Mouse(ratatui::crossterm::event::MouseEvent),
    Resize(u16, u16),
    Git(GitEvent),
    Task(TaskEvent),
    Tick,
    Action(ActionId),
}

impl Clone for GitEvent {
    fn clone(&self) -> Self {
        match self {
            GitEvent::BranchesUpdated(b) => GitEvent::BranchesUpdated(b.clone()),
            GitEvent::MergeStatusUpdated { branch, status } => GitEvent::MergeStatusUpdated { branch: branch.clone(), status: status.clone() },
            GitEvent::FileStatusesUpdated(f) => GitEvent::FileStatusesUpdated(f.clone()),
            GitEvent::CommitsUpdated(c) => GitEvent::CommitsUpdated(c.clone()),
        }
    }
}

impl Clone for TaskEvent {
    fn clone(&self) -> Self {
        match self {
            TaskEvent::AiAnalysisComplete(a) => TaskEvent::AiAnalysisComplete(a.clone()),
            TaskEvent::ConflictResolved { file_path, original_block, resolved_content } => TaskEvent::ConflictResolved { 
                file_path: file_path.clone(), 
                original_block: original_block.clone(), 
                resolved_content: resolved_content.clone() 
            },
            TaskEvent::AiModelsFetched(m) => TaskEvent::AiModelsFetched(m.clone()),
            TaskEvent::HighlightingComplete(p, l) => TaskEvent::HighlightingComplete(p.clone(), l.clone()),
            TaskEvent::TaskFailed(e) => TaskEvent::TaskFailed(e.clone()),
        }
    }
}
