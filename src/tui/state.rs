use crate::core::models::{Episode, Title};
use crate::tui::action::{Action, Effect};
use crate::tui::library::{SavedTitle, SavedWatch};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Search,
    Episodes,
    Launching,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Panel {
    #[default]
    Search,
    Main,
    ContextRail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiState {
    pub mode: Mode,
    pub focused_panel: Panel,
    pub search_focused: bool,
    pub query: String,
    pub is_loading: bool,
    pub results: Vec<Title>,
    pub selected_result: usize,
    pub current_title: Option<Title>,
    pub episodes: Vec<Episode>,
    pub selected_episode: usize,
    pub favorites: Vec<SavedTitle>,
    pub recently_watched: Vec<SavedWatch>,
    pub history: Vec<SavedWatch>,
    pub message: Option<String>,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            mode: Mode::Search,
            focused_panel: Panel::Search,
            search_focused: true,
            query: String::new(),
            is_loading: false,
            results: vec![],
            selected_result: 0,
            current_title: None,
            episodes: vec![],
            selected_episode: 0,
            favorites: vec![],
            recently_watched: vec![],
            history: vec![],
            message: None,
        }
    }
}

impl TuiState {
    pub fn apply(&mut self, action: Action) -> Option<Effect> {
        match action {
            Action::InsertChar(ch) => {
                if self.mode == Mode::Search {
                    self.search_focused = true;
                }
                self.query.push(ch);
                None
            }
            Action::FocusSearch => {
                self.mode = Mode::Search;
                self.focused_panel = Panel::Search;
                self.search_focused = true;
                self.message = None;
                None
            }
            Action::FocusNextPanel => {
                self.focused_panel = match self.focused_panel {
                    Panel::Search => Panel::ContextRail,
                    Panel::ContextRail => Panel::Main,
                    Panel::Main => Panel::Search,
                };
                None
            }
            Action::FocusPrevPanel => {
                self.focused_panel = match self.focused_panel {
                    Panel::Search => Panel::Main,
                    Panel::Main => Panel::ContextRail,
                    Panel::ContextRail => Panel::Search,
                };
                None
            }
            Action::MoveUp => {
                match self.mode {
                    Mode::Search => {
                        self.selected_result = self.selected_result.saturating_sub(1);
                    }
                    Mode::Episodes => {
                        self.selected_episode = self.selected_episode.saturating_sub(1);
                    }
                    Mode::Launching => {}
                }
                None
            }
            Action::MoveDown => {
                match self.mode {
                    Mode::Search => {
                        if !self.results.is_empty() {
                            self.selected_result =
                                (self.selected_result + 1).min(self.results.len() - 1);
                        }
                    }
                    Mode::Episodes => {
                        if !self.episodes.is_empty() {
                            self.selected_episode =
                                (self.selected_episode + 1).min(self.episodes.len() - 1);
                        }
                    }
                    Mode::Launching => {}
                }
                None
            }
            Action::ToggleFavorite => {
                if self.current_title.is_some() || !self.results.is_empty() {
                    Some(Effect::ToggleFavoriteForSelectedTitle)
                } else {
                    None
                }
            }
            Action::SubmitSearch => {
                self.mode = Mode::Search;
                self.focused_panel = Panel::Main;
                self.search_focused = false;
                self.is_loading = true;
                self.message = None;
                Some(Effect::SearchTitles(self.query.clone()))
            }
            Action::OpenSelectedTitle => {
                let title = self.results.get(self.selected_result)?.clone();
                self.is_loading = true;
                self.message = None;
                Some(Effect::LoadEpisodes(title))
            }
            Action::PlaySelectedEpisode => {
                let title = self.current_title.clone()?;
                let episode = self.episodes.get(self.selected_episode)?.clone();
                self.mode = Mode::Launching;
                self.is_loading = true;
                self.message = None;
                Some(Effect::PlayEpisode { title, episode })
            }
            Action::Back => {
                self.mode = Mode::Search;
                self.focused_panel = Panel::Main;
                self.search_focused = false;
                self.is_loading = false;
                self.message = None;
                None
            }
            Action::SearchCompleted(results) => {
                self.is_loading = false;
                self.focused_panel = Panel::Main;
                self.selected_result = 0;
                self.results = results;
                None
            }
            Action::SearchFailed(message) => {
                self.is_loading = false;
                self.message = Some(message);
                None
            }
            Action::EpisodesCompleted { title, episodes } => {
                self.mode = Mode::Episodes;
                self.is_loading = false;
                self.current_title = Some(title);
                self.selected_episode = 0;
                self.episodes = episodes;
                None
            }
            Action::EpisodesFailed(message) => {
                self.is_loading = false;
                self.message = Some(message);
                None
            }
            Action::PlaybackStarted => {
                self.mode = Mode::Episodes;
                self.is_loading = false;
                self.message = Some("Playback started".to_string());
                None
            }
            Action::PlaybackFailed(message) => {
                self.mode = Mode::Episodes;
                self.is_loading = false;
                self.message = Some(message);
                None
            }
        }
    }
}
