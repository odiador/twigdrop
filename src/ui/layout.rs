use crate::state::ui::AppMode;
use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DiffLayout {
    pub inner_area: Rect,
    pub files_list_area: Rect,
    pub ai_area: Rect,
    pub preview_area: Rect,
}

pub fn calculate_diff_layout(area: Rect) -> DiffLayout {
    let overlay_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(90),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        )
        .split(area)[1];

    let inner_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(90),
                Constraint::Percentage(5),
            ]
            .as_ref(),
        )
        .split(overlay_area)[1];

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(25), Constraint::Percentage(75)].as_ref())
        .split(inner_area);

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
        .split(main_chunks[0]);

    DiffLayout {
        inner_area,
        files_list_area: left_chunks[0],
        ai_area: left_chunks[1],
        preview_area: main_chunks[1],
    }
}

pub fn calculate_modal_rect(mode: &AppMode, area: Rect) -> Rect {
    match mode {
        AppMode::Help => crate::ui::components::centered_rect(80, 80, area),
        AppMode::Filter => {
            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(20),
                    Constraint::Percentage(60),
                    Constraint::Percentage(20),
                ])
                .split(area)[1];
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(v)[1]
        }
        AppMode::Settings => {
            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(15),
                    Constraint::Percentage(70),
                    Constraint::Percentage(15),
                ])
                .split(area)[1];
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(15),
                    Constraint::Percentage(70),
                    Constraint::Percentage(15),
                ])
                .split(v)[1]
        }
        AppMode::Manage => {
            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(area)[1];
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(v)[1]
        }
        AppMode::ConfirmDelete(_) | AppMode::Message(_) => {
            let v = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(40),
                    Constraint::Percentage(30),
                ])
                .split(area)[1];
            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(15),
                    Constraint::Percentage(70),
                    Constraint::Percentage(15),
                ])
                .split(v)[1]
        }
        _ => crate::ui::components::centered_rect(50, 40, area),
    }
}
