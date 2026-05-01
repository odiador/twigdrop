pub mod components;
pub mod screens;

use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::Paragraph,
    style::{Style, Color},
    Frame,
};

use crate::app::{App, AppMode};
use crate::ui::screens::{render_main_list, render_help, render_manage, render_filter, render_message};

pub const ASCII_LOGO: &str = r#"
████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
   ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
   ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
   ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3), Constraint::Length(1)].as_ref())
        .split(f.area());

    let title = Paragraph::new(ASCII_LOGO).style(Style::default().fg(Color::Cyan)).alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    render_main_list(f, chunks[1], app);

    let shortcuts = Paragraph::new(" ↑/k: move │ f: filter │ m/Enter: manage │ h: help │ q: quit ")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(shortcuts, chunks[2]);

    match &app.mode {
        AppMode::Help => render_help(f),
        AppMode::Manage => render_manage(f, app),
        AppMode::Filter => render_filter(f, app),
        AppMode::Message(msg) => render_message(f, msg),
        _ => {}
    }
}
