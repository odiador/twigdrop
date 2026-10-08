use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::state::ui::{DatePickerField, DatePickerState};

pub fn render_date_picker(f: &mut Frame, _app: &App, state: &DatePickerState) {
    let (mx, my) = crate::ui::layout::DATE_PICKER_MODAL;
    let inner = crate::ui::components::centered_rect(mx, my, f.area());

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(" Select Date & Time (Enter to Save) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(block, inner);

    // Five equal wheels: Year / Month / Day / Hour / Min.
    let picker_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Ratio(1, 5); 5])
        .margin(1)
        .split(inner);

    let fields = [
        ("Year", DatePickerField::Year),
        ("Month", DatePickerField::Month),
        ("Day", DatePickerField::Day),
        ("Hour", DatePickerField::Hour),
        ("Min", DatePickerField::Minute),
    ];

    for (i, (label, field)) in fields.iter().enumerate() {
        let is_focused = state.active_field == *field;
        let col_area = picker_area[i];

        // Column Label
        let label_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(col_area);

        f.render_widget(
            Paragraph::new(Span::styled(
                *label,
                Style::default()
                    .fg(if is_focused {
                        Color::Cyan
                    } else {
                        Color::DarkGray
                    })
                    .add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center),
            label_area[0],
        );

        // Wheel (Previous, Current, Next)
        let wheel_items = match field {
            DatePickerField::Year => vec![
                (state.year - 1).to_string(),
                state.year.to_string(),
                (state.year + 1).to_string(),
            ],
            DatePickerField::Month => vec![
                get_month_name(if state.month == 1 {
                    12
                } else {
                    state.month - 1
                })
                .to_string(),
                get_month_name(state.month).to_string(),
                get_month_name(if state.month == 12 {
                    1
                } else {
                    state.month + 1
                })
                .to_string(),
            ],
            DatePickerField::Day => {
                let max_days = crate::utils::days_in_month(state.month, state.year);
                vec![
                    (if state.day == 1 {
                        max_days
                    } else {
                        state.day - 1
                    })
                    .to_string(),
                    state.day.to_string(),
                    (if state.day == max_days {
                        1
                    } else {
                        state.day + 1
                    })
                    .to_string(),
                ]
            }
            DatePickerField::Hour => vec![
                format!("{:02}", if state.hour == 0 { 23 } else { state.hour - 1 }),
                format!("{:02}", state.hour),
                format!("{:02}", (state.hour + 1) % 24),
            ],
            DatePickerField::Minute => vec![
                format!(
                    "{:02}",
                    if state.minute == 0 {
                        59
                    } else {
                        state.minute - 1
                    }
                ),
                format!("{:02}", state.minute),
                format!("{:02}", (state.minute + 1) % 60),
            ],
        };

        let mut spans = vec![];
        for (idx, val) in wheel_items.iter().enumerate() {
            let mut style = Style::default().fg(Color::DarkGray);
            let mut text = val.clone();

            if idx == 1 {
                style = style.fg(Color::White).add_modifier(Modifier::BOLD);
                if is_focused {
                    style = crate::ui::theme::Theme::dark_default()
                        .select_cyan
                        .add_modifier(Modifier::BOLD);
                    text = format!(" {} ◄", text);
                } else {
                    text = format!(" {}  ", text);
                }
            } else {
                text = format!(" {}  ", text);
            }
            spans.push(ListItem::new(
                Line::from(Span::styled(text, style)).alignment(Alignment::Center),
            ));
        }

        let list = List::new(spans);
        f.render_widget(list, label_area[1]);
    }
}

fn get_month_name(m: u32) -> &'static str {
    match m {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}
