use super::App;
use crate::models::Branch;
use crate::state::ui::PrimaryMode;

/// Number of recent commits shown in the inspector panel.
pub use crate::constants::DEFAULT_INSPECTOR_COMMIT_LIMIT as INSPECTOR_COMMIT_LIMIT;

impl App {
    pub fn refresh_branches(&mut self, path: &str) {
        self.repo.branches = crate::git::build_branches(path);
        self.repo.head_commit = crate::git::get_branch_commits(path, &self.repo.current_branch, 1)
            .into_iter()
            .next();
        self.repo.git_file_statuses = crate::git::files::get_git_file_statuses(path);
        // The analyzer re-checks every branch on trigger; show that in the UI.
        if !self.trigger_tx.is_closed() {
            for b in &mut self.repo.branches {
                b.merge_status = crate::models::MergeStatus::Checking;
            }
        }
        self.refresh_filtered_branches();
        let _ = self.trigger_tx.try_send(());
    }

    pub fn cycle_branch_sort(&mut self) {
        self.ui.branch_sort_mode = match self.ui.branch_sort_mode {
            crate::state::ui::BranchSortMode::Recent => {
                crate::state::ui::BranchSortMode::PrunableFirst
            }
            crate::state::ui::BranchSortMode::PrunableFirst => {
                crate::state::ui::BranchSortMode::Alphabetical
            }
            crate::state::ui::BranchSortMode::Alphabetical => {
                crate::state::ui::BranchSortMode::Recent
            }
        };
        self.refresh_filtered_branches();
    }

    pub fn refresh_filtered_branches(&mut self) {
        let mut indices: Vec<usize> = self
            .repo
            .branches
            .iter()
            .enumerate()
            .filter(|(_, b)| {
                let status_match = if let Some(filter) = &self.ui.current_filter {
                    b.status.contains(filter)
                } else {
                    true
                };

                let search_match = if !self.ui.search_query.is_empty() {
                    b.name
                        .to_lowercase()
                        .contains(&self.ui.search_query.to_lowercase())
                } else {
                    true
                };

                let active_match = if self.ui.branches_active_only {
                    b.name == self.repo.current_branch
                        || b.ahead_count > 0
                        || b.behind_count > 0
                        || b.status
                            .contains(&crate::models::BranchStatus::HasUniqueCommits)
                        || b.status.contains(&crate::models::BranchStatus::Ahead)
                } else {
                    true
                };

                status_match && search_match && active_match
            })
            .map(|(i, _)| i)
            .collect();

        let current_branch = &self.repo.current_branch;
        let sort_mode = self.ui.branch_sort_mode;
        let branches = &self.repo.branches;

        indices.sort_by(|&a_idx, &b_idx| {
            let a = &branches[a_idx];
            let b = &branches[b_idx];

            let a_is_pseudo = a.name.starts_with('*');
            let b_is_pseudo = b.name.starts_with('*');
            if a_is_pseudo != b_is_pseudo {
                return b_is_pseudo.cmp(&a_is_pseudo);
            }
            if a_is_pseudo && b_is_pseudo {
                return a_idx.cmp(&b_idx);
            }

            let a_is_current = a.name == *current_branch;
            let b_is_current = b.name == *current_branch;
            if a_is_current != b_is_current {
                return b_is_current.cmp(&a_is_current);
            }

            match sort_mode {
                crate::state::ui::BranchSortMode::Recent => b
                    .commit_date
                    .cmp(&a.commit_date)
                    .then_with(|| a.name.cmp(&b.name)),
                crate::state::ui::BranchSortMode::PrunableFirst => {
                    let a_prunable = a.status.contains(&crate::models::BranchStatus::Gone)
                        && !self.config.is_protected(&a.name);
                    let b_prunable = b.status.contains(&crate::models::BranchStatus::Gone)
                        && !self.config.is_protected(&b.name);
                    b_prunable
                        .cmp(&a_prunable)
                        .then_with(|| b.commit_date.cmp(&a.commit_date))
                }
                crate::state::ui::BranchSortMode::Alphabetical => {
                    a.name.to_lowercase().cmp(&b.name.to_lowercase())
                }
            }
        });

        self.ui.filtered_indices = indices;

        let max = self.ui.filtered_indices.len().saturating_sub(1);
        if self.ui.selected_branch_idx > max {
            self.ui.selected_branch_idx = max;
        }
    }

    pub fn get_filtered_branches(&self) -> Vec<&Branch> {
        self.ui
            .filtered_indices
            .iter()
            .map(|&i| &self.repo.branches[i])
            .collect()
    }

    pub fn fetch_current_commit_details(&mut self, path: &str) {
        if let Some(commit) = self.repo.commit_tree.get(self.ui.selected_commit_idx) {
            let (stats, diff) = crate::git::commands::get_commit_details(path, &commit.hash);
            self.repo.commit_stats = Some(stats);
            self.repo.commit_diff = Some(diff);
            self.repo.commit_files = crate::git::files::get_commit_files(path, &commit.hash);
            if self.ui.selected_commit_file_idx >= self.repo.commit_files.len() {
                self.ui.selected_commit_file_idx = 0;
            }
            self.fetch_selected_commit_file_diff(path);
        } else {
            self.repo.commit_stats = None;
            self.repo.commit_diff = None;
            self.repo.commit_files.clear();
            self.repo.commit_file_diff = None;
        }
    }

    pub fn fetch_selected_commit_file_diff(&mut self, path: &str) {
        if let Some(commit) = self.repo.commit_tree.get(self.ui.selected_commit_idx) {
            if let Some(entry) = self.repo.commit_files.get(self.ui.selected_commit_file_idx) {
                let file_path = entry.path.to_string_lossy().replace('\\', "/");
                match crate::actions::commands::get_commit_file_diff(path, &commit.hash, &file_path)
                {
                    Ok(diff) => self.repo.commit_file_diff = Some(diff),
                    Err(_) => self.repo.commit_file_diff = None,
                }
            } else {
                self.repo.commit_file_diff = None;
            }
        } else {
            self.repo.commit_file_diff = None;
        }
    }

    /// Inspector cache refresh (update phase only — never call from render).
    /// Re-runs `git log` only when the selected branch changed; also fixes
    /// the legacy hardcoded `"."` repo path. Called at `draw` entry for the
    /// Branches view so every selection path (keyboard/mouse/filter/sort)
    /// stays covered without per-frame subprocesses.
    pub fn refresh_inspector_commits(&mut self, path: &str) {
        let branch = self
            .ui
            .filtered_indices
            .get(self.ui.selected_branch_idx)
            .and_then(|&i| self.repo.branches.get(i))
            .map(|b| b.name.clone())
            .unwrap_or_default();
        if branch.is_empty() || branch.starts_with('*') {
            self.repo.inspector_branch = branch;
            self.repo.inspector_commits.clear();
            return;
        }
        if branch == self.repo.inspector_branch {
            return;
        }
        self.repo.inspector_commits =
            crate::git::get_branch_commits(path, &branch, INSPECTOR_COMMIT_LIMIT)
                .into_iter()
                .map(|c| format!("{} {}", c.hash, c.message))
                .collect();
        self.repo.inspector_branch = branch;
    }

    /// Sidebar worktree cache refresh (update phase only — never in render).
    /// Also fixes the legacy hardcoded `"."` repo path. Cheap enough to run
    /// whenever the sidebar is visible; skips when hidden.
    pub fn refresh_sidebar_worktrees(&mut self, path: &str) {
        let out = crate::actions::commands::list_worktrees(path).unwrap_or_default();
        let mut worktrees = Vec::new();
        let mut current_wt = String::new();
        for line in out.lines() {
            if let Some(wt) = line.strip_prefix("worktree ") {
                current_wt = std::path::Path::new(wt)
                    .file_name()
                    .map(|s| s.to_string_lossy().to_string())
                    .unwrap_or_else(|| wt.to_string());
            } else if let Some(branch) = line.strip_prefix("branch refs/heads/") {
                worktrees.push((current_wt.clone(), branch.to_string()));
            }
        }
        self.repo.worktrees = worktrees;
    }

    pub fn next(&mut self, path: &str) {
        match self.ui.primary_mode {
            PrimaryMode::Branches => {
                let max = self.ui.filtered_indices.len().saturating_sub(1);
                if self.ui.selected_branch_idx < max {
                    self.ui.selected_branch_idx += 1;
                }
            }
            PrimaryMode::Files => {
                let max = self.ui.filtered_file_indices.len().saturating_sub(1);
                if self.ui.selected_file_idx < max {
                    self.ui.selected_file_idx += 1;
                }
            }
            PrimaryMode::Commits => {
                if self.ui.commit_panel == crate::state::ui::CommitPanel::Files {
                    if self.ui.selected_commit_file_idx + 1 < self.repo.commit_files.len() {
                        self.ui.selected_commit_file_idx += 1;
                        self.ui.info_scroll = 0;
                        self.fetch_selected_commit_file_diff(path);
                    }
                } else if self.ui.selected_commit_idx
                    < self.repo.commit_tree.len().saturating_sub(1)
                {
                    let mut next_idx = self.ui.selected_commit_idx + 1;
                    while next_idx < self.repo.commit_tree.len()
                        && self.repo.commit_tree[next_idx].hash.is_empty()
                    {
                        next_idx += 1;
                    }
                    if next_idx < self.repo.commit_tree.len() {
                        self.ui.selected_commit_idx = next_idx;
                        self.ui.info_scroll = 0;
                        self.fetch_current_commit_details(path);
                    }
                }
            }
            PrimaryMode::Stashes => {
                if self.ui.selected_stash_idx < self.repo.stashes.len().saturating_sub(1) {
                    self.ui.selected_stash_idx += 1;
                }
            }
            PrimaryMode::PullRequests => {
                let max = self.repo.pull_requests.len().saturating_sub(1);
                if self.ui.selected_pr_idx < max {
                    self.ui.selected_pr_idx += 1;
                    self.ui.pr_scroll_y = 0;
                    self.fetch_current_pr_diff(path);
                }
            }
        }
    }

    pub fn previous(&mut self, path: &str) {
        match self.ui.primary_mode {
            PrimaryMode::Branches => {
                if self.ui.selected_branch_idx > 0 {
                    self.ui.selected_branch_idx -= 1;
                }
            }
            PrimaryMode::Files => {
                if self.ui.selected_file_idx > 0 {
                    self.ui.selected_file_idx -= 1;
                }
            }
            PrimaryMode::Commits => {
                if self.ui.commit_panel == crate::state::ui::CommitPanel::Files {
                    if self.ui.selected_commit_file_idx > 0 {
                        self.ui.selected_commit_file_idx -= 1;
                        self.ui.info_scroll = 0;
                        self.fetch_selected_commit_file_diff(path);
                    }
                } else if self.ui.selected_commit_idx > 0 {
                    let mut prev_idx = self.ui.selected_commit_idx - 1;
                    while prev_idx > 0 && self.repo.commit_tree[prev_idx].hash.is_empty() {
                        prev_idx -= 1;
                    }
                    if !self.repo.commit_tree[prev_idx].hash.is_empty() {
                        self.ui.selected_commit_idx = prev_idx;
                        self.ui.info_scroll = 0;
                        self.fetch_current_commit_details(path);
                    }
                }
            }
            PrimaryMode::Stashes => {
                if self.ui.selected_stash_idx > 0 {
                    self.ui.selected_stash_idx -= 1;
                }
            }
            PrimaryMode::PullRequests => {
                if self.ui.selected_pr_idx > 0 {
                    self.ui.selected_pr_idx -= 1;
                    self.ui.pr_scroll_y = 0;
                    self.fetch_current_pr_diff(path);
                }
            }
        }
    }

    pub fn toggle_selection(&mut self) {
        if let Some(&idx) = self.ui.filtered_indices.get(self.ui.selected_branch_idx) {
            let name = self.repo.branches[idx].name.clone();
            if self.config.is_protected(&name) {
                return;
            }
            if self.ui.bulk_selected.contains(&name) {
                self.ui.bulk_selected.remove(&name);
            } else {
                self.ui.bulk_selected.insert(name);
            }
        }
    }
}
