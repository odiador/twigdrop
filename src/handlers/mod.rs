pub mod keyboard;
pub mod mouse;

use ratatui::crossterm::event::{self, Event};
use crate::app::App;
use crate::handlers::keyboard::handle_keyboard;
use crate::handlers::mouse::handle_mouse;

pub fn handle_event(app: &mut App, path: &str) -> Result<bool, std::io::Error> {
    // Wait for the first event (blocking)
    let first_event = event::read()?;
    let mut quit = process_single_event(app, first_event, path);
    
    // Process any other events already in the queue to avoid lag
    while event::poll(std::time::Duration::from_millis(0))? {
        let next_event = event::read()?;
        if process_single_event(app, next_event, path) {
            quit = true;
        }
    }
    
    Ok(quit)
}

fn process_single_event(app: &mut App, event: Event, path: &str) -> bool {
    match event {
        Event::Key(key) => {
            return handle_keyboard(app, key, path);
        }
        Event::Mouse(mouse) => {
            handle_mouse(app, mouse);
        }
        _ => {}
    }
    false
}
