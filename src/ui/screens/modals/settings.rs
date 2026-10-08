use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::ui::theme::Theme;

pub fn render_settings(f: &mut Frame, app: &App) {
    let theme = Theme::dark_default();
    let (mx, my) = crate::ui::layout::SETTINGS_MODAL;
    let inner = crate::ui::components::centered_rect(mx, my, f.area());
    f.render_widget(Clear, inner);

    let options = [
        format!(
            "[ Editor ] Primary IDE          : {}",
            app.config.ide_command
        ),
        format!(
            "[ Editor ] Alternative IDE      : {}",
            app.config.alternative_ide_command
        ),
        format!(
            "[ AI ]     AI Provider          : {}",
            app.config.ai_provider
        ),
        format!(
            "[ AI ]     AI Model             : {}",
            app.config.current_provider().model
        ),
        format!(
            "[ AI ]     OpenAI API Key       : {}",
            if app.config.current_provider().api_key.is_empty() {
                "None".to_string()
            } else {
                "****".to_string()
            }
        ),
        format!(
            "[ AI ]     Ollama URL           : {}",
            app.config.current_provider().url
        ),
        format!(
            "[ UI ]     Enable Animations    : {}",
            app.config.enable_animations
        ),
        format!(
            "[ UI ]     Default Sidebar Width: {}",
            app.config.default_sidebar_width
        ),
        "           [ Save and Exit ]".to_string(),
    ];
    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.settings_state.selected {
            style = theme.select_hi.add_modifier(Modifier::BOLD);
        }

        let text = if i == app.ui.settings_state.selected {
            if app.ui.settings_state.editing {
                if i == 4 {
                    format!("> {}", "*".repeat(app.ui.settings_state.input.len()))
                } else {
                    format!("> {}", app.ui.settings_state.input)
                }
            } else if app.ui.settings_state.selecting {
                format!("{} (Selecting...)", opt)
            } else {
                opt.clone()
            }
        } else {
            opt.clone()
        };

        if i == app.ui.settings_state.selected {
            style = style.fg(Color::White);
        }

        items.push(ListItem::new(text).style(style));
    }

    f.render_widget(
        List::new(items).block(
            Block::default()
                .title(Line::from(" [ Twigdrop Settings ] ").alignment(Alignment::Center))
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan)),
        ),
        inner,
    );

    if app.ui.settings_state.selecting {
        let menu_width = 40;
        let menu_height = app.ui.settings_state.choices.len().min(10) as u16 + 2;

        let center_x = inner.x + (inner.width / 2);
        let center_y = inner.y + (inner.height / 2);

        let menu_area = Rect::new(
            center_x.saturating_sub(menu_width / 2),
            center_y.saturating_sub(menu_height / 2),
            menu_width,
            menu_height,
        );
        f.render_widget(Clear, menu_area);

        let mut choice_items = vec![];
        for (i, choice) in app.ui.settings_state.choices.iter().enumerate() {
            let mut style = Style::default().fg(Color::Gray);
            if i == app.ui.settings_state.choice_idx {
                style = theme.select_hi.add_modifier(Modifier::BOLD);
            }
            choice_items.push(ListItem::new(format!(" {} ", choice)).style(style));
        }
        let list = List::new(choice_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .title(" Select Option "),
        );
        f.render_widget(list, menu_area);
    }

    let footer_msg = if app.ui.settings_state.selecting {
        "↑/↓: cycle options │ Enter: select │ Esc: cancel"
    } else if app.ui.settings_state.editing {
        "Type your value │ Enter: save │ Esc: cancel"
    } else {
        "↑/↓: navigate │ Enter: edit/select │ Esc: cancel"
    };

    let help_area = Rect::new(inner.x, inner.y + inner.height - 2, inner.width, 1);
    f.render_widget(
        Paragraph::new(footer_msg)
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray)),
        help_area,
    );
}
