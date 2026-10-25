use crate::app::App;
use crate::state::ui::{AppMode, FolderProfileState};
use crate::utils::profiles::{audit_workspace, save_profile, scaffold_missing};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(
    app: &mut App,
    mut state: FolderProfileState,
    key: KeyEvent,
    path: &str,
) -> Option<bool> {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') if state.selected > 0 => {
            state.selected -= 1;
        }
        KeyCode::Down | KeyCode::Char('j')
            if state.selected < state.audit_report.items.len().saturating_sub(1) =>
        {
            state.selected += 1;
        }
        KeyCode::Char('s') => {
            // Scaffold missing directories
            match scaffold_missing(path, &state.audit_report) {
                Ok(created) => {
                    let count = created.len();
                    app.load_file_tree(path);
                    // Re-audit
                    let current_prof = &state.available_profiles[state.profile_idx];
                    state.audit_report = audit_workspace(path, current_prof);
                    state.selected = 0;
                    app.ui.notify(
                        crate::state::toast::ToastKind::Success,
                        format!("Scaffolded {count} missing directories"),
                        None,
                    );
                }
                Err(e) => {
                    app.ui.notify(
                        crate::state::toast::ToastKind::Error,
                        format!("Scaffolding failed: {e}"),
                        None,
                    );
                }
            }
        }
        KeyCode::Char('w') => {
            // Save current workspace as custom profile
            let custom_prof =
                crate::utils::profiles::FolderProfile::from_workspace(path, "Project Custom");
            match save_profile(path, &custom_prof) {
                Ok(saved_path) => {
                    state.available_profiles = crate::utils::profiles::load_profiles(path);
                    if let Some(pos) = state
                        .available_profiles
                        .iter()
                        .position(|p| p.name == custom_prof.name)
                    {
                        state.profile_idx = pos;
                        state.audit_report = audit_workspace(path, &state.available_profiles[pos]);
                    }
                    app.ui.notify(
                        crate::state::toast::ToastKind::Success,
                        format!("Saved profile to {}", saved_path.display()),
                        None,
                    );
                }
                Err(e) => {
                    app.ui.notify(
                        crate::state::toast::ToastKind::Error,
                        format!("Save profile failed: {e}"),
                        None,
                    );
                }
            }
        }
        KeyCode::Char('p') if !state.available_profiles.is_empty() => {
            // Cycle presets
            state.profile_idx = (state.profile_idx + 1) % state.available_profiles.len();
            let next_prof = &state.available_profiles[state.profile_idx];
            state.audit_report = audit_workspace(path, next_prof);
            state.selected = 0;
            let prof_name = state.audit_report.profile_name.clone();
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                format!("Switched profile: {prof_name}"),
                None,
            );
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.ui.pop_modal();
            return Some(false);
        }
        _ => {}
    }

    if let AppMode::FolderProfile(s) = app.ui.current_mode_mut() {
        *s = state;
    }
    Some(false)
}
