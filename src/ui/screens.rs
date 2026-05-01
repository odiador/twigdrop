use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment, Rect},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    style::{Style, Color},
    Frame,
};

use crate::app::{App, AppMode};
use crate::ui::components::get_status_icons;
use crate::ui::ASCII_LOGO;

pub fn render_main_list(f: &mut Frame, area: Rect, app: &mut App) {
    let branches_len = app.branches.len();
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

    let mut items: Vec<ListItem> = vec![];
    let show_top_dots = start > 0;
    let show_bottom_dots = (start + inner_height) < branches_len;

    if show_top_dots {
        items.push(ListItem::new("...").style(Style::default().fg(Color::DarkGray)));
    }

    // Number of branch items to show
    let branch_items_to_show = inner_height 
        - (if show_top_dots { 1 } else { 0 }) 
        - (if show_bottom_dots { 1 } else { 0 });

    for i in 0..branch_items_to_show {
        let branch_idx = start + i;
        if branch_idx >= branches_len { break; }
        
        let b = &app.branches[branch_idx];
        let selected = branch_idx == app.selected;
        let marked = app.marked.get(branch_idx).copied().unwrap_or(false);

        let checkbox = if marked { "[x]" } else { "[ ]" };
        let (icons, color) = get_status_icons(&b.status);

        let line = format!("{}  {:<4}  {}", checkbox, icons, b.name);
        let mut item = ListItem::new(line).style(Style::default().fg(color));

        if selected {
            item = item.style(Style::default().fg(color).bg(Color::Rgb(40, 40, 40)));
        }
        items.push(item);
    }

    if show_bottom_dots {
        items.push(ListItem::new("...").style(Style::default().fg(Color::DarkGray)));
    }

    let list = List::new(items)
        .block(Block::default().title(" Branches ").borders(Borders::ALL));

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
        .title(" Help & Legend ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    let help_text = vec![
        "Twigdrop helps you clean up your local branches safely.",
        "",
        "Actions:",
        "  [x]  : Marked for bulk deletion (Space to toggle)",
        "  [ ]  : Not marked",
        "",
        "Status Icons:",
        "  ▲ (Red)     : Has Unique Commits (DANGER: Not in remote!)",
        "  ⨯ (Gray)    : Gone (Upstream branch was deleted)",
        "  ↑ (Yellow)  : Ahead of upstream (Local has new commits)",
        "  ↓ (Cyan)    : Behind upstream (Remote has new commits)",
        "  ✓ (Blue)    : Merged (Safe to delete)",
        "  L (Magenta) : Local Only (No tracking branch)",
        "  S (Green)   : Stashed changes exist for this branch",
        "  ● (Green)   : Clean / Normal",
        "",
        "Shortcuts:",
        "  ↑/k, ↓/j    : Navigate list",
        "  Space       : Mark/Unmark branch",
        "  d           : Delete marked branches",
        "  m / Enter   : Manage selected branch (Checkout, Diff)",
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

    let options = vec!["1. Checkout", "2. View Diff", "3. Help", "4. Cancel"];
    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.manage_selected {
            style = style.fg(Color::Cyan).bg(Color::Rgb(40, 40, 40));
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let b_name = app.branches.get(app.selected).map(|b| b.name.as_str()).unwrap_or("none");
    let list = List::new(items).block(Block::default().title(format!(" Manage: {} ", b_name)).borders(Borders::ALL));
    f.render_widget(list, inner);
}
