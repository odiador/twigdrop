use super::App;
use crate::actions::github;

impl App {
    pub fn load_pull_requests(&mut self, path: &str) {
        let filter_arg = self.ui.pr_filter.as_gh_arg();
        match github::list_pull_requests(path, filter_arg) {
            Ok(prs) => {
                self.repo.pull_requests = prs;
                self.repo.pr_error = None;
                if self.ui.selected_pr_idx >= self.repo.pull_requests.len() {
                    self.ui.selected_pr_idx = self.repo.pull_requests.len().saturating_sub(1);
                }
                self.fetch_current_pr_diff(path);
                self.fetch_current_pr_checks(path);
            }
            Err(e) => {
                self.repo.pull_requests.clear();
                self.repo.pr_error = Some(e.to_string());
                self.repo.pr_diff = None;
                self.repo.pr_checks.clear();
            }
        }
    }

    pub fn fetch_current_pr_diff(&mut self, path: &str) {
        if let Some(pr) = self.repo.pull_requests.get(self.ui.selected_pr_idx) {
            match github::get_pull_request_diff(path, pr.number) {
                Ok(diff) => self.repo.pr_diff = Some(diff),
                Err(e) => self.repo.pr_diff = Some(format!("Error fetching diff: {e}")),
            }
        } else {
            self.repo.pr_diff = None;
        }
    }

    pub fn fetch_current_pr_checks(&mut self, path: &str) {
        if let Some(pr) = self.repo.pull_requests.get(self.ui.selected_pr_idx) {
            let pr_number = pr.number;
            if let std::collections::hash_map::Entry::Vacant(e) =
                self.repo.pr_checks.entry(pr_number)
                && let Ok(checks) = github::get_pull_request_checks(path, pr_number)
            {
                e.insert(checks);
            }
        }
    }
}
