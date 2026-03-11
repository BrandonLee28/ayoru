use crate::core::models::{Episode, Title};
use crate::tui::action::{Action, Effect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Search,
    Episodes,
    Launching,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TuiState {
    pub mode: Mode,
    pub query: String,
    pub is_loading: bool,
    pub results: Vec<Title>,
    pub selected_result: usize,
    pub current_title: Option<Title>,
    pub episodes: Vec<Episode>,
    pub selected_episode: usize,
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
                self.is_loading = false;
                self.message = None;
                None
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
