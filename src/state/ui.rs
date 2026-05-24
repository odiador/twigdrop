use crate::app::PreviewState;
use crate::app::PrimaryMode;
use crate::app::FilePanel;

#[allow(dead_code)]
#[derive(PartialEq, Debug, Clone)]
pub enum ModalType {
    Help,
    Manage,
    Filter,
    StashDetail,
    Settings,
    Search,
    ConfirmDelete(Vec<String>),
    CreateBranch(String),
    Commits,
    CommitAction(String),
    Shell(String),
    QuickActions,
    Message(String),
    CommandPalette(String), // For global actions
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ModalState {
    pub modal_type: ModalType,
    pub focused_index: usize,
}

#[allow(dead_code)]
pub struct UiState {
    pub primary_mode: PrimaryMode,
    pub modal_stack: Vec<ModalState>,
    pub code_preview: Option<PreviewState>,
    pub diff_panel: FilePanel,
    pub needs_clear: bool,
    pub show_terminal: bool,
    
    // Virtualization / scrolling
    pub list_offsets: std::collections::HashMap<String, usize>,
}

impl UiState {
    pub fn new(primary_mode: PrimaryMode) -> Self {
        Self {
            primary_mode,
            modal_stack: Vec::new(),
            code_preview: None,
            diff_panel: FilePanel::Directory,
            needs_clear: false,
            show_terminal: false,
            list_offsets: std::collections::HashMap::new(),
        }
    }

    pub fn push_modal(&mut self, modal: ModalType) {
        self.modal_stack.push(ModalState {
            modal_type: modal,
            focused_index: 0,
        });
    }

    pub fn pop_modal(&mut self) -> Option<ModalState> {
        let popped = self.modal_stack.pop();
        self.needs_clear = true;
        popped
    }

    pub fn current_modal(&self) -> Option<&ModalState> {
        self.modal_stack.last()
    }
    
    pub fn current_modal_mut(&mut self) -> Option<&mut ModalState> {
        self.modal_stack.last_mut()
    }
}
