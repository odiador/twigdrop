mod actions;
mod ai;
mod app;
mod db;
mod git;
mod handlers;
mod models;
mod ui;
mod utils;

use anyhow::Result;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
            PushKeyboardEnhancementFlags,
        },
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};
use std::{env, io};
use tokio::sync::mpsc;

use app::{AIUpdate, App, ConflictResolutionUpdate, MergeUpdate};
use handlers::handle_event;
use models::ConflictBlock;

#[tokio::main]
async fn main() -> Result<()> {
    // Set panic hook to ensure terminal is restored
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        default_panic(info);
    }));

    let args: Vec<String> = env::args().collect();
    let path = args.get(1).cloned().unwrap_or_else(|| ".".to_string());

    let branches = git::build_branches(&path);
    let current_branch = git::get_current_branch(&path);

    let (tx, rx) = mpsc::channel(100);
    let (trigger_tx, mut trigger_rx) = mpsc::channel::<()>(1);
    
    let (ai_update_tx, ai_rx) = mpsc::channel::<AIUpdate>(10);
    let (ai_trigger_tx, mut ai_trigger_rx) = mpsc::channel::<(String, String)>(10);
    
    let (conflict_resolution_tx, conflict_resolution_rx) = mpsc::channel::<ConflictResolutionUpdate>(10);
    let (conflict_trigger_tx, mut conflict_trigger_rx) = mpsc::channel::<(String, ConflictBlock)>(10);

    let (file_status_tx, file_status_rx) = mpsc::channel::<app::FileStatusUpdate>(10);

    let mut app = App::new(
        &path,
        branches,
        current_branch,
        rx,
        trigger_tx.clone(),
        ai_rx,
        ai_trigger_tx.clone(),
        conflict_resolution_rx,
        conflict_trigger_tx.clone(),
        file_status_rx,
    );
    app.setup_ai(&path);

    // Background file status poller
    let path_clone_files = path.clone();
    let file_status_tx_clone = file_status_tx.clone();
    tokio::spawn(async move {
        loop {
            let statuses = crate::git::files::get_git_file_statuses(&path_clone_files);
            let _ = file_status_tx_clone.send(app::FileStatusUpdate { statuses }).await;
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        }
    });

    // Background merge analyzer
    let path_clone = path.clone();
    let tx_clone = tx.clone();

    // Initial trigger
    let _ = trigger_tx.try_send(());

    tokio::spawn(async move {
        while trigger_rx.recv().await.is_some() {
            let branches = git::build_branches(&path_clone);
            let current_branch = git::get_current_branch(&path_clone);

            for branch in branches {
                let tx = tx_clone.clone();
                let p = path_clone.clone();
                let b = branch.name.clone();
                let cb = current_branch.clone();
                tokio::spawn(async move {
                    let status = git::analyze_merge_status(&p, &b, &cb);
                    let _ = tx
                        .send(MergeUpdate {
                            branch_name: b,
                            status,
                        })
                        .await;
                });
            }
        }
    });

    // Background AI analyzer using 'rig'
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
                        let hash =
                            match crate::git::commands::run_git(&repo_path, &["rev-parse", &branch_name]) {
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
                            let _ = ai_update_tx
                                .send(AIUpdate {
                                    analysis: "Analyzing with AI...".to_string(),
                                })
                                .await;
                            let diff = crate::git::get_branch_info(&repo_path, &branch_name);
                            let summary_res = w.inner.summarize_diff(&diff).await;
                            let cleanup_res = w.inner.recommend_cleanup(&branch_name).await;

                            match (summary_res, cleanup_res) {
                                (Ok(s), Ok(c)) => {
                                    if let Some(ref d) = db {
                                        let _ = d.save_analysis(&branch_name, &hash, &s, &c);
                                    }
                                    let _ = ai_update_tx
                                        .send(AIUpdate {
                                            analysis: format!("Summary:\n{}\n\nRecommendation:\n{}", s, c),
                                        })
                                        .await;
                                }
                                _ => {
                                    let _ = ai_update_tx
                                        .send(AIUpdate {
                                            analysis: "AI Analysis failed.".to_string(),
                                        })
                                        .await;
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

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app, &path).await;

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        event::PopKeyboardEnhancementFlags
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("{:?}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    path: &str,
) -> io::Result<()> {
    loop {
        app.update_from_channel(path);

        if app.needs_clear {
            terminal.clear()?;
            app.needs_clear = false;
        }

        terminal.draw(|f| ui::draw(f, app, path))?;

        if event::poll(std::time::Duration::from_millis(50))? && handle_event(app, path)? {
            return Ok(());
        }
    }
}
