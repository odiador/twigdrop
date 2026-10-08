use ratatui::{
    Frame,
    layout::Alignment,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
};

use crate::app::App;
use crate::state::ui::AppMode;
use crate::ui::theme::Theme;

pub fn render_switcher(f: &mut Frame, app: &App) {
    let theme = Theme::dark_default();
    let (mx, my) = crate::ui::layout::SWITCHER_MODAL;
    let inner = crate::ui::components::centered_rect(mx, my, f.area());

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
            style = theme.select_hi.add_modifier(Modifier::BOLD);
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
