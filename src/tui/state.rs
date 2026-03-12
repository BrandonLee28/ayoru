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
pub enum Tab {
    #[default]
    MediaBrowser,
    Favorites,
    History,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TuiState {
    pub mode: Mode,
    pub active_tab: Tab,
    pub search_focused: bool,
    pub query: String,
    pub is_loading: bool,
    pub results: Vec<Title>,
    pub selected_result: usize,
    pub current_title: Option<Title>,
    pub episodes: Vec<Episode>,
    pub selected_episode: usize,
    pub favorites: Vec<SavedTitle>,
    pub selected_favorite: usize,
    pub history: Vec<SavedWatch>,
    pub selected_history: usize,
    pub recently_watched: Vec<SavedWatch>,
    pub message: Option<String>,
}

impl Default for TuiState {
    fn default() -> Self {
        Self {
            mode: Mode::Search,
            active_tab: Tab::MediaBrowser,
            search_focused: true,
            query: String::new(),
            is_loading: false,
            results: vec![],
            selected_result: 0,
            current_title: None,
            episodes: vec![],
            selected_episode: 0,
            favorites: vec![],
            selected_favorite: 0,
            history: vec![],
            selected_history: 0,
            recently_watched: vec![],
            message: None,
        }
    }
}

impl TuiState {
    pub fn apply(&mut self, action: Action) -> Option<Effect> {
        match action {
            Action::InsertChar(ch) => {
                if self.mode == Mode::Search {
                    self.active_tab = Tab::MediaBrowser;
                    self.search_focused = true;
                }
                self.query.push(ch);
                None
            }
            Action::DeleteChar => {
                if self.mode == Mode::Search {
                    self.active_tab = Tab::MediaBrowser;
                    self.search_focused = true;
                }
                self.query.pop();
                None
            }
            Action::FocusSearch => {
                self.mode = Mode::Search;
                self.active_tab = Tab::MediaBrowser;
                self.search_focused = true;
                self.message = None;
                None
            }
            Action::NextTab => {
                if self.mode == Mode::Search {
                    self.active_tab = match self.active_tab {
                        Tab::MediaBrowser => Tab::Favorites,
                        Tab::Favorites => Tab::History,
                        Tab::History => Tab::MediaBrowser,
                    };
                    self.search_focused = false;
                }
                None
            }
            Action::PrevTab => {
                if self.mode == Mode::Search {
                    self.active_tab = match self.active_tab {
                        Tab::MediaBrowser => Tab::History,
                        Tab::Favorites => Tab::MediaBrowser,
                        Tab::History => Tab::Favorites,
                    };
                    self.search_focused = false;
                }
                None
            }
            Action::MoveUp => {
                match self.mode {
                    Mode::Search => match self.active_tab {
                        Tab::MediaBrowser => {
                            self.selected_result = self.selected_result.saturating_sub(1);
                        }
                        Tab::Favorites => {
                            self.selected_favorite = self.selected_favorite.saturating_sub(1);
                        }
                        Tab::History => {
                            self.selected_history = self.selected_history.saturating_sub(1);
                        }
                    },
                    Mode::Episodes => {
                        self.selected_episode = self.selected_episode.saturating_sub(1);
                    }
                    Mode::Launching => {}
                }
                None
            }
            Action::MoveDown => {
                match self.mode {
                    Mode::Search => match self.active_tab {
                        Tab::MediaBrowser => {
                            if !self.results.is_empty() {
                                self.selected_result =
                                    (self.selected_result + 1).min(self.results.len() - 1);
                            }
                        }
                        Tab::Favorites => {
                            if !self.favorites.is_empty() {
                                self.selected_favorite =
                                    (self.selected_favorite + 1).min(self.favorites.len() - 1);
                            }
                        }
                        Tab::History => {
                            if !self.history.is_empty() {
                                self.selected_history =
                                    (self.selected_history + 1).min(self.history.len() - 1);
                            }
                        }
                    },
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
                if self.selected_title().is_some() {
                    Some(Effect::ToggleFavoriteForSelectedTitle)
                } else {
                    None
                }
            }
            Action::SubmitSearch => {
                self.mode = Mode::Search;
                self.active_tab = Tab::MediaBrowser;
                self.search_focused = false;
                self.is_loading = true;
                self.message = None;
                Some(Effect::SearchTitles(self.query.clone()))
            }
            Action::OpenSelectedTitle => {
                let title = self.selected_title()?;
                self.is_loading = true;
                self.message = None;
                Some(Effect::LoadEpisodes(title))
            }
            Action::PlaySelectedHistory => {
                if self.mode == Mode::Search
                    && self.active_tab == Tab::History
                    && !self.history.is_empty()
                {
                    self.is_loading = true;
                    self.message = None;
                    Some(Effect::PlayHistoryEntry)
                } else {
                    None
                }
            }
            Action::PlaySelectedEpisode => {
                let title = self.current_title.clone()?;
                let episode = self.episodes.get(self.selected_episode)?.clone();
                self.mode = Mode::Launching;
                self.is_loading = true;
                self.message = None;
                Some(Effect::PlayEpisode { title, episode })
            }
            Action::DeleteSelectedItem => {
                if self.mode == Mode::Search {
                    match self.active_tab {
                        Tab::Favorites if !self.favorites.is_empty() => {
                            Some(Effect::DeleteSelectedLibraryItem)
                        }
                        Tab::History if !self.history.is_empty() => {
                            Some(Effect::DeleteSelectedLibraryItem)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            Action::ClearHistory => {
                if self.mode == Mode::Search
                    && self.active_tab == Tab::History
                    && !self.history.is_empty()
                {
                    Some(Effect::ClearHistoryLibrary)
                } else {
                    None
                }
            }
            Action::Back => {
                self.mode = Mode::Search;
                self.search_focused = false;
                self.is_loading = false;
                self.message = None;
                None
            }
            Action::SearchCompleted(results) => {
                self.is_loading = false;
                self.active_tab = Tab::MediaBrowser;
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

    pub fn selected_title(&self) -> Option<Title> {
        if let Some(title) = &self.current_title {
            return Some(title.clone());
        }

        match self.active_tab {
            Tab::MediaBrowser => self.results.get(self.selected_result).cloned(),
            Tab::Favorites => self
                .favorites
                .get(self.selected_favorite)
                .map(|title| Title {
                    id: title.id.clone(),
                    name: title.name.clone(),
                }),
            Tab::History => self.history.get(self.selected_history).map(|watch| Title {
                id: watch.title.id.clone(),
                name: watch.title.name.clone(),
            }),
        }
    }

    pub fn clamp_library_selections(&mut self) {
        self.selected_favorite = self
            .selected_favorite
            .min(self.favorites.len().saturating_sub(1));
        self.selected_history = self
            .selected_history
            .min(self.history.len().saturating_sub(1));
    }
}
