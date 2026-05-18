mod actions;
mod ai;
mod app;
mod db;
mod git;
mod handlers;
mod models;
mod ui;
mod utils;

use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{
            self, DisableMouseCapture, EnableMouseCapture, KeyboardEnhancementFlags,
            PushKeyboardEnhancementFlags,
        },
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::{env, error::Error, io};
use tokio::sync::mpsc;

use app::{AIUpdate, App, MergeUpdate};
use handlers::handle_event;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    let mut app = App::new(
        branches,
        current_branch,
        rx,
        trigger_tx.clone(),
        ai_rx,
        ai_trigger_tx.clone(),
    );
    app.setup_ai(&path);

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

    // Background AI analyzer
    let ai_path = path.clone();
    tokio::spawn(async move {
        let mut bg_app = App::new(
            vec![],
            "".to_string(),
            mpsc::channel(1).1,
            mpsc::channel(1).0,
            mpsc::channel(1).1,
            mpsc::channel(1).0,
        );
        bg_app.setup_ai(&ai_path);

        while let Some((p, b)) = ai_trigger_rx.recv().await {
            bg_app.trigger_ai_analysis(&p, &b).await;
            if let Some(analysis) = bg_app.ai_state.ai_analysis.take() {
                let _ = ai_update_tx.send(AIUpdate { analysis }).await;
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
        println!("{:?}", err);
    }

    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    path: &str,
) -> io::Result<()> {
    loop {
        app.update_from_channel();

        if app.needs_clear {
            terminal.clear()?;
            app.needs_clear = false;
        }

        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(50))? && handle_event(app, path)? {
            return Ok(());
        }
    }
}
