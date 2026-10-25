use crate::actions::github::{self, ReviewAction};
use crate::app::App;
use crate::state::ui::{AppMode, PrReviewKind, PrReviewModalState};
use ratatui::crossterm::event::{KeyCode, KeyEvent};

pub fn handle(
    app: &mut App,
    key: KeyEvent,
    mut state: PrReviewModalState,
    path: &str,
) -> Option<bool> {
    match key.code {
        KeyCode::Esc => {
            app.ui.pop_modal();
            Some(false)
        }
        KeyCode::Backspace => {
            state.body.pop();
            if let AppMode::PrReview(s) = app.ui.current_mode_mut() {
                *s = state;
            }
            Some(false)
        }
        KeyCode::Char(c) => {
            state.body.push(c);
            if let AppMode::PrReview(s) = app.ui.current_mode_mut() {
                *s = state;
            }
            Some(false)
        }
        KeyCode::Enter => {
            let pr_number = state.pr_number;
            let body = state.body.trim().to_string();

            match state.kind {
                PrReviewKind::Approve => {
                    let body_opt = if body.is_empty() {
                        None
                    } else {
                        Some(body.as_str())
                    };
                    match github::review_pull_request(
                        path,
                        pr_number,
                        ReviewAction::Approve,
                        body_opt,
                    ) {
                        Ok(_) => {
                            app.ui.pop_modal();
                            app.load_pull_requests(path);
                            app.ui.notify(
                                crate::state::toast::ToastKind::Success,
                                format!("✓ Approved PR #{pr_number}"),
                                None,
                            );
                        }
                        Err(e) => {
                            app.ui.notify(
                                crate::state::toast::ToastKind::Error,
                                format!("Approval failed: {e}"),
                                None,
                            );
                        }
                    }
                }
                PrReviewKind::RequestChanges => {
                    if body.is_empty() {
                        app.ui.notify(
                            crate::state::toast::ToastKind::Error,
                            "Please provide a reason for requesting changes".to_string(),
                            None,
                        );
                        return Some(false);
                    }
                    match github::review_pull_request(
                        path,
                        pr_number,
                        ReviewAction::RequestChanges,
                        Some(&body),
                    ) {
                        Ok(_) => {
                            app.ui.pop_modal();
                            app.load_pull_requests(path);
                            app.ui.notify(
                                crate::state::toast::ToastKind::Info,
                                format!("Requested changes on PR #{pr_number}"),
                                None,
                            );
                        }
                        Err(e) => {
                            app.ui.notify(
                                crate::state::toast::ToastKind::Error,
                                format!("Request changes failed: {e}"),
                                None,
                            );
                        }
                    }
                }
                PrReviewKind::Comment => {
                    if body.is_empty() {
                        app.ui.notify(
                            crate::state::toast::ToastKind::Error,
                            "Please provide a comment message".to_string(),
                            None,
                        );
                        return Some(false);
                    }
                    match github::comment_pull_request(path, pr_number, &body) {
                        Ok(_) => {
                            app.ui.pop_modal();
                            app.load_pull_requests(path);
                            app.ui.notify(
                                crate::state::toast::ToastKind::Success,
                                format!("Comment posted on PR #{pr_number}"),
                                None,
                            );
                        }
                        Err(e) => {
                            app.ui.notify(
                                crate::state::toast::ToastKind::Error,
                                format!("Comment failed: {e}"),
                                None,
                            );
                        }
                    }
                }
            }
            Some(false)
        }
        _ => Some(false),
    }
}
