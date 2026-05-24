use crate::models::MergeStatus;
use crate::git::files::FileStatus;
use std::collections::HashMap;

#[derive(Debug)]
pub enum GitEvent {
    MergeStatusUpdated { branch: String, status: MergeStatus },
    FileStatusesUpdated(HashMap<String, FileStatus>),
}

#[derive(Debug)]
pub enum TaskEvent {
    AiAnalysisComplete(String),
    AiModelsFetched(Vec<String>),
    HighlightingComplete(String, Vec<ratatui::text::Line<'static>>),
    TaskFailed(String),
    ConflictResolved { file_path: String, original_block: String, resolved_content: String },
}

#[derive(Debug, Clone)]
pub enum Event {
    Key(ratatui::crossterm::event::KeyEvent),
    Mouse(ratatui::crossterm::event::MouseEvent),
    Resize,
    Git(GitEvent),
    Task(TaskEvent),
}

impl Clone for GitEvent {
    fn clone(&self) -> Self {
        match self {
            GitEvent::MergeStatusUpdated { branch, status } => GitEvent::MergeStatusUpdated { branch: branch.clone(), status: status.clone() },
            GitEvent::FileStatusesUpdated(f) => GitEvent::FileStatusesUpdated(f.clone()),
        }
    }
}

impl Clone for TaskEvent {
    fn clone(&self) -> Self {
        match self {
            TaskEvent::AiAnalysisComplete(a) => TaskEvent::AiAnalysisComplete(a.clone()),
            TaskEvent::AiModelsFetched(m) => TaskEvent::AiModelsFetched(m.clone()),
            TaskEvent::HighlightingComplete(p, l) => TaskEvent::HighlightingComplete(p.clone(), l.clone()),
            TaskEvent::TaskFailed(e) => TaskEvent::TaskFailed(e.clone()),
            TaskEvent::ConflictResolved { file_path, original_block, resolved_content } => TaskEvent::ConflictResolved { 
                file_path: file_path.clone(), 
                original_block: original_block.clone(), 
                resolved_content: resolved_content.clone() 
            },
        }
    }
}
