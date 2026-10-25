use crate::state::ui::AppMode;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum DatePickerField {
    Year,
    Month,
    Day,
    Hour,
    Minute,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct DatePickerState {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub active_field: DatePickerField,
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CommandAction {
    CheckoutBranch,
    CreateBranch,
    OpenSettings,
    InteractiveRebase,
    StashChanges,
    PopStash,
    Fetch,
    Pull,
    Push,
    CommitChanges,
    ContinueRebase,
    AbortRebase,
    AbortMerge,
}

#[derive(Default, PartialEq, Eq, Debug, Clone)]
pub struct CommandPaletteState {
    pub query: String,
    pub selected: usize,
    pub actions: Vec<(String, CommandAction)>,
}

#[derive(Default, Debug)]
pub struct SettingsState {
    pub selected: usize,
    pub editing: bool,
    pub selecting: bool,
    pub choice_idx: usize,
    pub choices: Vec<String>,
    pub input: String,
}

#[derive(Default, Debug)]
pub struct MainMenuState {
    pub selected: usize,
}

#[derive(Default, Debug)]
pub struct QuickActionsState {
    pub selected: usize,
    pub actions: Vec<String>,
}

#[derive(Debug)]
pub struct ModalState {
    pub mode: AppMode,
}
