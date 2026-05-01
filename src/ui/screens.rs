use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    style::{Style, Color, Modifier},
    text::Line,
    Frame,
};

use crate::app::{App, AppMode};
use crate::ui::components::get_status_icons;
use crate::ui::ASCII_LOGO;

pub fn render_main_list(f: &mut Frame, area: Rect, app: &mut App) {
    let branches_len = app.filtered_indices.len();
    let inner_height = area.height.saturating_sub(2) as usize;

    if inner_height == 0 { return; }

    let mut start = 0;
    if branches_len > inner_height {
        let half_height = inner_height / 2;
        if app.selected > half_height {
            start = app.selected - half_height;
        }
        let mut end = start + inner_height;
        if end > branches_len {
            end = branches_len;
            start = end.saturating_sub(inner_height);
        }
    }
    app.list_start_index = start;
    let filtered_branches = app.get_filtered_branches();

    let show_top_dots = start > 0;
    let show_bottom_dots = (start + inner_height) < branches_len;
    
    let mut items: Vec<ListItem> = vec![];
    if show_top_dots {
        items.push(ListItem::new("...").style(Style::default().fg(Color::DarkGray)));
    }

    let branch_items_to_show = inner_height 
        - (if show_top_dots { 1 } else { 0 }) 
        - (if show_bottom_dots { 1 } else { 0 });

    for i in 0..branch_items_to_show {
        let branch_idx = start + i;
        if branch_idx >= branches_len { break; }
        
        let b = filtered_branches[branch_idx];
        let selected = branch_idx == app.selected;
        let is_current = b.name == app.current_branch;
        
        let (icons, color) = get_status_icons(&b.status);

        let current_tag = if is_current { " (current)" } else { "" };
        let line = format!("  {:<5}  {}{}", icons, b.name, current_tag);
        
        let mut item_style = Style::default().fg(color);
        if is_current {
            item_style = item_style.add_modifier(Modifier::BOLD).fg(Color::White);
        }
        
        let mut item = ListItem::new(line).style(item_style);

        if selected {
            item = item.style(item_style.bg(Color::Rgb(40, 40, 40)));
        }
        items.push(item);
    }

    if show_bottom_dots {
        items.push(ListItem::new("...").style(Style::default().fg(Color::DarkGray)));
    }

    let filter_text = if let Some(f) = &app.current_filter {
        format!(" Filter: {:?} ", f)
    } else {
        " All Branches ".to_string()
    };

    let title = format!(" {} │ Current: {} ", filter_text, app.current_branch);
    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL));

    if app.mode == AppMode::Diff {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
            .split(area);
        f.render_widget(list, chunks[0]);
        let diff = Paragraph::new(app.branch_info.as_str())
            .block(Block::default().title(" Diff & Commits ").borders(Borders::ALL))
            .scroll((app.info_scroll, 0));
        f.render_widget(diff, chunks[1]);
    } else {
        f.render_widget(list, area);
    }
}

pub fn render_filter(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)].as_ref())
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
        .split(area)[1];

    f.render_widget(Clear, inner);

    let options = vec![
        "0. All",
        "1. Merged (✓)",
        "2. Local Only (L)",
        "3. Stashed (S)",
        "4. Gone (⨯)",
        "5. Ahead (↑)",
        "6. Behind (↓)",
        "7. Unique Commits (▲)",
        "8. Remote Tracked (R)",
        "9. Remote Untracked (U)",
    ];

    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.filter_selected {
            style = style.fg(Color::Magenta).bg(Color::Rgb(40, 40, 40));
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let block = Block::default()
        .title(Line::from(" Filter by Status ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL);
    let list = List::new(items).block(block);
    f.render_widget(list, inner);
}

pub fn render_help(f: &mut Frame) {
    let area = f.area();
    f.render_widget(Clear, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // Logo
            Constraint::Min(10),   // Content
        ].as_ref())
        .split(area);

    let logo = Paragraph::new(ASCII_LOGO)
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center);
    f.render_widget(logo, chunks[0]);

    let block = Block::default()
        .title(Line::from(" Help & Legend ").alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    let help_text = vec![
        "Twigdrop helps you clean up your local branches safely.",
        "",
        "Status Icons:",
        "  ▲ (Red)     : Has Unique Commits (DANGER: Not in remote!)",
        "  ⨯ (Gray)    : Gone (Upstream branch was deleted)",
        "  ↑ (Yellow)  : Ahead of upstream (Local has new commits)",
        "  ↓ (Cyan)    : Behind upstream (Remote has new commits)",
        "  ✓ (Blue)    : Merged (Safe to delete)",
        "  L (Magenta) : Local Only (No tracking branch)",
        "  S (Green)   : Stashed changes exist for this branch",
        "  R (White)   : Remote Tracked",
        "  U (Gray)    : Remote Untracked",
        "  ● (Green)   : Clean / Normal",
        "",
        "Shortcuts:",
        "  ↑/k, ↓/j    : Navigate list",
        "  Double Click: Manage selected branch",
        "  f           : Open Filters",
        "  m / Enter   : Manage selected branch (Checkout, Diff, Delete)",
        "  q / Esc     : Quit / Back",
    ];

    let p = Paragraph::new(help_text.join("\n"))
        .block(block)
        .alignment(Alignment::Left);

    let help_inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(15), Constraint::Percentage(70), Constraint::Percentage(15)].as_ref())
        .split(chunks[1])[1];

    f.render_widget(p, help_inner);
}

pub fn render_manage(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
        .split(area)[1];

    f.render_widget(Clear, inner);

    let options = vec!["1. Checkout", "2. View Diff", "3. Delete (Individual)", "4. Help", "5. Cancel"];
    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.manage_selected {
            style = style.fg(Color::Cyan).bg(Color::Rgb(40, 40, 40));
        }
        if i == 2 { // Delete option
            style = style.fg(Color::Red);
            if i == app.manage_selected {
                style = style.bg(Color::Rgb(40, 40, 40));
            }
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let b_name = app.get_filtered_branches().get(app.selected).map(|b| b.name.as_str()).unwrap_or("none");
    let block = Block::default()
        .title(Line::from(format!(" Manage: {} ", b_name)).alignment(Alignment::Left))
        .title(Line::from(" [X] ").alignment(Alignment::Right))
        .borders(Borders::ALL);
    let list = List::new(items).block(block);
    f.render_widget(list, inner);
}

pub fn render_message(f: &mut Frame, msg: &str) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(40), Constraint::Percentage(30)].as_ref())
        .split(f.area())[1];

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(15), Constraint::Percentage(70), Constraint::Percentage(15)].as_ref())
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
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::White))
        .wrap(ratatui::widgets::Wrap { trim: true });

    f.render_widget(p, inner);
}
