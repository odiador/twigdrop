mod actions;
mod ai;
mod app;
mod db;
mod git;
mod handlers;
mod models;
mod ui;
mod utils;
mod runtime;

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

use app::{AIUpdate, App, ConflictResolutionUpdate};
use handlers::handle_event;
use models::ConflictBlock;
use runtime::Runtime;

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
    let (trigger_tx, trigger_rx) = mpsc::channel::<()>(1);
    
    let (ai_update_tx, ai_rx) = mpsc::channel::<AIUpdate>(10);
    let (ai_trigger_tx, ai_trigger_rx) = mpsc::channel::<(String, String)>(10);
    
    let (conflict_resolution_tx, conflict_resolution_rx) = mpsc::channel::<ConflictResolutionUpdate>(10);
    let (conflict_trigger_tx, conflict_trigger_rx) = mpsc::channel::<(String, ConflictBlock)>(10);

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

    let runtime = Runtime::new(&path);
    
    // Spawn specialized background workers
    runtime.spawn_file_status_poller(file_status_tx, app.shared_primary_mode.clone());
    runtime.spawn_merge_analyzer(trigger_rx, tx);
    runtime.spawn_ai_worker(ai_trigger_rx, ai_update_tx, conflict_trigger_rx, conflict_resolution_tx);

    // Initial trigger for merge analysis
    let _ = trigger_tx.try_send(());

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
