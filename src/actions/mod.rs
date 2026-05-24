pub mod commands;

pub use commands::{
    apply_stash, bulk_delete_branches, prune_branches,
};
#[allow(unused_imports)]
pub use commands::apply_resolution_to_file;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ActionId {
    DeleteBranch,
    CheckoutBranch,
    MergeBranch,
    StashChanges,
    FetchAll,
    Pull,
    Push,
    CommitAmend,
    InteractiveRebase,
    TogglePrimaryMode,
    ToggleHelp,
    Quit,
}

