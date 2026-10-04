pub mod en;
pub mod es;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    En,
    Es,
}

impl Language {
    pub fn name(&self) -> &'static str {
        match self {
            Language::En => "English",
            Language::Es => "Español",
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Language::En => "en",
            Language::Es => "es",
        }
    }
}

pub struct CommonStrings {
    pub ok: &'static str,
    pub cancel: &'static str,
    pub yes: &'static str,
    pub no: &'static str,
    pub close: &'static str,
    pub select: &'static str,
    pub execute: &'static str,
    pub none: &'static str,
    pub error: &'static str,
    pub warning: &'static str,
    pub quit: &'static str,
    pub move_keys: &'static str,
}

pub struct BranchStrings {
    pub header_sel: &'static str,
    pub header_branch: &'static str,
    pub header_sync: &'static str,
    pub header_merge_health: &'static str,
    pub header_age: &'static str,
    pub header_last_commit: &'static str,
    pub title_branches: &'static str,
    pub type_to_filter: &'static str,
    pub esc_to_clear: &'static str,
    pub sort_label: &'static str,
    pub sort_recent: &'static str,
    pub sort_prunable: &'static str,
    pub sort_alphabetical: &'static str,
    pub inspector_hint: &'static str,
    pub sidebar_hint: &'static str,
    pub sync_synced: &'static str,
    pub sync_local: &'static str,
    pub merge_clean: &'static str,
    pub merge_checking: &'static str,
    pub merge_conflict: &'static str,
    pub merge_not_analyzed: &'static str,
    pub legend_unique_commits: &'static str,
    pub legend_gone: &'static str,
    pub legend_ahead_behind: &'static str,
    pub legend_merged: &'static str,
    pub legend_protected: &'static str,
    pub manage_title: &'static str,
    pub manage_target_prefix: &'static str,
    pub manage_instruction: &'static str,
    pub act_checkout: &'static str,
    pub act_checkout_desc: &'static str,
    pub act_diff_ai: &'static str,
    pub act_diff_ai_desc: &'static str,
    pub act_delete: &'static str,
    pub act_delete_desc: &'static str,
    pub act_rename: &'static str,
    pub act_rename_desc: &'static str,
    pub act_stash: &'static str,
    pub act_stash_desc: &'static str,
    pub act_help: &'static str,
    pub act_help_desc: &'static str,
    pub act_cancel: &'static str,
    pub act_cancel_desc: &'static str,
    pub confirm_unpushed_title: &'static str,
    pub confirm_unique_msg: &'static str,
    pub confirm_data_loss: &'static str,
    pub confirm_prompt: &'static str,
}

pub struct ModalStrings {
    pub main_menu_quick_actions: &'static str,
    pub main_menu_command_palette: &'static str,
    pub main_menu_app_switcher: &'static str,
    pub main_menu_settings: &'static str,
    pub main_menu_help: &'static str,
    pub main_menu_quit: &'static str,
    pub command_palette_title: &'static str,
    pub command_palette_placeholder: &'static str,
    pub cmd_checkout: &'static str,
    pub cmd_create_branch: &'static str,
    pub cmd_settings: &'static str,
    pub cmd_rebase: &'static str,
    pub cmd_stash: &'static str,
    pub cmd_pop_stash: &'static str,
    pub cmd_fetch: &'static str,
    pub cmd_pull: &'static str,
    pub cmd_push: &'static str,
    pub cmd_commit: &'static str,
    pub help_branch_ops_title: &'static str,
    pub help_status_badges_title: &'static str,
    pub help_global_shortcuts_title: &'static str,
    pub help_files_shortcuts_title: &'static str,
    pub help_footer_close: &'static str,
    pub help_footer_credit: &'static str,
}

pub struct FileStrings {
    pub stage_unstage: &'static str,
    pub open_explorer: &'static str,
    pub open_ide: &'static str,
    pub resize_sidebar: &'static str,
}

pub struct FooterStrings {
    pub branches_normal: &'static str,
    pub branches_shift: &'static str,
    pub branches_alt: &'static str,
    pub files_normal: &'static str,
    pub commits_normal: &'static str,
    pub stashes_normal: &'static str,
    pub status_tag: &'static str,
}

pub struct Locale {
    pub common: CommonStrings,
    pub branches: BranchStrings,
    pub modals: ModalStrings,
    pub files: FileStrings,
    pub footer: FooterStrings,
}

impl Locale {
    pub fn format_range(&self, start: usize, end: usize, total: usize) -> String {
        format!(" {}-{} of {} ", start, end, total)
    }

    pub fn format_branches_count(&self, count: usize) -> String {
        format!("{} branches", count)
    }
}

pub fn get_locale(lang: Language) -> &'static Locale {
    match lang {
        Language::En => &en::EN,
        Language::Es => &es::ES,
    }
}

pub fn detect_system_language() -> Language {
    if let Ok(lang) = std::env::var("LANG").or_else(|_| std::env::var("LC_ALL")) {
        let l = lang.to_lowercase();
        if l.starts_with("es") {
            return Language::Es;
        }
    }
    Language::En
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_name_and_code() {
        assert_eq!(Language::En.name(), "English");
        assert_eq!(Language::En.code(), "en");
        assert_eq!(Language::Es.name(), "Español");
        assert_eq!(Language::Es.code(), "es");
    }

    #[test]
    fn test_i18n_catalogs_non_empty() {
        for lang in [Language::En, Language::Es] {
            let loc = get_locale(lang);
            // Common
            assert!(!loc.common.ok.is_empty());
            assert!(!loc.common.cancel.is_empty());
            assert!(!loc.common.yes.is_empty());
            assert!(!loc.common.no.is_empty());
            assert!(!loc.common.close.is_empty());
            assert!(!loc.common.quit.is_empty());

            // Branches
            assert!(!loc.branches.header_branch.is_empty());
            assert!(!loc.branches.title_branches.is_empty());
            assert!(!loc.branches.type_to_filter.is_empty());
            assert!(!loc.branches.act_checkout.is_empty());
            assert!(!loc.branches.confirm_unpushed_title.is_empty());

            // Modals
            assert!(!loc.modals.command_palette_title.is_empty());
            assert!(!loc.modals.help_branch_ops_title.is_empty());
            assert!(!loc.modals.cmd_checkout.is_empty());

            // Footer
            assert!(!loc.footer.branches_normal.is_empty());
            assert!(!loc.footer.status_tag.is_empty());

            // Formatting
            assert_eq!(loc.format_range(1, 10, 50), " 1-10 of 50 ");
            assert_eq!(loc.format_branches_count(5), "5 branches");
        }
    }
}
