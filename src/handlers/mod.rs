pub mod keyboard;
pub mod mouse;

use ratatui::crossterm::event::{self, Event};
use crate::app::App;
use crate::handlers::keyboard::handle_keyboard;
use crate::handlers::mouse::handle_mouse;

pub fn handle_event(app: &mut App, path: &str) -> Result<bool, std::io::Error> {
    if event::poll(std::time::Duration::from_millis(10))? {
        match event::read()? {
            Event::Key(key) => {
                return Ok(handle_keyboard(app, key, path));
            }
            Event::Mouse(mouse) => {
                handle_mouse(app, mouse);
            }
            _ => {}
        }
    }
    Ok(false)
}
