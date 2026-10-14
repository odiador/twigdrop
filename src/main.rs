use anyhow::Result;
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
use std::{env, io};
use tokio::sync::mpsc;

use twigdrop::{app::App, events::Event, git, handlers, models, runtime::Runtime, ui};

#[tokio::main]
async fn main() -> Result<()> {
    let default_panic = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let mut stdout = std::io::stdout();
        let _ = execute!(stdout, LeaveAlternateScreen, DisableMouseCapture);
        default_panic(info);
    }));

    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!(
            "{name} {version}\n\
             {desc}\n\
             \n\
             Usage: {name} [PATH]\n\
             \n\
             Arguments:\n  \
             [PATH]  Repository to manage (default: .)\n\
             \n\
             Options:\n  \
             -h, --help     Show this help\n  \
             -V, --version  Show version",
            name = env!("CARGO_PKG_NAME"),
            version = env!("CARGO_PKG_VERSION"),
            desc = env!("CARGO_PKG_DESCRIPTION"),
        );
        return Ok(());
    }
    if args.iter().any(|a| a == "--version" || a == "-V") {
        println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    let path = args.get(1).cloned().unwrap_or_else(|| ".".to_string());

    if !git::commands::is_inside_git_work_tree(&path) {
        eprintln!(
            "fatal: not a git repository (or any of the parent directories): {}",
            path
        );
        std::process::exit(1);
    }

    let branches = git::build_branches(&path);
    let current_branch = git::get_current_branch(&path);

    let (trigger_tx, trigger_rx) = mpsc::channel::<()>(1);
    let (ai_trigger_tx, ai_trigger_rx) = mpsc::channel::<(String, String, String)>(10);
    let (conflict_trigger_tx, conflict_trigger_rx) =
        mpsc::channel::<(String, models::ConflictBlock)>(10);

    let (event_tx, mut event_rx) = mpsc::channel::<Event>(1000);

    let mut app = App::new(
        &path,
        branches,
        current_branch,
        trigger_tx.clone(),
        ai_trigger_tx.clone(),
        conflict_trigger_tx.clone(),
    );
    app.setup_ai(&path);
    app.event_tx = Some(event_tx.clone());

    let runtime = Runtime::new(&path);

    runtime.spawn_file_status_poller(event_tx.clone(), app.shared_primary_mode.clone());
    runtime.spawn_merge_analyzer(trigger_rx, event_tx.clone());
    runtime.spawn_ai_worker(ai_trigger_rx, conflict_trigger_rx, event_tx.clone());

    let _ = trigger_tx.try_send(());

    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        PushKeyboardEnhancementFlags(
            KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
                | KeyboardEnhancementFlags::REPORT_EVENT_TYPES
        )
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let event_tx_clone = event_tx.clone();
    tokio::task::spawn_blocking(move || {
        loop {
            if event::poll(std::time::Duration::from_millis(10)).unwrap_or(false)
                && let Ok(crossterm_event) = event::read()
            {
                match crossterm_event {
                    event::Event::Key(k) => {
                        let _ = event_tx_clone.blocking_send(Event::Key(k));
                    }
                    event::Event::Mouse(m) => {
                        let _ = event_tx_clone.blocking_send(Event::Mouse(m));
                    }
                    event::Event::Resize(_, _) => {
                        let _ = event_tx_clone.blocking_send(Event::Resize);
                    }
                    _ => {}
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
fn process_event(app: &mut App, event: &Event, path: &str) -> bool {
    match event {
        Event::Key(key) => handlers::keyboard::handle_keyboard(app, *key, path),
        Event::Mouse(mouse) => {
            handlers::mouse::handle_mouse(app, *mouse, path);
            false
        }
        _ => false,
    }
}

/// Completes when the process receives SIGTERM or SIGINT (unix).
/// On other platforms it never completes, so the event loop is unaffected.
async fn wait_for_signal(
    #[cfg(unix)] sigterm: &mut tokio::signal::unix::Signal,
    #[cfg(unix)] sigint: &mut tokio::signal::unix::Signal,
) {
    #[cfg(unix)]
    {
        tokio::select! {
            _ = sigterm.recv() => {},
            _ = sigint.recv() => {},
        }
    }
    #[cfg(not(unix))]
    std::future::pending::<()>().await
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    path: &str,
    event_rx: &mut mpsc::Receiver<Event>,
) -> io::Result<()> {
    // Graceful shutdown: SIGTERM/SIGINT return from the loop so `main`
    // runs the terminal-restore path (raw mode, alternate screen, mouse).
    // Without this, `pkill twigdrop` would leave the shell unusable.
    #[cfg(unix)]
    let mut sigterm =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("sigterm handler");
    #[cfg(unix)]
    let mut sigint =
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
            .expect("sigint handler");

    // Initial draw
    terminal.draw(|f| ui::draw(f, app, path))?;

    loop {
        let has_animation = app.config.enable_animations && app.ui.snap_animation.is_some();
        let mut should_draw;

        if has_animation {
            // Active animation: drain pending events or tick with ~60 FPS timeout
            tokio::select! {
                maybe_event = event_rx.recv() => {
                    if let Some(event) = maybe_event {
                        if process_event(app, &event, path) {
                            return Ok(());
                        }
                        app.update(event);
                        should_draw = true;
                    } else {
                        return Ok(());
                    }
                }
                _ = tokio::time::sleep(std::time::Duration::from_millis(16)) => {
                    should_draw = true;
                }
                _ = wait_for_signal(&mut sigterm, &mut sigint) => {
                    return Ok(());
                }
            }
        } else {
            // Idle state: sleep until an event arrives — unless transient
            // UI (toast TTL, busy spinner) needs ticks to advance.
            let needs_tick =
                app.is_busy() || app.ui.toast.as_ref().is_some_and(|t| t.is_live());
            if needs_tick {
                tokio::select! {
                    maybe_event = event_rx.recv() => {
                        if let Some(event) = maybe_event {
                            if process_event(app, &event, path) {
                                return Ok(());
                            }
                            app.update(event);
                            should_draw = true;
                        } else {
                            return Ok(());
                        }
                    }
                    _ = tokio::time::sleep(std::time::Duration::from_millis(250)) => {
                        should_draw = true;
                    }
                    _ = wait_for_signal(&mut sigterm, &mut sigint) => {
                        return Ok(());
                    }
                }
            } else {
                tokio::select! {
                    maybe_event = event_rx.recv() => {
                        if let Some(event) = maybe_event {
                            if process_event(app, &event, path) {
                                return Ok(());
                            }
                            app.update(event);
                            should_draw = true;
                        } else {
                            return Ok(());
                        }
                    }
                    _ = wait_for_signal(&mut sigterm, &mut sigint) => {
                        return Ok(());
                    }
                }
            }
        }

        // Drain any burst events that queued up during processing
        while let Ok(event) = event_rx.try_recv() {
            if process_event(app, &event, path) {
                return Ok(());
            }
            app.update(event);
            should_draw = true;
        }

        if should_draw {
            if app.ui.needs_clear {
                terminal.clear()?;
                app.ui.needs_clear = false;
            }
            terminal.draw(|f| ui::draw(f, app, path))?;
        }
    }
}
