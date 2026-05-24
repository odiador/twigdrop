mod actions;
mod ai;
mod app;
mod db;
mod events;
mod git;
mod handlers;
mod models;
mod state;
mod tasks;
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
use models::ConflictBlock;
use runtime::Runtime;

use events::Event;

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
    let (ai_trigger_tx, ai_trigger_rx) = mpsc::channel::<(String, String, String)>(10);
    
    let (conflict_resolution_tx, conflict_resolution_rx) = mpsc::channel::<ConflictResolutionUpdate>(10);
    let (conflict_trigger_tx, conflict_trigger_rx) = mpsc::channel::<(String, ConflictBlock)>(10);

    let (file_status_tx, file_status_rx) = mpsc::channel::<app::FileStatusUpdate>(10);
    let (fetched_models_tx, fetched_models_rx) = mpsc::channel::<Vec<String>>(1);
    
    let (event_tx, mut event_rx) = mpsc::channel::<Event>(100);

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
        fetched_models_rx,
    );
    app.setup_ai(&path);
    app.event_tx = Some(event_tx.clone());

    let runtime = Runtime::new(&path);
    
    // Spawn specialized background workers
    runtime.spawn_file_status_poller(file_status_tx, app.shared_primary_mode.clone());
    runtime.spawn_merge_analyzer(trigger_rx, tx);
    runtime.spawn_ai_worker(ai_trigger_rx, ai_update_tx, conflict_trigger_rx, conflict_resolution_tx, fetched_models_tx);

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

    // Spawn event listener task
    let event_tx_clone = event_tx.clone();
    tokio::task::spawn_blocking(move || {
        loop {
            if event::poll(std::time::Duration::from_millis(50)).unwrap_or(false) {
                if let Ok(crossterm_event) = event::read() {
                    match crossterm_event {
                        event::Event::Key(k) => { let _ = event_tx_clone.blocking_send(Event::Key(k)); }
                        event::Event::Mouse(m) => { let _ = event_tx_clone.blocking_send(Event::Mouse(m)); }
                        event::Event::Resize(w, h) => { let _ = event_tx_clone.blocking_send(Event::Resize(w, h)); }
                        _ => {}
                    }
                }
            }
        }
    });

    let res = run_app(&mut terminal, &mut app, &path, &mut event_rx).await;

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
        std::process::exit(1);
    }

    std::process::exit(0);
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    path: &str,
    event_rx: &mut mpsc::Receiver<Event>,
) -> io::Result<()> {
    loop {
        app.update_from_channel(path);

        if app.needs_clear {
            terminal.clear()?;
            app.needs_clear = false;
        }

        terminal.draw(|f| ui::draw(f, app, path))?;

        while let Ok(event) = event_rx.try_recv() {
            // First we give it to keyboard/mouse handlers
            match &event {
                Event::Key(key) => {
                    if handlers::keyboard::handle_keyboard(app, *key, path) {
                        return Ok(());
                    }
                }
                Event::Mouse(mouse) => {
                    handlers::mouse::handle_mouse(app, *mouse, path);
                }
                _ => {}
            }
            // Then we update the app state
            app.update(event);
        }
        
        tokio::time::sleep(std::time::Duration::from_millis(16)).await;
    }
}
