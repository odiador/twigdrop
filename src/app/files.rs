use super::App;
use crate::git::files::FileStatus;

impl App {
    pub fn load_file_tree(&mut self, path: &str) {
        self.repo.git_file_statuses = crate::git::files::get_git_file_statuses(path);
        let mut new_tree = Vec::new();
        self.build_tree_recursive(path, "", 0, &mut new_tree);
        self.repo.file_tree = new_tree;
        self.refresh_filtered_files();
    }

    pub fn refresh_filtered_files(&mut self) {
        self.ui.filtered_file_indices = self
            .repo
            .file_tree
            .iter()
            .enumerate()
            .filter(|(_, entry)| {
                if self.ui.files_changed_only
                    && matches!(entry.status, FileStatus::Normal | FileStatus::Ignored)
                {
                    return false;
                }
                if !self.ui.files_search_query.is_empty() {
                    let path_str = entry.path.to_string_lossy().to_lowercase();
                    if !path_str.contains(&self.ui.files_search_query.to_lowercase()) {
                        return false;
                    }
                }
                true
            })
            .map(|(i, _)| i)
            .collect();

        let max = self.ui.filtered_file_indices.len().saturating_sub(1);
        if self.ui.selected_file_idx > max {
            self.ui.selected_file_idx = max;
        }
    }

    pub fn selected_file(&self) -> Option<&crate::git::files::FileEntry> {
        self.ui
            .filtered_file_indices
            .get(self.ui.selected_file_idx)
            .and_then(|&i| self.repo.file_tree.get(i))
    }

    fn build_tree_recursive(
        &self,
        root: &str,
        current_dir: &str,
        depth: usize,
        tree: &mut Vec<crate::git::files::FileEntry>,
    ) {
        let entries = crate::git::files::build_file_tree(
            root,
            current_dir,
            depth,
            &self.repo.git_file_statuses,
            self.ui.show_hidden_files,
        );

        for mut entry in entries {
            let path_str = entry.path.to_string_lossy().to_string();
            let is_open = self.ui.open_paths.contains(&path_str)
                || (self.ui.files_changed_only
                    && entry.status != FileStatus::Normal
                    && entry.status != FileStatus::Ignored);
            entry.is_open = is_open;

            tree.push(entry.clone());

            if entry.is_dir && is_open {
                self.build_tree_recursive(root, &path_str, depth + 1, tree);
            }
        }
    }

    pub fn toggle_file_dir(&mut self, path_str: &str) {
        let Some(entry) = self.selected_file().cloned() else {
            return;
        };
        if !entry.is_dir {
            return;
        }

        let path = entry.path.to_string_lossy().to_string();
        if self.ui.open_paths.contains(&path) {
            self.ui.open_paths.remove(&path);
        } else {
            self.ui.open_paths.insert(path);
        }

        let mut new_tree = Vec::new();
        self.build_tree_recursive(path_str, "", 0, &mut new_tree);
        self.repo.file_tree = new_tree;
        self.refresh_filtered_files();
    }
}
