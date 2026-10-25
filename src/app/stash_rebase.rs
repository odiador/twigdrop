use super::App;
use crate::state::ui::RebaseAction;

impl App {
    pub fn load_stashes(&mut self, path: &str) {
        self.repo.stashes = crate::git::stash::get_stashes(path);
        self.ui.selected_stash_idx = 0;
    }

    pub fn load_rebase_commits(&mut self, path: &str) {
        let commits = crate::git::get_unpushed_commits(path);
        self.ui.rebase_state.commits = commits
            .into_iter()
            .map(|c| crate::state::ui::RebaseCommit {
                hash: c.hash,
                original_message: c.message,
                new_message: None,
                action: RebaseAction::Pick,
            })
            .collect();
        self.ui.rebase_state.selected = 0;
        self.ui.rebase_state.editing = false;
        self.ui.rebase_state.input.clear();
    }

    pub fn load_rebase_commits_from_hash(&mut self, path: &str, hash: &str) {
        // Instead of getting unpushed commits, we want to get the commits from the hash to HEAD.
        let out = crate::git::commands::run_git(
            path,
            &["log", "--format=%h|%s", &format!("{hash}..HEAD")],
        )
        .unwrap_or_default();
        let commits: Vec<crate::state::ui::RebaseCommit> = out
            .lines()
            .map(|line| {
                let mut parts = line.splitn(2, '|');
                crate::state::ui::RebaseCommit {
                    hash: parts.next().unwrap_or_default().to_string(),
                    original_message: parts.next().unwrap_or_default().to_string(),
                    new_message: None,
                    action: RebaseAction::Pick,
                }
            })
            .collect();

        self.ui.rebase_state.commits = commits;
        self.ui.rebase_state.selected = 0;
        self.ui.rebase_state.editing = false;
        self.ui.rebase_state.input.clear();
    }

    pub fn load_stash_detail(&mut self, path: &str) {
        if let Some(stash) = self.repo.stashes.get(self.ui.selected_stash_idx) {
            self.repo.stash_files = crate::git::stash::get_stash_files(path, &stash.id);
            self.repo.stash_diff = crate::git::stash::get_stash_diff(path, &stash.id);
        }
    }
}
