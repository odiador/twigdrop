pub mod command_palette;
pub mod date_picker;
pub mod help;
pub mod main_menu;
pub mod message;
pub mod search;
pub mod settings;
pub mod shell;
pub mod switcher;

pub use command_palette::render_command_palette;
pub use date_picker::render_date_picker;
pub use help::{ASCII_LOGO, render_help, render_help_content};
pub use main_menu::{render_main_menu, render_transparent_ascii};
pub use message::render_message;
pub use search::render_search;
pub use settings::render_settings;
pub use shell::{render_quick_actions, render_shell};
pub use switcher::{format_mode, render_switcher};
