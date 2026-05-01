mod app;
mod models;
mod git;
mod actions;
mod ui;
mod handlers;

use std::{env, error::Error, io};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        event::{DisableMouseCapture, EnableMouseCapture},
    },
    Terminal,
};

use app::App;
use handlers::handle_event;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).cloned().unwrap_or_else(|| ".".to_string());

    let branches = git::build_branches(&path);
    let current_branch = git::get_current_branch(&path);
    let mut app = App::new(branches, current_branch);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, &mut app, &path);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
    path: &str,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if handle_event(app, path)? {
            return Ok(());
        }
    }
}
