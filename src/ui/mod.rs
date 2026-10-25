pub mod animations;
pub mod components;
pub mod layout;
pub mod markdown;
pub mod screens;
pub mod theme;

pub use markdown::render_markdown;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
};

use crate::app::App;
use crate::state::ui::{AppMode, PrimaryMode};
use crate::ui::animations::snap::SnapCell;

pub fn draw(f: &mut Frame, app: &mut App, path: &str) {
    // 0. Update phase (needs `&mut`): drawer caches.
    if app.ui.primary_mode == PrimaryMode::Branches && app.ui.show_inspector_drawer {
        app.refresh_inspector_commits(path);
    }
    if app.ui.show_nav_sidebar {
        app.refresh_sidebar_worktrees(path);
    }
    if app.ui.primary_mode == PrimaryMode::Files && app.repo.file_tree.is_empty() {
        app.load_file_tree(path);
    }

    // Advance the busy spinner once per frame while background work runs.
    if app.is_busy() {
        app.frame_cache.borrow_mut().spinner_tick += 1;
    }
    // Expire stale toasts.
    app.ui.expire_toast();

    let toast_live = app.ui.toast.as_ref().is_some_and(|t| t.is_live());
    // NOTE: two separate splits — a zero-height middle chunk would still
    // claim half the leftover space (`Min(0)` shares it), pushing the
    // footer mid-screen when no toast is live.
    let (main_area, toast_area, footer_area) = if toast_live {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(1),
                Constraint::Length(1),
            ])
            .split(f.area());
        (chunks[0], Some(chunks[1]), chunks[2])
    } else {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(f.area());
        (chunks[0], None, chunks[1])
    };

    // 1. Primary views (pure: `&App`).
    render_views(f, app, main_area);

    // 2. Snap deletion (update): ALWAYS completes, even with animations
    // off — otherwise confirmed deletes silently do nothing (the modal
    // is already popped at this point). Visual capture only when enabled.
    let needs_capture = matches!(&app.ui.snap_animation, Some(anim) if !anim.captured);
    if needs_capture {
        if app.config.enable_animations {
            // Map confirmed rows to this frame's hit-test positions.
            let positions: Vec<(String, u16)> = {
                let cache = app.frame_cache.borrow();
                cache
                    .branch_screen_positions
                    .iter()
                    .filter_map(|(idx, y)| {
                        app.repo.branches.get(*idx).map(|b| (b.name.clone(), *y))
                    })
                    .collect()
            };
            if let Some(anim) = app.ui.snap_animation.as_mut() {
                for row in anim.rows.iter_mut() {
                    row.screen_y = positions
                        .iter()
                        .find(|(name, _)| *name == row.branch_name)
                        .map(|(_, y)| *y);
                }
                let buf = f.buffer_mut();
                for row in anim.rows.iter_mut() {
                    if let Some(y) = row.screen_y {
                        for x in 0..main_area.width {
                            let cell = &buf[(main_area.x + x, main_area.y + y)];
                            row.cells.push(SnapCell {
                                x,
                                ch: ' ', // Not used anymore but kept for struct
                                color: cell.fg,
                                dissolved: false,
                            });
                        }
                    }
                }
            }
        }
        if let Some(anim) = app.ui.snap_animation.as_mut() {
            anim.captured = true;
        }
        app.ui.pop_modal(); // pop the confirm delete modal
        app.complete_snap_deletion(path);
        if !app.config.enable_animations {
            app.ui.snap_animation = None;
        }
    }

    // 2b. Advance a running animation; drop it when done so the loop
    // idles again (and `is_busy` clears).
    if app.config.enable_animations {
        let done = if let Some(anim) = app.ui.snap_animation.as_mut() {
            anim.tick();
            anim.phase == crate::ui::animations::snap::SnapPhase::Done
        } else {
            false
        };
        if done {
            app.ui.snap_animation = None;
        }
    }

    // 3. Chrome: particles + status + footer + modals (pure: `&App`).
    render_chrome(f, app, toast_area, footer_area);
}

/// Pure render: reads `App`, writes the frame. Per-frame hit-test data
/// goes to `App::frame_cache` (interior mutability), never into domain
/// or UI selection state. `git`/I/O here is a bug — see ARCHITECTURE §1.
pub fn render_views(f: &mut Frame, app: &App, area: Rect) {
    // 1. Primary Views
    match app.ui.primary_mode {
        PrimaryMode::Branches => {
            screens::render_main_list(f, area, app);
        }
        PrimaryMode::Files => {
            screens::render_directory_searcher(f, area, app);
        }
        PrimaryMode::Commits => {
            screens::render_commits(f, area, app);
        }
        PrimaryMode::Stashes => {
            screens::render_stash_detail(f, area, app);
        }
        PrimaryMode::PullRequests => {
            screens::render_pull_requests(f, area, app);
        }
    }
}

/// Chrome over an already-rendered frame: particles + status + footer +
/// modals. Pure (`&App`); particle animation only reads its state.
pub fn render_chrome(f: &mut Frame, app: &App, toast_area: Option<Rect>, footer_area: Rect) {
    // 2. Particles (read-only animation state)
    if app.config.enable_animations
        && let Some(ref anim) = app.ui.snap_animation
    {
        for p in &anim.particles.particles {
            if p.x >= 0.0
                && p.x < f.area().width as f32
                && p.y >= 0.0
                && p.y < f.area().height as f32
            {
                let cell = &mut f.buffer_mut()[(p.x as u16, p.y as u16)];
                let density_chars = [" ", ".", ":", "-", "=", "+", "*", "#", "%", "@"];
                cell.set_symbol(density_chars[p.density as usize]);
                cell.set_fg(p.color);
            }
        }
    }

    let loc = app.locale();

    // 3. Status Bar (spinner prefix while background work runs)
    let status_prefix = match app.ui.primary_mode {
        PrimaryMode::Branches => {
            let filter_text = if let Some(f) = &app.ui.current_filter {
                format!("sort: {f:?}")
            } else {
                loc.common.none.to_string()
            };
            format!(
                " 🧹 twigdrop │ {} · {} {} · {} │",
                app.repo.current_branch,
                app.ui.filtered_indices.len(),
                loc.footer.status_tag,
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
        PrimaryMode::PullRequests => {
            format!(" 🐙 PRs │ {} │", app.repo.current_branch)
        }
    };
    let status_prefix = if app.is_busy() {
        let tick = usize::try_from(app.frame_cache.borrow().spinner_tick).unwrap_or(0);
        let frames = crate::ui::theme::SPINNER_FRAMES;
        format!("{} {}", frames[tick % frames.len()], status_prefix)
    } else {
        status_prefix
    };

    // 4. Footer shortcuts — static per (View, Modal) context.
    // Never reactive to a held modifier (no stored shift/alt state).
    let footer_shortcuts = if let AppMode::CodePreview(_) = app.ui.current_mode() {
        loc.footer.preview
    } else if *app.ui.current_mode() == AppMode::Diff {
        loc.footer.diff
    } else if *app.ui.current_mode() == AppMode::Switcher {
        loc.footer.switcher
    } else {
        match app.ui.primary_mode {
            PrimaryMode::Branches => loc.footer.branches_normal,
            PrimaryMode::Files => loc.footer.files_normal,
            PrimaryMode::Commits => loc.footer.commits_normal,
            PrimaryMode::Stashes => loc.footer.stashes_normal,
            PrimaryMode::PullRequests => loc.footer.pull_requests_normal,
        }
    };

    let theme = app.theme();
    let footer_line = ratatui::text::Line::from(vec![
        ratatui::text::Span::styled(
            status_prefix,
            theme.nav.add_modifier(ratatui::style::Modifier::BOLD),
        ),
        ratatui::text::Span::styled(
            footer_shortcuts,
            ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray),
        ),
    ]);

    let footer = ratatui::widgets::Paragraph::new(footer_line);
    f.render_widget(footer, footer_area);

    // 4b. Toast bar (only while live).
    if let (Some(area), Some(toast)) = (toast_area, app.ui.toast.as_ref().filter(|t| t.is_live())) {
        use crate::state::toast::ToastKind;
        let (glyph, style) = match toast.kind {
            ToastKind::Success => (
                "✓",
                theme.success.add_modifier(ratatui::style::Modifier::BOLD),
            ),
            ToastKind::Error => (
                "✗",
                theme.danger.add_modifier(ratatui::style::Modifier::BOLD),
            ),
            ToastKind::Info => ("ℹ", theme.info.add_modifier(ratatui::style::Modifier::BOLD)),
        };
        let mut spans = vec![
            ratatui::text::Span::styled(format!(" {glyph} "), style),
            ratatui::text::Span::styled(toast.text.clone(), theme.base),
        ];
        if toast.undo.is_some() {
            spans.push(ratatui::text::Span::styled(
                format!("  {}", loc.branches.toast_undo_hint),
                theme.warning.add_modifier(ratatui::style::Modifier::BOLD),
            ));
        }
        f.render_widget(
            ratatui::widgets::Paragraph::new(ratatui::text::Line::from(spans)),
            area,
        );
    }

    // 5. Modals and Overlays (overlay set owned by `AppMode::is_overlay`)
    let is_modal = !app.ui.modal_stack.is_empty();
    let is_overlay_modal = app.ui.current_mode().is_overlay();
    let is_code_preview = matches!(app.ui.current_mode(), AppMode::CodePreview(_));
    if is_modal && !is_overlay_modal && !is_code_preview {
        let overlay = Rect::new(0, 0, f.area().width, f.area().height);
        f.render_widget(ratatui::widgets::Clear, overlay);
    }

    match app.ui.current_mode() {
        AppMode::Normal => {}
        AppMode::Help => screens::render_help(f, app),
        AppMode::Message(msg) => screens::render_message(f, msg, &theme),
        AppMode::Settings => screens::render_settings(f, app),
        AppMode::Filter => screens::render_filter(f, app),
        AppMode::Manage => screens::render_manage(f, app),
        AppMode::Diff => screens::render_diff(f, app),
        AppMode::CodePreview(_) => {} // Rendered inline in render_directory_searcher
        AppMode::ConfirmDelete(names) => screens::render_confirm_delete(f, names, loc, &theme),
        AppMode::CreateBranch(input) => screens::render_create_branch(f, input, &theme),
        AppMode::RenameBranch(old, input) => screens::render_rename_branch(f, old, input, &theme),
        AppMode::CommitAction(hash) => screens::render_commit_action(f, app, hash),
        AppMode::DatePicker(state) => screens::render_date_picker(f, app, state),
        AppMode::CommitFiles(hash, files) => screens::render_commit_files(f, app, hash, files),
        AppMode::InteractiveRebase => screens::render_interactive_rebase(f, app),
        AppMode::Shell(input) => screens::render_shell(f, input),
        AppMode::QuickActions => screens::render_quick_actions(f, app),
        AppMode::Switcher => screens::render_switcher(f, app),
        AppMode::MainMenu => screens::render_main_menu(f, app),
        AppMode::CommandPalette(state) => screens::render_command_palette(f, state, loc, &theme),
        AppMode::FolderProfile(state) => screens::render_folder_profile(f, app, state),
        AppMode::PrReview(state) => screens::render_pr_review(f, app, state),
    }
}

#[cfg(test)]
mod tests;
