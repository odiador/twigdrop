pub mod components;
pub mod screens;

use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Paragraph, Clear},
    style::{Style, Color, Modifier},
    Frame,
};

use crate::app::{App, AppMode};
use crate::ui::screens::{render_main_list, render_help_content, render_manage, render_filter, render_message};

pub const ASCII_LOGO: &str = r#"
████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
   ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
   ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
   ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

pub fn draw(f: &mut Frame, app: &mut App) {
    f.render_widget(Clear, f.area());
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3), Constraint::Length(1)].as_ref())
        .split(f.area());

    let logo = Paragraph::new(ASCII_LOGO.trim_matches('\n'))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(logo, chunks[0]);

    match &app.mode {
        AppMode::Help => {
            render_help_content(f, chunks[1]);
            let footer_text = vec![
                ratatui::text::Line::from(vec![
                    ratatui::text::Span::raw("Made by: "),
                    ratatui::text::Span::styled("odiador", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                    ratatui::text::Span::raw(" ❤️ for the community"),
                ]),
            ];
            let footer_p = Paragraph::new(footer_text).alignment(ratatui::layout::Alignment::Center);
            f.render_widget(footer_p, chunks[2]);
        }
        _ => {
            render_main_list(f, chunks[1], app);
            let shortcuts = Paragraph::new(" ↑/k: move │ f: filter │ m/Enter: manage │ h: help │ q: quit ")
                .style(Style::default().fg(Color::DarkGray));
            f.render_widget(shortcuts, chunks[2]);

            match &app.mode {
                AppMode::Manage => render_manage(f, app),
                AppMode::Filter => render_filter(f, app),
                AppMode::Message(msg) => render_message(f, msg),
                _ => {}
            }
        }
    }
}
