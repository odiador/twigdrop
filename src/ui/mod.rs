pub mod components;
pub mod screens;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
};

use crate::app::{App, AppMode, PrimaryMode};

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)].as_ref())
        .split(f.area());

    match app.primary_mode {
        PrimaryMode::Branches => {
            screens::render_main_list(f, chunks[0], app);
        }
        PrimaryMode::Files => {
            screens::render_directory_searcher(f, chunks[0], app);
        }
    }

    // Modals
    match &app.mode {
        AppMode::Manage => screens::render_manage(f, app),
        AppMode::Filter => screens::render_filter(f, app),
        AppMode::Message(msg) => screens::render_message(f, msg),
        AppMode::Help => screens::render_help_content(f, chunks[0], app),
        AppMode::StashDetail => screens::render_stash_detail(f, chunks[0], app),
        AppMode::Settings => screens::render_settings(f, app),
        _ => {}
    }

    // Footer shortcuts
    let footer_text = if app.shift_pressed {
        match app.primary_mode {
            PrimaryMode::Branches => {
                " S: Stash Mgr │ D: Delete ALL Selected │ h: Legend │ q: quit "
            }
            PrimaryMode::Files => " S: Stash Mgr │ h: Legend │ q: back ",
        }
    } else if app.alt_pressed {
        match app.primary_mode {
            PrimaryMode::Branches => {
                " ↑/↓: move │ d: switch mode │ Alt+t: External TTY │ f: filter "
            }
            PrimaryMode::Files => {
                " ↑/↓: move │ d: switch mode │ v: IDE (Path) │ a: Alt IDE (Path) │ Alt+t: External TTY "
            }
        }
    } else {
        match app.primary_mode {
            PrimaryMode::Branches => {
                " ↑/↓: move │ d: files │ p: prune │ f: filter │ m: manage │ h: help │ q: quit "
            }
            PrimaryMode::Files => {
                " ↑/↓: move │ d: branches │ Enter: open/close │ v: IDE │ t: Inline TTY │ h: help "
            }
        }
    };

    let footer = ratatui::widgets::Paragraph::new(footer_text)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
    f.render_widget(footer, chunks[1]);
}
