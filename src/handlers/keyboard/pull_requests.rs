use crate::app::App;
use crate::state::ui::PrTab;
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle_pull_requests_keyboard(app: &mut App, key: KeyEvent, path: &str) -> bool {
    match key.code {
        KeyCode::Char('j') | KeyCode::Down => {
            if app.ui.selected_pr_idx < app.repo.pull_requests.len().saturating_sub(1) {
                app.ui.selected_pr_idx += 1;
                app.ui.pr_scroll_y = 0;
                app.fetch_current_pr_diff(path);
            }
            false
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.ui.selected_pr_idx > 0 {
                app.ui.selected_pr_idx -= 1;
                app.ui.pr_scroll_y = 0;
                app.fetch_current_pr_diff(path);
            }
            false
        }
        KeyCode::Tab => {
            app.ui.pr_tab = match app.ui.pr_tab {
                PrTab::Description => PrTab::Diff,
                PrTab::Diff => PrTab::Checks,
                PrTab::Checks => PrTab::Description,
            };
            app.ui.pr_scroll_y = 0;
            if app.ui.pr_tab == PrTab::Diff && app.repo.pr_diff.is_none() {
                app.fetch_current_pr_diff(path);
            } else if app.ui.pr_tab == PrTab::Checks {
                app.fetch_current_pr_checks(path);
            }
            false
        }
        KeyCode::Char('1') => {
            app.ui.pr_tab = PrTab::Description;
            app.ui.pr_scroll_y = 0;
            false
        }
        KeyCode::Char('2') => {
            app.ui.pr_tab = PrTab::Diff;
            app.ui.pr_scroll_y = 0;
            if app.repo.pr_diff.is_none() {
                app.fetch_current_pr_diff(path);
            }
            false
        }
        KeyCode::Char('3') => {
            app.ui.pr_tab = PrTab::Checks;
            app.ui.pr_scroll_y = 0;
            app.fetch_current_pr_checks(path);
            false
        }
        KeyCode::Char('a') => {
            if let Some(pr) = app.repo.pull_requests.get(app.ui.selected_pr_idx) {
                app.ui.push_modal(crate::state::ui::AppMode::PrReview(
                    crate::state::ui::PrReviewModalState {
                        pr_number: pr.number,
                        pr_title: pr.title.clone(),
                        kind: crate::state::ui::PrReviewKind::Approve,
                        body: String::new(),
                    },
                ));
            }
            false
        }
        KeyCode::Char('x') => {
            if let Some(pr) = app.repo.pull_requests.get(app.ui.selected_pr_idx) {
                app.ui.push_modal(crate::state::ui::AppMode::PrReview(
                    crate::state::ui::PrReviewModalState {
                        pr_number: pr.number,
                        pr_title: pr.title.clone(),
                        kind: crate::state::ui::PrReviewKind::RequestChanges,
                        body: String::new(),
                    },
                ));
            }
            false
        }
        KeyCode::Char('m') => {
            if let Some(pr) = app.repo.pull_requests.get(app.ui.selected_pr_idx) {
                app.ui.push_modal(crate::state::ui::AppMode::PrReview(
                    crate::state::ui::PrReviewModalState {
                        pr_number: pr.number,
                        pr_title: pr.title.clone(),
                        kind: crate::state::ui::PrReviewKind::Comment,
                        body: String::new(),
                    },
                ));
            }
            false
        }
        KeyCode::Char('f') => {
            app.ui.pr_filter = app.ui.pr_filter.next();
            app.load_pull_requests(path);
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                format!("Filter: {} PRs", app.ui.pr_filter.label()),
                None,
            );
            false
        }
        KeyCode::Char('r') => {
            app.load_pull_requests(path);
            app.ui.notify(
                crate::state::toast::ToastKind::Info,
                "Reloaded pull requests".to_string(),
                None,
            );
            false
        }
        KeyCode::Char('c') => {
            if let Some(pr) = app.repo.pull_requests.get(app.ui.selected_pr_idx) {
                let pr_num = pr.number;
                let head_branch = pr.head_ref_name.clone();
                match crate::actions::github::checkout_pull_request(path, pr_num) {
                    Ok(_) => {
                        app.refresh_branches(path);
                        app.ui.notify(
                            crate::state::toast::ToastKind::Info,
                            format!("Checked out PR #{pr_num}: {head_branch}"),
                            None,
                        );
                    }
                    Err(e) => {
                        app.ui.notify(
                            crate::state::toast::ToastKind::Error,
                            format!("Checkout failed: {e}"),
                            None,
                        );
                    }
                }
            }
            false
        }
        KeyCode::Char('o') => {
            if let Some(pr) = app.repo.pull_requests.get(app.ui.selected_pr_idx) {
                let pr_num = pr.number;
                match crate::actions::github::open_pull_request_web(path, pr_num) {
                    Ok(()) => {
                        app.ui.notify(
                            crate::state::toast::ToastKind::Info,
                            format!("Opened PR #{pr_num} in browser"),
                            None,
                        );
                    }
                    Err(e) => {
                        app.ui.notify(
                            crate::state::toast::ToastKind::Error,
                            format!("Failed to open browser: {e}"),
                            None,
                        );
                    }
                }
            }
            false
        }
        KeyCode::Char('d') => {
            app.ui.pr_scroll_y = app.ui.pr_scroll_y.saturating_add(5);
            false
        }
        KeyCode::Char('u') => {
            app.ui.pr_scroll_y = app.ui.pr_scroll_y.saturating_sub(5);
            false
        }
        KeyCode::PageDown => {
            app.ui.pr_scroll_y = app.ui.pr_scroll_y.saturating_add(15);
            false
        }
        KeyCode::PageUp => {
            app.ui.pr_scroll_y = app.ui.pr_scroll_y.saturating_sub(15);
            false
        }
        KeyCode::Esc => {
            app.set_primary_mode(crate::state::ui::PrimaryMode::Branches);
            app.ui.track_history(crate::state::ui::View::Branches);
            false
        }
        _ => false,
    }
}
