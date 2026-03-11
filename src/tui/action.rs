use crate::core::models::{Episode, Title};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    InsertChar(char),
    FocusSearch,
    FocusNextPanel,
    FocusPrevPanel,
    MoveUp,
    MoveDown,
    ToggleFavorite,
    SubmitSearch,
    OpenSelectedTitle,
    PlaySelectedEpisode,
    Back,
    SearchCompleted(Vec<Title>),
    SearchFailed(String),
    EpisodesCompleted {
        title: Title,
        episodes: Vec<Episode>,
    },
    EpisodesFailed(String),
    PlaybackStarted,
    PlaybackFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    SearchTitles(String),
    LoadEpisodes(Title),
    PlayEpisode { title: Title, episode: Episode },
    ToggleFavoriteForSelectedTitle,
}
