use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};

use crate::app::App;
use crate::state::ui::{
    AppMode, CommandPaletteState, DatePickerField, DatePickerState, PrimaryMode,
};

pub const ASCII_LOGO: &str = r#"
████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
   ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
   ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
   ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

pub fn render_help(f: &mut Frame, app: &App) {
    let area = crate::ui::components::centered_rect(80, 80, f.area());
    render_help_content(f, area, app);
}

pub fn render_help_content(f: &mut Frame, area: Rect, app: &App) {
    f.render_widget(Clear, area);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(8), // Logo
                Constraint::Min(10),   // Content
                Constraint::Length(1), // Footer
            ]
            .as_ref(),
        )
        .split(area);

    let logo = Paragraph::new(ASCII_LOGO.trim_matches('\n'))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(logo, chunks[0]);

    let block = Block::default()
        .title(Line::from(" Help & Legend ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    let mut help_text = vec![
        "Twigdrop helps you clean up your local branches safely.",
        "",
    ];

    if app.ui.primary_mode == PrimaryMode::Branches {
        help_text.extend(vec![
            "Status Icons (Branches):",
            "  ▲ (Red)     : Has Unique Commits (DANGER: Not in remote!)",
            "  ⨯ (Gray)    : Gone (Upstream branch was deleted)",
            "  ↑ (Yellow)  : Ahead of upstream (Local has new commits)",
            "  ↓ (Cyan)    : Behind upstream (Remote has new commits)",
            "  ✓ (Green)   : Merged (Safe to delete)",
            "  L (Lavender) : Local Only (No tracking branch)",
            "  S (Green)   : Stashed changes exist for this branch",
            "",
            "Shortcuts (Branches):",
            "  ↑/k, ↓/j       : Navigate list",
            "  /              : Fuzzy search branches",
            "  Space          : Toggle branch selection (for bulk delete)",
            "  Shift+D        : Bulk delete selected branches",
            "  p              : Prune 'Gone' branches (Safe only)",
            "  i              : AI Intelligence Analysis for branch",
            "  c              : Create new branch",
            "  f              : Open Filters",
            "  m / Enter      : Manage selected branch (Checkout, Diff, Delete)",
        ]);
    } else {
        help_text.extend(vec![
            "Status Colors (Files):",
            "  Yellow      : Modified",
            "  Green       : Added / New",
            "  Blue        : Staged (in index)",
            "  Pink        : Untracked",
            "  Gray        : Ignored (.gitignore / .twigignore)",
            "",
            "Shortcuts (Files):",
            "  ↑/k, ↓/j    : Navigate tree",
            "  → / Enter   : Open folder / Move into children",
            "  ←           : Close folder / Move to parent",
            "  Enter       : Preview file content",
            "  e           : Open folder in Explorer (Alt+e for selected path)",
            "  s           : Stage / Unstage file",
            "  v           : Open in IDE (Root by default, Path with Alt)",
            "  t           : Internal TTY (Alt+t for External)",
            "  a           : Alt IDE (Root by default, Path with Alt)",
            "  [ / ]       : Expand / Contract sidebar width",
            "  Tab         : Switch focus between sidebar and preview",
        ]);
    }

    help_text.extend(vec![
        "",
        "Global Shortcuts:",
        "  d              : Switch between Branches and Files mode",
        "  Shift+Tab      : Open App Switcher",
        "  Esc            : Open Main Menu (Settings, Help, Quit)",
        "  !              : Open Shell Command Prompt",
        "  :              : Open Git Quick Actions Palette",
        "  ? / h          : Help & Legend",
        "  Shift+S        : Open Stash Manager",
        "  Shift+C        : Open Unpushed Commits Manager",
        "  Shift+F        : AI Auto-Fix (in Diff mode with conflicts)",
        "  Shift+R        : Interactive Rebase",
        "  Ctrl+o         : Open IDE (Root by default, Path with Alt)",
        "  q              : Quit",
    ]);

    let p = Paragraph::new(help_text.join("\n"))
        .block(block)
        .alignment(Alignment::Left);

    let help_inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(chunks[1])[1];

    f.render_widget(p, help_inner);

    let footer_text = vec![Line::from(vec![
        Span::raw("Made by: "),
        Span::styled(
            "odiador",
            Style::default()
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" ❤️ for the community"),
    ])];
    let footer_p = Paragraph::new(footer_text).alignment(Alignment::Center);
    f.render_widget(footer_p, chunks[2]);
}

pub fn render_message(f: &mut Frame, msg: &str) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(area)[1];

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(Line::from(" Git Response ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow))
        .style(Style::default().bg(Color::Rgb(20, 20, 20)));

    let p = Paragraph::new(msg)
        .block(block)
        .alignment(Alignment::Left)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(p, inner);
}

pub fn render_command_palette(f: &mut Frame, state: &CommandPaletteState) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area)[1];

    f.render_widget(Clear, inner);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)])
        .split(inner);

    let input_block = Block::default()
        .title(" Command Palette (Ctrl+P) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let query_text = format!("> {}", state.query);
    f.render_widget(Paragraph::new(query_text).block(input_block), chunks[0]);

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::DarkGray));

    let mut items = vec![];
    for (name, _) in &state.actions {
        if !state.query.is_empty() && !name.to_lowercase().contains(&state.query.to_lowercase()) {
            continue;
        }

        let mut style = Style::default().fg(Color::Gray);
        if items.len() == state.selected {
            style = style
                .bg(Color::Rgb(80, 80, 100))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);
        }
        items.push(ListItem::new(name.clone()).style(style));
    }

    if items.is_empty() {
        items.push(ListItem::new("No commands found.").style(Style::default().fg(Color::DarkGray)));
    }

    f.render_widget(List::new(items).block(list_block), chunks[1]);
}

pub fn render_shell(f: &mut Frame, input: &str) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(40),
                Constraint::Length(3),
                Constraint::Percentage(40),
            ]
            .as_ref(),
        )
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(10),
                Constraint::Percentage(80),
                Constraint::Percentage(10),
            ]
            .as_ref(),
        )
        .split(area)[1];

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(" Execute Shell Command ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));

    let p = Paragraph::new(format!("$ {}", input))
        .block(block)
        .alignment(Alignment::Left);
    f.render_widget(p, inner);
}

pub fn render_quick_actions(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ]
            .as_ref(),
        )
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(30),
                Constraint::Percentage(40),
                Constraint::Percentage(30),
            ]
            .as_ref(),
        )
        .split(area)[1];

    f.render_widget(Clear, inner);

    let mut items = vec![];
    for (i, action) in app.ui.quick_actions_state.actions.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.ui.quick_actions_state.selected {
            style = style
                .bg(Color::Rgb(80, 80, 100))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);
        }
        items.push(ListItem::new(format!(" {} ", action)).style(style));
    }

    let list = List::new(items).block(
        Block::default()
            .title(" Git Quick Actions ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );
    f.render_widget(list, inner);
}

pub fn render_settings(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(f.area())[1];
    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(area)[1];
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
            style = style
                .bg(Color::Rgb(80, 80, 100))
                .add_modifier(Modifier::BOLD);
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
                style = style
                    .bg(Color::Rgb(80, 80, 100))
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD);
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

pub fn render_search(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
        .split(f.area())[0];
    f.render_widget(Clear, area);
    f.render_widget(
        Paragraph::new(format!("> {}", app.ui.search_query)).block(
            Block::default()
                .title(" Search Branches ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow)),
        ),
        area,
    );
}

fn render_transparent_ascii(
    buf: &mut ratatui::buffer::Buffer,
    ascii: &str,
    area: Rect,
    color: Color,
) {
    let lines: Vec<&str> = ascii.trim_matches('\n').lines().collect();
    if lines.is_empty() {
        return;
    }

    let max_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0) as u16;
    let height = lines.len() as u16;

    let start_x = area.x + area.width.saturating_sub(max_width) / 2;
    let start_y = area.y + area.height.saturating_sub(height) / 2;

    for (row_idx, line) in lines.iter().enumerate() {
        let y = start_y + row_idx as u16;
        if y >= area.bottom() {
            break;
        }

        for (x, ch) in (start_x..).zip(line.chars()) {
            if x >= area.right() {
                break;
            }
            if ch != ' ' {
                buf[(x, y)].set_char(ch).set_fg(color);
            }
        }
    }
}

pub fn render_main_menu(f: &mut Frame, app: &App) {
    let area = f.area();
    let buf = f.buffer_mut();

    let ascii_logo = r#"
 ████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
 ╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
    ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
    ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
    ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
    ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

    let opt_options = r#"
  ___  ____ _____ ___ ___  _  _ ___ 
 / _ \|  _ \_   _|_ _/ _ \| \| / __|
| (_) | |_) || |  | | (_) | .` \__ \
 \___/| .__/ |_| |___\___/|_|\_|___/
      |_|                           
"#;

    let opt_help = r#"
 _  _ ___ _    ___ 
| || | __| |  | _ \
| __ | _|| |__|  _/
|_||_|___|____|_|  
"#;

    let opt_quit = r#"
  ___  _   _ ___ _____ 
 / _ \| | | |_ _|_   _|
| (_) | |_| || |  | |  
 \__\_\\___/|___| |_|  
"#;

    let total_height = 8 + 2 + 6 + 5 + 5;
    let v_margin = area.height.saturating_sub(total_height) / 2;

    let v_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(v_margin),
            Constraint::Length(8),
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    render_transparent_ascii(buf, ascii_logo, v_chunks[1], Color::Cyan);

    let options_ascii = [opt_options, opt_help, opt_quit];
    for (i, opt) in options_ascii.iter().enumerate() {
        let is_selected = i == app.ui.main_menu_state.selected;
        let color = if is_selected {
            Color::White
        } else {
            Color::Rgb(60, 60, 80)
        };

        render_transparent_ascii(buf, opt, v_chunks[3 + i], color);
    }
}

pub fn render_switcher(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(50),
            Constraint::Percentage(25),
        ])
        .split(f.area());

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(area[1])[1];

    f.render_widget(Clear, inner);

    let modes = [
        AppMode::FilesView,
        AppMode::BranchesView,
        AppMode::CommitsView,
        AppMode::Help,
    ];

    let mut items = Vec::new();
    for (i, mode) in modes.iter().enumerate() {
        let text = format_mode(mode);
        let mut style = Style::default().fg(Color::Gray);

        if i == app.ui.switcher_index {
            style = style
                .bg(Color::Rgb(80, 80, 100))
                .fg(Color::White)
                .add_modifier(Modifier::BOLD);
        }
        items.push(ListItem::new(text).style(style));
    }

    if items.is_empty() {
        items.push(ListItem::new("  No history  ").style(Style::default().fg(Color::Gray)));
    }

    let list = List::new(items).block(
        Block::default()
            .title(ratatui::text::Line::from(" App Switcher ").alignment(Alignment::Center))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan)),
    );

    f.render_widget(list, inner);
}

pub fn format_mode(mode: &AppMode) -> String {
    match mode {
        AppMode::BranchesView => "Branches".to_string(),
        AppMode::FilesView => "Files".to_string(),
        AppMode::CommitsView => "Commit Tree".to_string(),
        AppMode::Normal => "Main Views".to_string(),
        AppMode::Help => "Help".to_string(),
        AppMode::Manage => "Manage Branch".to_string(),
        AppMode::Filter => "Filters".to_string(),
        AppMode::StashDetail => "Stash Details".to_string(),
        AppMode::Settings => "Settings".to_string(),
        AppMode::Search => "Search".to_string(),
        AppMode::Diff => "Diff".to_string(),
        AppMode::CodePreview(state) => format!("Preview: {}", state.file_path),
        AppMode::ConfirmDelete(_) => "Confirm Delete".to_string(),
        AppMode::CreateBranch(_) => "Create Branch".to_string(),
        AppMode::CommitAction(_) => "Commit Actions".to_string(),
        AppMode::InteractiveRebase => "Interactive Rebase".to_string(),
        AppMode::Shell(_) => "Shell".to_string(),
        AppMode::QuickActions => "Quick Actions".to_string(),
        AppMode::MainMenu => "Main Menu".to_string(),
        AppMode::Message(_) => "Message".to_string(),
        AppMode::Switcher => "App Switcher".to_string(),
        AppMode::DatePicker(_) => "Date Picker".to_string(),
        AppMode::CommitFiles(hash, _) => format!("Files in {}", hash),
        AppMode::CommandPalette(_) => "Command Palette".to_string(),
    }
}

pub fn render_date_picker(f: &mut Frame, _app: &App, state: &DatePickerState) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .split(area)[1];

    f.render_widget(Clear, inner);

    let block = Block::default()
        .title(" Select Date & Time (Enter to Save) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));

    f.render_widget(block, inner);

    let picker_area = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
            Constraint::Percentage(20),
        ])
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
                    style = style.bg(Color::Rgb(80, 80, 100)).fg(Color::Cyan);
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
