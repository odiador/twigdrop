pub mod animations;
pub mod components;
pub mod screens;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::Color,
};

use crate::app::App;
use crate::state::ui::{AppMode, PrimaryMode};
use crate::ui::animations::{DENSITY_CHARS, SnapPhase};

pub fn draw(f: &mut Frame, app: &mut App, path: &str) {
    let area = f.area();
    let main_constraints = if app.ui.show_terminal {
        vec![
            Constraint::Min(3),
            Constraint::Percentage(30),
            Constraint::Length(1),
        ]
    } else {
        vec![Constraint::Min(3), Constraint::Length(1)]
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(main_constraints)
        .split(area);

    // 1. Regular Rendering (includes side-by-side preview if active)
    match app.ui.primary_mode {
        PrimaryMode::Branches => {
            screens::render_main_list(f, chunks[0], app);
        }
        PrimaryMode::Files => {
            screens::render_directory_searcher(f, chunks[0], app);
        }
        PrimaryMode::Commits => {
            screens::render_commits(f, chunks[0], app);
        }
    }

    // 2. Capture and Animate
    if app.config.enable_animations {
        if let Some(ref mut anim) = app.ui.snap_animation {
            if !anim.captured {
                let buf = f.buffer_mut();
                for row in anim.rows.iter_mut() {
                    // Find screen Y for this branch
                    if let Some(&(_, screen_y)) =
                        app.ui.branch_screen_positions.iter().find(|&&(idx, _)| {
                            if idx < app.ui.filtered_indices.len() {
                                app.repo.branches[app.ui.filtered_indices[idx]].name
                                    == row.branch_name
                            } else {
                                false
                            }
                        })
                    {
                        row.screen_y = Some(screen_y);
                        // Capture cells at this Y
                        for x in 0..area.width {
                            let cell = &buf[(x, screen_y)];
                            let ch = cell.symbol().chars().next().unwrap_or(' ');
                            if ch != ' ' {
                                row.cells.push(crate::ui::animations::snap::SnapCell {
                                    x,
                                    ch,
                                    color: cell.fg,
                                    dissolved: false,
                                });
                            }
                        }
                    }
                }
                anim.captured = true;
            }

            // Apply Dissolve on Buffer
            let buf = f.buffer_mut();
            if anim.phase == SnapPhase::Flash {
                for x in 0..area.width {
                    for y in 0..area.height {
                        buf[(x, y)].set_bg(Color::Rgb(30, 30, 40));
                        buf[(x, y)].set_fg(Color::White);
                    }
                }
            }

            for row in &anim.rows {
                if let Some(y) = row.screen_y {
                    for cell in &row.cells {
                        if cell.dissolved {
                            buf[(cell.x, y)].set_symbol(" ");
                        }
                    }
                }
            }

            for p in &anim.particles.particles {
                if p.x >= 0.0 && p.x < area.width as f32 && p.y >= 0.0 && p.y < area.height as f32 {
                    let cell = &mut buf[(p.x as u16, p.y as u16)];
                    cell.set_symbol(&DENSITY_CHARS[p.density as usize].to_string());
                    cell.set_fg(p.color);
                }
            }

            anim.tick();

            if anim.phase == SnapPhase::Done {
                let msg = app.apply_snap_deletion(path);
                app.ui.push_modal(AppMode::Message(msg));
                app.ui.snap_animation = None;
            }
        }
    } else if app.ui.snap_animation.is_some() {
        let msg = app.apply_snap_deletion(path);
        app.ui.push_modal(AppMode::Message(msg));
        app.ui.snap_animation = None;
    }

    if app.ui.show_terminal {
        let terminal_block = ratatui::widgets::Block::default()
            .title(" Integrated TTY (Alt+j to toggle) ")
            .borders(ratatui::widgets::Borders::ALL)
            .border_style(
                ratatui::style::Style::default().fg(ratatui::style::Color::Rgb(74, 79, 106)),
            );
        let terminal_placeholder = ratatui::widgets::Paragraph::new(
            "Terminal session placeholder...\n(Working on full PTY integration)",
        )
        .block(terminal_block)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
        f.render_widget(terminal_placeholder, chunks[1]);
    }

    let footer_area = if app.ui.show_terminal {
        chunks[2]
    } else {
        chunks[1]
    };

    // Status prefix
    let status_prefix = match app.ui.primary_mode {
        PrimaryMode::Branches => {
            let filter_text = if let Some(f) = &app.ui.current_filter {
                format!("sort: {:?}", f)
            } else {
                "none".to_string()
            };
            format!(
                " 🧹 twigdrop │ {} · {} branches · {} │",
                app.repo.current_branch,
                app.ui.filtered_indices.len(),
                filter_text
            )
        }
        PrimaryMode::Files => {
            format!(" 📂 Files │ {} │", app.repo.current_branch)
        }
        PrimaryMode::Commits => {
            format!(" 🌳 Commits │ {} │", app.repo.current_branch)
        }
    };

    // 4. Footer shortcuts
    let footer_shortcuts = if let AppMode::CodePreview(_) = app.ui.current_mode() {
        " hjkl: navigate │ Esc: close │ [ / ]: resize sidebar "
    } else if *app.ui.current_mode() == AppMode::Diff {
        " Shift+F: AI Auto-Fix Conflicts │ q/Esc: Back "
    } else if *app.ui.current_mode() == AppMode::CommitsView {
        " ↑/k, ↓/j: navigate │ Enter: select │ Esc: close "
    } else if *app.ui.current_mode() == AppMode::Switcher {
        " ↑/k, ↓/j: navigate │ Enter: confirm │ Esc/q: cancel "
    } else if app.ui.shift_pressed {
        match app.ui.primary_mode {
            PrimaryMode::Branches => {
                " S: Stash Mgr │ C: Unpushed Commits │ D: Delete ALL Selected │ h: Legend │ q: quit "
            }
            PrimaryMode::Files => " S: Stash Mgr │ C: Unpushed Commits │ h: Legend │ q: quit ",
            PrimaryMode::Commits => " S: Stash Mgr │ h: Legend │ q: quit ",
        }
    } else if app.ui.alt_pressed {
        match app.ui.primary_mode {
            PrimaryMode::Branches => {
                " ↑/↓: move │ d: switch mode │ Alt+t: External TTY │ Alt+j: TTY │ f: filter "
            }
            PrimaryMode::Files => {
                " ↑/↓: move │ d: switch mode │ v: IDE (Path) │ a: Alt IDE (Path) │ Alt+t: External TTY │ Alt+j: TTY "
            }
            PrimaryMode::Commits => {
                " ↑/↓: move │ d: switch mode │ Alt+t: External TTY │ Alt+j: TTY "
            }
        }
    } else {
        match app.ui.primary_mode {
            PrimaryMode::Branches => {
                " ↑/↓: move │ Shift+Tab: app switcher │ d: files │ f: filter │ /: search │ c: create │ p: prune │ :: actions │ !: shell │ Shift+D: bulk delete │ m: manage │ ?: help │ q: quit "
            }
            PrimaryMode::Files => {
                " ↑/↓: move │ Shift+Tab: app switcher │ d: commits │ e: explorer │ v: IDE │ s: stage/unstage │ !: shell │ t: TTY (Alt+j toggle) │ ?: help │ q: quit "
            }
            PrimaryMode::Commits => {
                " ↑/↓: move │ Shift+Tab: app switcher │ d: branches │ Enter: actions │ !: shell │ ?: help │ q: quit "
            }
        }
    };

    let footer_line = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled(
            status_prefix,
            ratatui::style::Style::default()
                .fg(ratatui::style::Color::Rgb(180, 190, 254))
                .add_modifier(ratatui::style::Modifier::BOLD),
        ),
        ratatui::text::Span::styled(
            footer_shortcuts,
            ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray),
        ),
    ]);

    let footer = ratatui::widgets::Paragraph::new(footer_line);
    f.render_widget(footer, footer_area);

    // Apply global darkening overlay for modals
    let is_modal = !matches!(
        app.ui.current_mode(),
        AppMode::Normal | AppMode::CodePreview(_) | AppMode::Diff | AppMode::BranchesView | AppMode::FilesView | AppMode::CommitsView
    );

    if is_modal {
        let area = f.area();
        let buf = f.buffer_mut();
        for x in area.left()..area.right() {
            for y in area.top()..area.bottom() {
                let cell = &mut buf[(x, y)];
                cell.set_bg(ratatui::style::Color::Rgb(15, 15, 20)); // Dark background
                cell.set_fg(ratatui::style::Color::Rgb(70, 70, 85)); // Muted foreground
            }
        }
    }

    // 3. Modals and Overlays
    match app.ui.current_mode() {
        AppMode::Help => screens::render_help_content(f, f.area(), app),
        AppMode::StashDetail => screens::render_stash_detail(f, f.area(), app),
        AppMode::Manage => screens::render_manage(f, app),
        AppMode::Filter => screens::render_filter(f, app),
        AppMode::MainMenu => screens::render_main_menu(f, app),
        AppMode::Message(msg) => screens::render_message(f, msg),
        AppMode::Settings => screens::render_settings(f, app),
        AppMode::Search => screens::render_search(f, app),
        AppMode::ConfirmDelete(names) => screens::render_confirm_delete(f, names),
        AppMode::CreateBranch(input) => screens::render_create_branch(f, input),
        AppMode::DatePicker(state) => screens::render_date_picker(f, app, state),
        AppMode::CommitAction(hash) => screens::render_commit_action(f, app, hash),
        AppMode::InteractiveRebase => screens::render_interactive_rebase(f, app),
        AppMode::Shell(input) => screens::render_shell(f, input),
        AppMode::QuickActions => screens::render_quick_actions(f, app),
        AppMode::Switcher => screens::render_switcher(f, app),
        _ => {}
    }
}
