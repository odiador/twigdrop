#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum RebaseAction {
    #[default]
    Pick,
    Reword,
    Drop,
    Squash,
}

#[derive(Default, Clone, Debug)]
pub struct RebaseCommit {
    pub hash: String,
    pub original_message: String,
    pub new_message: Option<String>,
    pub action: RebaseAction,
}

#[derive(Default, Debug)]
pub struct RebaseState {
    pub commits: Vec<RebaseCommit>,
    pub selected: usize,
    pub editing: bool,
    pub input: String,
    pub ai_analyzing: bool,
}
