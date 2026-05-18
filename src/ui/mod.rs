pub mod components;
pub mod screens;
pub mod animations;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
    style::Color,
};

use crate::app::{App, AppMode, PrimaryMode};
use crate::ui::animations::{SnapPhase, DENSITY_CHARS};

pub fn draw(f: &mut Frame, app: &mut App, path: &str) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(1)].as_ref())
        .split(area);

    // 1. Regular Rendering
    match app.primary_mode {
        PrimaryMode::Branches => {
            screens::render_main_list(f, chunks[0], app);
        }
        PrimaryMode::Files => {
            screens::render_directory_searcher(f, chunks[0], app);
        }
    }

    // 2. Capture and Animate
    if let Some(ref mut anim) = app.snap_animation {
        if !anim.captured {
            let buf = f.buffer_mut();
            for row in anim.rows.iter_mut() {
                // Find screen Y for this branch
                if let Some(&(_, screen_y)) = app.branch_screen_positions.iter()
                    .find(|&&(idx, _)| {
                        if idx < app.branch_state.filtered_indices.len() {
                            app.branch_state.branches[app.branch_state.filtered_indices[idx]].name == row.branch_name
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
            // Flash effect: fill background with gray temporarily
            for x in 0..area.width {
                for y in 0..area.height {
                    buf[(x, y)].set_bg(Color::Rgb(60, 60, 60));
                }
            }
        }

        for row in &anim.rows {
            if let Some(y) = row.screen_y {
                for cell in &row.cells {
                    if cell.dissolved {
                        // Clear the cell from the main buffer
                        buf[(cell.x, y)].set_symbol(" ");
                    }
                }
            }
        }

        // Render Particles
        for p in &anim.particles.particles {
            if p.x >= 0.0 && p.x < area.width as f32 && p.y >= 0.0 && p.y < area.height as f32 {
                let cell = &mut buf[(p.x as u16, p.y as u16)];
                cell.set_symbol(&DENSITY_CHARS[p.density as usize].to_string());
                cell.set_fg(p.color);
            }
        }
        
        anim.tick();
        
        if anim.phase == SnapPhase::Done {
            // Perform actual deletion after animation
            let msg = app.apply_snap_deletion(path);
            app.mode = AppMode::Message(msg);
            app.snap_animation = None;
        }
    }

    // 3. Modals and Overlays
    match &app.mode {
        AppMode::Manage => screens::render_manage(f, app),
        AppMode::Filter => screens::render_filter(f, app),
        AppMode::Message(msg) => screens::render_message(f, msg),
        AppMode::Help => screens::render_help_content(f, chunks[0], app),
        AppMode::StashDetail => screens::render_stash_detail(f, chunks[0], app),
        AppMode::Settings => screens::render_settings(f, app),
        AppMode::Search => screens::render_search(f, app),
        AppMode::CodePreview(path_str, content) => screens::render_code_preview(f, app, path_str, content),
        _ => {}
    }

    // 4. Footer shortcuts
    let footer_text = if app.mode == AppMode::Diff {
        " Shift+F: AI Auto-Fix Conflicts │ q/Esc: Back "
    } else if app.shift_pressed {
        match app.primary_mode {
            PrimaryMode::Branches => " S: Stash Mgr │ D: Delete ALL Selected │ h: Legend │ q: quit ",
            PrimaryMode::Files => " S: Stash Mgr │ h: Legend │ q: back ",
        }
    } else if app.alt_pressed {
        match app.primary_mode {
            PrimaryMode::Branches => " ↑/↓: move │ d: switch mode │ Alt+t: External TTY │ f: filter ",
            PrimaryMode::Files => {
                " ↑/↓: move │ d: switch mode │ v: IDE (Path) │ a: Alt IDE (Path) │ Alt+t: External TTY "
            }
        }
    } else {
        match app.primary_mode {
            PrimaryMode::Branches => {
                " ↑/↓: move │ d: files │ /: search │ p: prune │ f: filter │ m: manage │ h: help │ q: quit "
            }
            PrimaryMode::Files => {
                " ↑/↓: move │ d: branches │ Enter: preview │ v: IDE │ t: Inline TTY │ h: help "
            }
        }
    };

    let footer = ratatui::widgets::Paragraph::new(footer_text)
        .style(ratatui::style::Style::default().fg(ratatui::style::Color::DarkGray));
    f.render_widget(footer, chunks[1]);
}
