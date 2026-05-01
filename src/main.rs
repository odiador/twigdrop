mod actions;
mod app;
mod git;
mod models;
mod ui;

use actions::{delete_branch, checkout_branch};
use app::{App, AppMode};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};use ratatui::crossterm::event::{MouseEventKind, MouseButton};use std::env;
use std::error::Error;
use std::io;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let path = args.get(1).cloned().unwrap_or_else(|| ".".to_string());

    let branches = git::build_branches(&path);
    let app = App::new(branches);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_app(&mut terminal, app, &path);

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
    mut app: App,
    path: &str,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if let AppMode::Message(_) = app.mode {
                // Any key closes the message
                app.mode = AppMode::Normal;
                continue;
            }

            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => {
                    if app.mode != AppMode::Normal {
                        app.mode = AppMode::Normal;
                    } else {
                        return Ok(());
                    }
                }
                KeyCode::Char('h') => app.toggle_help(),
                KeyCode::Char('m') | KeyCode::Tab => {
                    if app.mode == AppMode::Normal && !app.branches.is_empty() {
                        app.mode = AppMode::Manage;
                        app.manage_selected = 0;
                    } else if app.mode == AppMode::Diff {
                        app.mode = AppMode::Normal;
                    }
                }
                KeyCode::Enter => {
                    if let AppMode::Message(_) = app.mode {
                        app.mode = AppMode::Normal;
                    } else if app.mode == AppMode::Manage {
                        match app.manage_selected {
                            0 => { // Checkout
                                let b = &app.branches[app.selected];
                                let msg = checkout_branch(path, &b.name);
                                app.branches = git::build_branches(path);
                                app.mode = AppMode::Message(msg);
                            }
                            1 => { // View Diff
                                app.mode = AppMode::Diff;
                                app.branch_info = git::get_branch_info(path, &app.branches[app.selected].name);
                                app.info_scroll = 0;
                            }
                            2 => { // Explain Icons
                                app.mode = AppMode::Help;
                            }
                            _ => { // Cancel
                                app.mode = AppMode::Normal;
                            }
                        }
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if app.mode == AppMode::Diff {
                        app.info_scroll += 1;
                    } else if app.mode == AppMode::Manage {
                        if app.manage_selected < 3 { app.manage_selected += 1; }
                    } else if app.mode == AppMode::Normal { 
                        app.next() 
                    }
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if app.mode == AppMode::Diff {
                        app.info_scroll = app.info_scroll.saturating_sub(1);
                    } else if app.mode == AppMode::Manage {
                        if app.manage_selected > 0 { app.manage_selected -= 1; }
                    } else if app.mode == AppMode::Normal { 
                        app.previous() 
                    }
                }
                KeyCode::Char(' ') => {
                    if app.mode == AppMode::Normal { app.toggle() }
                }
                KeyCode::Char('d') => {
                    if app.mode == AppMode::Normal {
                        let mut output_msg = String::new();
                        for (i, b) in app.branches.iter().enumerate() {
                            if app.marked[i] {
                                let msg = delete_branch(path, &b.name);
                                output_msg.push_str(&format!("{}\\n", msg));
                            }
                        }
                        
                        // Refresh data after delete
                        app.branches = git::build_branches(path);
                        let len = app.branches.len();
                        app.marked = vec![false; len];
                        app.selected = app.selected.min(len.saturating_sub(1));
                        
                        if !output_msg.is_empty() {
                            app.mode = AppMode::Message(output_msg);
                        }
                    }
                }
                _ => {}
            }
        } else if let Event::Mouse(mouse_event) = event::read()? {
            if app.mode == AppMode::Normal {
                if let MouseEventKind::Down(MouseButton::Left) = mouse_event.kind {
                    let row = mouse_event.row;
                    if row >= 11 {
                        let mut rel_row = (row - 11) as usize;
                        if app.list_start_index > 0 && rel_row > 0 {
                            rel_row -= 1; // Adjust for "..."
                        }
                        let idx = app.list_start_index + rel_row;
                        if idx < app.branches.len() {
                            app.selected = idx;
                        }
                    }
                }
            }
        }
    }
}
