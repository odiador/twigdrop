pub mod components;
pub mod screens;
pub mod animations;

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use crate::app::App;
use crate::state::ui::{AppMode, PrimaryMode};
use crate::ui::animations::snap::SnapCell;

pub fn draw(f: &mut Frame, app: &mut App, path: &str) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(1)])
        .split(f.area());

    // 1. Primary Views
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
        PrimaryMode::Stashes => {
            screens::render_stash_detail(f, chunks[0], app);
        }
    }

    // 2. Capture and Animate
    if app.config.enable_animations && let Some(ref mut anim) = app.ui.snap_animation {
        if !anim.captured {
            let buf = f.buffer_mut();
            for row in anim.rows.iter_mut() {
                if let Some(y) = row.screen_y {
                    for x in 0..chunks[0].width {
                        let cell = &buf[(chunks[0].x + x, chunks[0].y + y)];
                        row.cells.push(SnapCell {
                            x,
                            ch: ' ', // Not used anymore but kept for struct
                            color: cell.fg,
                            dissolved: false,
                        });
                    }
                }
            }
            anim.captured = true;
            app.ui.pop_modal(); // pop the confirm delete modal
            app.delete_selected_branches(path);
        }

        if let Some(ref mut anim) = app.ui.snap_animation {
            for p in &anim.particles.particles {
                if p.x >= 0.0 && p.x < f.area().width as f32 && p.y >= 0.0 && p.y < f.area().height as f32 {
                    let cell = &mut f.buffer_mut()[(p.x as u16, p.y as u16)];
                    let density_chars = [" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];
                    cell.set_symbol(density_chars[p.density as usize]);
                    cell.set_fg(p.color);
                }
            }
        }
    }

    // 3. Status Bar
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
        PrimaryMode::Stashes => {
            format!(" 📦 Stashes │ {} │", app.repo.current_branch)
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
                " S: Stash Mgr │ C: Commit Tree │ D: Delete Selected │ h: Legend │ q: quit "
            }
            _ => " S: Stash Mgr │ C: Commit Tree │ h: Legend │ q: quit ",
        }
    } else if app.ui.alt_pressed {
        match app.ui.primary_mode {
            PrimaryMode::Branches => {
                " ↑/↓: move │ d: switch mode │ Alt+t: Ext TTY │ Alt+j: TTY │ f: filter "
            }
            PrimaryMode::Files => {
                " ↑/↓: move │ d: switch mode │ v: IDE (Path) │ a: Alt IDE (Path) │ Alt+t: Ext TTY │ Alt+j: TTY "
            }
            _ => " ↑/↓: move │ d: switch mode │ Alt+t: Ext TTY │ Alt+j: TTY ",
        }
    } else {
        match app.ui.primary_mode {
            PrimaryMode::Branches => {
                " ↑/↓: move │ Shift+Tab: switcher │ d: files │ f: filter │ /: search │ c: create │ p: prune │ :: actions │ !: shell │ Shift+D: bulk delete │ m: manage │ ?: help │ q: quit "
            }
            PrimaryMode::Files => {
                " ↑/↓: move │ Shift+Tab: switcher │ d: commits │ e: explorer │ v: IDE │ s: stage/unstage │ !: shell │ t: TTY (Alt+j toggle) │ ?: help │ q: quit "
            }
            PrimaryMode::Commits => {
                " ↑/↓: move │ Shift+Tab: switcher │ d: stashes │ Enter: details │ ?: help │ q: quit "
            }
            PrimaryMode::Stashes => {
                " ↑/↓: move │ Shift+Tab: switcher │ d: branches │ a: apply │ ?: help │ q: quit "
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
    f.render_widget(footer, chunks[1]);

    // 5. Modals and Overlays
    let is_modal = !app.ui.modal_stack.is_empty();
    if is_modal {
        let overlay = Rect::new(0, 0, f.area().width, f.area().height);
        f.render_widget(ratatui::widgets::Clear, overlay);
    }

    match app.ui.current_mode() {
        AppMode::Normal => {}
        AppMode::Help => screens::render_help(f, app),
        AppMode::Message(msg) => screens::render_message(f, msg),
        AppMode::Settings => screens::render_settings(f, app),
        AppMode::Search => screens::render_search(f, app),
        AppMode::Filter => screens::render_filter(f, app),
        AppMode::Manage => screens::render_manage(f, app),
        AppMode::StashDetail => screens::render_stash_detail(f, f.area(), app),
        AppMode::Diff => screens::render_diff(f, app),
        AppMode::CodePreview(state) => screens::render_code_preview(f, app, f.area(), state),
        AppMode::ConfirmDelete(names) => screens::render_confirm_delete(f, names),
        AppMode::CreateBranch(input) => screens::render_create_branch(f, input),
        AppMode::CommitAction(hash) => screens::render_commit_action(f, app, hash),
        AppMode::DatePicker(state) => screens::render_date_picker(f, app, state),
        AppMode::CommitFiles(hash, files) => screens::render_commit_files(f, app, hash, files),
        AppMode::InteractiveRebase => screens::render_interactive_rebase(f, app),
        AppMode::Shell(input) => screens::render_shell(f, input),
        AppMode::QuickActions => screens::render_quick_actions(f, app),
        AppMode::Switcher => screens::render_switcher(f, app),
        _ => {}
    }
}
