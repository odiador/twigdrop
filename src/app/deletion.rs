use super::App;

impl App {
    /// Snapshot pre-delete SHAs for undo (skips protected/unresolvable).
    fn snapshot_shas(&self, path: &str, names: &[String]) -> Vec<(String, String)> {
        names
            .iter()
            .filter(|n| !self.config.is_protected(n))
            .filter_map(|n| {
                crate::actions::commands::resolve_branch_sha(path, n).map(|sha| (n.clone(), sha))
            })
            .collect()
    }

    /// Delete an explicit name list (snap-confirmed flow): backups run
    /// inside `bulk_delete_branches`; survivors feed the undo toast.
    /// Fixes the legacy path that deleted the bulk set instead of the
    /// confirmed names.
    pub fn delete_branches_by_name(&mut self, path: &str, names: &[String]) {
        if names.is_empty() {
            self.refresh_branches(path);
            return;
        }
        let undo_items = self.snapshot_shas(path, names);
        let extra = self.config.protected_branches.clone();
        crate::actions::bulk_delete_branches(path, names, &extra);
        for name in names {
            self.ui.bulk_selected.remove(name);
        }
        self.refresh_branches(path);
        let loc = self.locale();
        let undo = if undo_items.is_empty() {
            None
        } else {
            Some(crate::state::toast::UndoRestore {
                branches: undo_items,
            })
        };
        self.ui.notify(
            crate::state::toast::ToastKind::Success,
            format!("{} {}", names.len(), loc.branches.toast_deleted),
            undo,
        );
    }

    /// Snap-capture completion: deletes the animation's confirmed rows
    /// (not the bulk set) and arms the undo toast.
    pub fn complete_snap_deletion(&mut self, path: &str) {
        let names: Vec<String> = self
            .ui
            .snap_animation
            .as_ref()
            .map(|anim| {
                anim.rows
                    .iter()
                    .map(|row| row.branch_name.clone())
                    .collect()
            })
            .unwrap_or_default();
        self.delete_branches_by_name(path, &names);
    }

    /// Recreate branches from the toast's undo data (`u` binding).
    /// Already-existing names are skipped (reported, not an error).
    pub fn undo_delete(&mut self, path: &str) {
        let Some(restore) = self.ui.undoable_toast().and_then(|t| t.undo.clone()) else {
            return;
        };
        let mut restored = 0;
        for (name, sha) in &restore.branches {
            if crate::git::commands::run_git(path, &["branch", name, sha]).is_ok() {
                restored += 1;
            }
        }
        self.refresh_branches(path);
        let loc = self.locale();
        let text = if restored > 0 {
            format!("{} {}", restored, loc.branches.toast_restored)
        } else {
            loc.branches.toast_restore_empty.to_string()
        };
        self.ui
            .notify(crate::state::toast::ToastKind::Success, text, None);
    }
}
