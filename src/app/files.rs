use super::App;

impl App {
    pub fn load_file_tree(&mut self, path: &str) {
        self.repo.git_file_statuses = crate::git::files::get_git_file_statuses(path);
        let mut new_tree = Vec::new();
        self.build_tree_recursive(path, "", 0, &mut new_tree);
        self.repo.file_tree = new_tree;
        if self.ui.selected_file_idx >= self.repo.file_tree.len() {
            self.ui.selected_file_idx = self.repo.file_tree.len().saturating_sub(1);
        }
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
        );

        for mut entry in entries {
            let path_str = entry.path.to_string_lossy().to_string();
            let is_open = self.ui.open_paths.contains(&path_str);
            entry.is_open = is_open;

            tree.push(entry.clone());

            if entry.is_dir && is_open {
                self.build_tree_recursive(root, &path_str, depth + 1, tree);
            }
        }
    }

    pub fn toggle_file_dir(&mut self, path_str: &str) {
        if self.ui.selected_file_idx >= self.repo.file_tree.len() {
            return;
        }

        let entry = &self.repo.file_tree[self.ui.selected_file_idx];
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
    }
}
