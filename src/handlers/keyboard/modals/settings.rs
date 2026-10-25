use crate::app::App;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(app: &mut App, key: KeyEvent) -> Option<bool> {
    Some(handle_settings_keyboard(app, key))
}

pub fn handle_settings_keyboard(app: &mut App, key: KeyEvent) -> bool {
    if app.ui.settings_state.editing {
        match key.code {
            KeyCode::Enter => {
                match app.ui.settings_state.selected {
                    0 => app.config.ide_command = app.ui.settings_state.input.clone(),
                    1 => app.config.alternative_ide_command = app.ui.settings_state.input.clone(),
                    2 => app.config.ai_provider = app.ui.settings_state.input.clone(),
                    3 => {
                        app.config.current_provider_mut().model =
                            app.ui.settings_state.input.clone()
                    }
                    4 => {
                        app.config.current_provider_mut().api_key =
                            crate::utils::config::obfuscate(&app.ui.settings_state.input)
                    }
                    5 => {
                        app.config.current_provider_mut().url = app.ui.settings_state.input.clone()
                    }
                    8 => {
                        app.config.default_sidebar_width =
                            app.ui.settings_state.input.parse().unwrap_or(30)
                    }
                    _ => {}
                }
                app.ui.settings_state.editing = false;
                crate::utils::config::save_config(&app.config);
            }
            KeyCode::Esc => {
                app.ui.settings_state.editing = false;
            }
            KeyCode::Char(c) => {
                app.ui.settings_state.input.push(c);
            }
            KeyCode::Backspace => {
                app.ui.settings_state.input.pop();
            }
            _ => {}
        }
        return false;
    }

    if app.ui.settings_state.selecting {
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if app.ui.settings_state.choice_idx > 0 {
                    app.ui.settings_state.choice_idx -= 1;
                } else {
                    app.ui.settings_state.choice_idx =
                        app.ui.settings_state.choices.len().saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if app.ui.settings_state.choice_idx + 1 < app.ui.settings_state.choices.len() {
                    app.ui.settings_state.choice_idx += 1;
                } else {
                    app.ui.settings_state.choice_idx = 0;
                }
            }
            KeyCode::Enter => {
                if let Some(choice) = app
                    .ui
                    .settings_state
                    .choices
                    .get(app.ui.settings_state.choice_idx)
                {
                    match app.ui.settings_state.selected {
                        2 => app.config.ai_provider = choice.clone(),
                        3 => app.config.current_provider_mut().model = choice.clone(),
                        _ => {}
                    }
                }
                app.ui.settings_state.selecting = false;
                crate::utils::config::save_config(&app.config);
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                app.ui.settings_state.selecting = false;
            }
            _ => {}
        }
        return false;
    }

    match key.code {
        KeyCode::Up | KeyCode::Char('k') if app.ui.settings_state.selected > 0 => {
            app.ui.settings_state.selected -= 1;
        }
        KeyCode::Down | KeyCode::Char('j') if app.ui.settings_state.selected < 9 => {
            app.ui.settings_state.selected += 1;
        }
        KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') => {
            match app.ui.settings_state.selected {
                2 => {
                    app.ui.settings_state.selecting = true;
                    app.ui.settings_state.choices = crate::utils::config::PROVIDER_ARCHETYPES
                        .iter()
                        .map(|s| s.to_string())
                        .collect();
                    app.ui.settings_state.choice_idx = app
                        .ui
                        .settings_state
                        .choices
                        .iter()
                        .position(|s| s == &app.config.ai_provider)
                        .unwrap_or(0);
                }
                3 => {
                    app.ui.settings_state.selecting = true;
                    app.ui.settings_state.choices = match app.config.ai_provider.as_str() {
                        "openai" => crate::utils::config::OPENAI_MODELS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "anthropic" => crate::utils::config::ANTHROPIC_MODELS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        "google" => crate::utils::config::GOOGLE_MODELS
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                        _ => vec!["Loading models...".to_string()],
                    };
                    app.ui.settings_state.choice_idx = app
                        .ui
                        .settings_state
                        .choices
                        .iter()
                        .position(|s| s == &app.config.current_provider().model)
                        .unwrap_or(0);
                }
                6 => {
                    app.config.enable_animations = !app.config.enable_animations;
                    crate::utils::config::save_config(&app.config);
                }
                7 => {
                    use crate::ui::theme::ThemeMode;
                    app.config.theme_mode = match app.config.theme_mode {
                        ThemeMode::Dark => ThemeMode::Light,
                        ThemeMode::Light => ThemeMode::Dark,
                    };
                    crate::utils::config::save_config(&app.config);
                }
                9 => {
                    crate::utils::config::save_config(&app.config);
                    app.ui.pop_modal();
                }
                _ => {
                    app.ui.settings_state.editing = true;
                    app.ui.settings_state.input = match app.ui.settings_state.selected {
                        0 => app.config.ide_command.clone(),
                        1 => app.config.alternative_ide_command.clone(),
                        4 => crate::utils::config::deobfuscate(
                            &app.config.current_provider().api_key,
                        ),
                        5 => app.config.current_provider().url.clone(),
                        8 => app.config.default_sidebar_width.to_string(),
                        _ => String::new(),
                    };
                }
            }
        }
        KeyCode::Char('q') | KeyCode::Esc | KeyCode::BackTab => {
            app.ui.pop_modal();
        }
        _ => {}
    }
    false
}
