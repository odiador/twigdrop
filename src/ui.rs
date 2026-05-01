use ratatui::{
    layout::{Constraint, Direction, Layout, Alignment},
    widgets::{Block, Borders, List, ListItem, Paragraph, Clear},
    style::{Style, Color},
    Frame,
};

use crate::app::{App, AppMode};
use crate::models::BranchStatus;

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Min(3), Constraint::Length(1)].as_ref())
        .split(f.area());

    let top_area = chunks[0];
    let list_area = chunks[1];
    let shortcuts_area = chunks[2];

    let small_ascii = r#"

████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
   ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
   ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
   ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;
    let title_p = Paragraph::new(small_ascii).style(Style::default().fg(Color::Cyan)).alignment(Alignment::Center);
    f.render_widget(title_p, top_area);

    let inner_height = list_area.height.saturating_sub(2) as usize; // Account for borders

    if inner_height == 0 {
        return;
    }

    let branches_len = app.branches.len();
    
    // Calculate the window of items to display
    let mut start = 0;
    let mut end = branches_len;

    if branches_len > inner_height {
        let half_height = inner_height / 2;
        if app.selected > half_height {
            start = app.selected - half_height;
        }
        end = start + inner_height;

        if end > branches_len {
            end = branches_len;
            start = end.saturating_sub(inner_height);
        }
    }

    app.list_start_index = start;

    let mut items: Vec<ListItem> = vec![];

    if start > 0 {
        items.push(ListItem::new("...").style(Style::default().fg(Color::DarkGray)));
        start += 1; // Make room for the top "..."
    }

    let mut actual_end = end;
    if end < branches_len {
        actual_end -= 1; // Make room for the bottom "..."
    }

    for i in start..actual_end {
        let b = &app.branches[i];
        let selected = i == app.selected;
        let marked = app.marked.get(i).copied().unwrap_or(false);

        let prefix = if marked { "[x]" } else { "[ ]" };

        let mut color = Color::Green;
        let mut icons = String::new();

        if b.status.contains(&BranchStatus::HasUniqueCommits) {
            color = Color::Red;
        } else if b.status.contains(&BranchStatus::Gone) {
            color = Color::DarkGray;
        } else if b.status.contains(&BranchStatus::Ahead) {
            color = Color::Yellow;
        } else if b.status.contains(&BranchStatus::Merged) {
            color = Color::Blue;
        }

        for s in &b.status {
            match s {
                BranchStatus::HasUniqueCommits => icons.push('▲'),
                BranchStatus::Gone => icons.push('⨯'),
                BranchStatus::Ahead => icons.push('↑'),
                BranchStatus::Merged => icons.push('✓'),
                BranchStatus::Safe => icons.push('●'),
            }
        }

        let line = format!("{} {:<3} {}", prefix, icons, b.name);
        
        let mut item = ListItem::new(line).style(Style::default().fg(color));

        if selected {
            item = item.style(Style::default().fg(color).bg(Color::Rgb(40, 40, 40)));
        }
        items.push(item);
    }

    if end < branches_len {
        items.push(ListItem::new("...").style(Style::default().fg(Color::DarkGray)));
    }

    let list = List::new(items)
        .block(Block::default().title("Branches").borders(Borders::ALL));

    if app.mode == AppMode::Diff {
        let split_list_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
            .split(list_area);

        f.render_widget(list, split_list_area[0]);

        let info_paragraph = Paragraph::new(app.branch_info.as_str())
            .block(Block::default().title(" Diff & Commits (Up/Down to scroll, m to back) ").borders(Borders::ALL))
            .scroll((app.info_scroll, 0));
        f.render_widget(info_paragraph, split_list_area[1]);
    } else {
        f.render_widget(list, list_area);
    }

    let shortcuts = Paragraph::new(" ↑/k: up │ ↓/j: down │ [space]: mark │ d: delete │ m: manage branch │ f: filter │ h: help │ q: quit ")
        .style(Style::default().fg(Color::DarkGray));
    f.render_widget(shortcuts, shortcuts_area);

    if app.mode == AppMode::Help {
        render_help(f);
    } else if app.mode == AppMode::Manage {
        render_manage(f, app);
    }
}

fn render_manage(f: &mut Frame, app: &App) {
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)].as_ref())
        .split(f.area());

    let inner = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)].as_ref())
        .split(area[1])[1];

    f.render_widget(Clear, inner);

    let options = vec![
        "1. Checkout",
        "2. View Diff",
        "3. Explain Icons",
        "4. Cancel",
    ];

    let mut items = vec![];
    for (i, opt) in options.iter().enumerate() {
        let mut style = Style::default().fg(Color::Gray);
        if i == app.manage_selected {
            style = style.fg(Color::Cyan).bg(Color::Rgb(40, 40, 40));
        }
        items.push(ListItem::new(*opt).style(style));
    }

    let b_name = if app.branches.is_empty() { "none" } else { &app.branches[app.selected].name };
    let block = Block::default()
        .title(format!(" Manage: {} ", b_name))
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));

    let list = List::new(items).block(block);
    f.render_widget(list, inner);
}

fn render_help(f: &mut Frame) {
    let area = f.area();
    
    f.render_widget(Clear, area);

    let ascii_art = r#"
████████╗██╗    ██╗██╗ ██████╗ ██████╗ ██████╗  ██████╗ ██████╗ 
╚══██╔══╝██║    ██║██║██╔════╝ ██╔══██╗██╔══██╗██╔═══██╗██╔══██╗
   ██║   ██║ █╗ ██║██║██║  ███╗██║  ██║██████╔╝██║   ██║██████╔╝
   ██║   ██║███╗██║██║██║   ██║██║  ██║██╔══██╗██║   ██║██╔═══╝ 
   ██║   ╚███╔███╔╝██║╚██████╔╝██████╔╝██║  ██║╚██████╔╝██║     
   ╚═╝    ╚══╝╚══╝ ╚═╝ ╚═════╝ ╚═════╝ ╚═╝  ╚═╝ ╚═════╝ ╚═╝     
"#;

    let help_text = format!(
        "{}\n\n\
        Twigdrop is a tool to safely clean up your local git branches.\n\n\
        Legend:\n\
        ▲ (Red)    : Has Unique Commits (Danger!)\n\
        ⨯ (Gray)   : Gone (Upstream deleted)\n\
        ↑ (Yellow) : Ahead of upstream\n\
        ✓ (Blue)   : Merged\n\
        ● (Green)  : Safe to delete / Normal\n\n\
        Press 'h' or 'q' or 'Esc' to close this help window.\n\n\
        Made by: odiador ❤️ for the community",
        ascii_art
    );

    let block = Block::default()
        .title(" Help ")
        .borders(Borders::ALL)
        .style(Style::default().bg(Color::Reset));
        
    let p = Paragraph::new(help_text)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(p, area);
}
