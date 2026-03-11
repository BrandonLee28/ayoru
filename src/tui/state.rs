use crate::core::models::Title;
use crate::tui::action::{Action, Effect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Search,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TuiState {
    pub mode: Mode,
    pub query: String,
    pub is_loading: bool,
    pub results: Vec<Title>,
    pub selected_result: usize,
    pub message: Option<String>,
}

impl TuiState {
    pub fn apply(&mut self, action: Action) -> Option<Effect> {
        match action {
            Action::InsertChar(ch) => {
                self.query.push(ch);
                None
            }
            Action::SubmitSearch => {
                self.mode = Mode::Search;
                self.is_loading = true;
                self.message = None;
                Some(Effect::SearchTitles(self.query.clone()))
            }
            Action::SearchCompleted(results) => {
                self.is_loading = false;
                self.selected_result = 0;
                self.results = results;
                None
            }
            Action::SearchFailed(message) => {
                self.is_loading = false;
                self.message = Some(message);
                None
            }
        }
    }
}
