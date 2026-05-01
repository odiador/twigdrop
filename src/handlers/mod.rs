pub mod keyboard;
pub mod mouse;

use ratatui::crossterm::event::{self, Event};
use crate::app::App;
use crate::handlers::keyboard::handle_keyboard;
use crate::handlers::mouse::handle_mouse;

pub fn handle_event(app: &mut App, path: &str) -> Result<bool, std::io::Error> {
    if let Event::Key(key) = event::read()? {
        return Ok(handle_keyboard(app, key, path));
    } else if let Event::Mouse(mouse) = event::read()? {
        handle_mouse(app, mouse);
    }
    Ok(false)
}
