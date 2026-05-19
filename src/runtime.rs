use tokio::sync::mpsc;
use crate::app::{AIUpdate, ConflictResolutionUpdate, MergeUpdate, FileStatusUpdate, PrimaryMode};
use crate::git;
use crate::models::ConflictBlock;
use std::sync::Arc;

pub struct Runtime {
    pub repo_path: String,
}

impl Runtime {
    pub fn new(repo_path: &str) -> Self {
        Self {
            repo_path: repo_path.to_string(),
        }
    }

    pub fn spawn_merge_analyzer(&self, mut trigger_rx: mpsc::Receiver<()>, tx: mpsc::Sender<MergeUpdate>) {
        let path = self.repo_path.clone();
        tokio::spawn(async move {
            while trigger_rx.recv().await.is_some() {
                let branches = git::build_branches(&path);
                let current_branch = git::get_current_branch(&path);

                for branch in branches {
                    let tx_clone = tx.clone();
                    let p = path.clone();
                    let b = branch.name.clone();
                    let cb = current_branch.clone();
                    tokio::spawn(async move {
                        let status = git::analyze_merge_status(&p, &b, &cb);
                        let _ = tx_clone.send(MergeUpdate {
                            branch_name: b,
                            status,
                        }).await;
                    });
                }
            }
        });
    }
pub fn spawn_file_status_poller(&self, file_status_tx: mpsc::Sender<FileStatusUpdate>, app_mode_rx: Arc<std::sync::RwLock<PrimaryMode>>) {
        let path = self.repo_path.clone();
        tokio::spawn(async move {
            loop {
                // Read current primary mode
                let mode = {
                    let r = app_mode_rx.read().unwrap_or_else(|e| e.into_inner());
                    *r
                };

                if mode == PrimaryMode::Files {
                    let statuses = git::files::get_git_file_statuses(&path);
                    let _ = file_status_tx.send(FileStatusUpdate { statuses }).await;
                }

                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
        });
    }


    pub fn spawn_ai_worker(
        &self, 
        mut ai_trigger_rx: mpsc::Receiver<(String, String)>, 
        ai_update_tx: mpsc::Sender<AIUpdate>,
        mut conflict_trigger_rx: mpsc::Receiver<(String, ConflictBlock)>,
        conflict_resolution_tx: mpsc::Sender<ConflictResolutionUpdate>,
    ) {
        tokio::spawn(async move {
            dotenv::dotenv().ok();
            let db_path = crate::utils::config::get_config_path()
                .unwrap_or_else(|| std::path::PathBuf::from(".git"))
                .parent()
                .unwrap_or(&std::path::PathBuf::from("."))
                .join("twigdrop.db");

            let db = crate::db::Database::new(db_path).ok();

            let provider_type = std::env::var("AI_PROVIDER").unwrap_or_else(|_| "ollama".to_string());
            let model = std::env::var("AI_MODEL").unwrap_or_else(|_| "llama3".to_string());
            let api_key = std::env::var("OPENAI_API_KEY").ok();
            let url = std::env::var("OLLAMA_URL").ok();

            let worker = crate::ai::AIWorker::new(&provider_type, &model, api_key, url).ok();

            if let Some(w) = worker {
                loop {
                    tokio::select! {
                        Some((repo_path, branch_name)) = ai_trigger_rx.recv() => {
                            let hash = match git::commands::run_git(&repo_path, &["rev-parse", &branch_name]) {
                                Ok(h) => h.trim().to_string(),
                                Err(_) => continue,
                            };

                            let mut cached_result = None;
                            if let Some(ref d) = db
                                && let Ok(Some((cached_hash, summary, cleanup))) = d.get_analysis(&branch_name)
                                && cached_hash == hash
                            {
                                cached_result = Some(format!("--- CACHED ANALYSIS ---\n\nSummary:\n{}\n\nRecommendation:\n{}", summary, cleanup));
                            }

                            if let Some(analysis) = cached_result {
                                let _ = ai_update_tx.send(AIUpdate { analysis }).await;
                            } else {
                                let _ = ai_update_tx.send(AIUpdate { analysis: "Analyzing with AI...".to_string() }).await;
                                let diff = git::get_branch_info(&repo_path, &branch_name);
                                let summary_res = w.inner.summarize_diff(&diff).await;
                                let cleanup_res = w.inner.recommend_cleanup(&branch_name).await;

                                match (summary_res, cleanup_res) {
                                    (Ok(s), Ok(c)) => {
                                        if let Some(ref d) = db {
                                            let _ = d.save_analysis(&branch_name, &hash, &s, &c);
                                        }
                                        let _ = ai_update_tx.send(AIUpdate {
                                            analysis: format!("Summary:\n{}\n\nRecommendation:\n{}", s, c),
                                        }).await;
                                    }
                                    _ => {
                                        let _ = ai_update_tx.send(AIUpdate { analysis: "AI Analysis failed.".to_string() }).await;
                                    }
                                }
                            }
                        }
                        Some((_repo_path, conflict)) = conflict_trigger_rx.recv() => {
                            let resolution = w.inner.resolve_conflict(&conflict.content).await;
                            if let Ok(resolved_content) = resolution {
                                let _ = conflict_resolution_tx.send(ConflictResolutionUpdate {
                                    file_path: conflict.file_path,
                                    resolved_content,
                                    original_block: conflict.content,
                                }).await;
                            }
                        }
                        else => break,
                    }
                }
            }
        });
    }
}
