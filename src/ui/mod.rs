pub mod components;
pub mod screens;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::{App, AppMode};
use crate::ui::screens::{
    render_directory_searcher, render_filter, render_help_content, render_main_list, render_manage,
    render_message, render_stash_detail,
};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)].as_ref())
        .split(f.area());

    match &app.mode {
        AppMode::DirectorySearcher => {
            render_directory_searcher(f, chunks[0], app);
        }
        AppMode::StashDetail => {
            render_stash_detail(f, chunks[0], app);
        }
        AppMode::Help => {
            render_help_content(f, chunks[0]);
        }
        _ => {
            render_main_list(f, chunks[0], app);

            match &app.mode {
                AppMode::Manage => render_manage(f, app),
                AppMode::Filter => render_filter(f, app),
                AppMode::Message(msg) => render_message(f, msg),
                _ => {}
            }
        }
    }

    // Footer shortcuts
    let footer_text = if app.alt_pressed {
        match &app.mode {
            AppMode::DirectorySearcher => {
                " q: back │ Enter: open/close │ v: IDE (Path) │ T: Inline TTY │ a: Alt IDE (Path) "
            }
            AppMode::Normal => {
                " ↑/↓: move │ Ctrl+b: files │ Ctrl+o: IDE (Path) │ S: stash │ p: prune │ f: filter │ m: manage │ h: help "
            }
            _ => " q: back ",
        }
    } else {
        match &app.mode {
            AppMode::DirectorySearcher => {
                " q: back │ Enter: open/close │ v: IDE (Root) │ t: External TTY │ a: Alt IDE (Root) "
            }
            AppMode::Normal => {
                " ↑/↓: move │ Ctrl+b: files │ Ctrl+o: IDE (Root) │ S: stash │ p: prune │ f: filter │ m: manage │ h: help "
            }
            _ => " q: back ",
        }
    };

    let footer = ratatui::widgets::Paragraph::new(footer_text)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
    f.render_widget(footer, chunks[1]);
}
