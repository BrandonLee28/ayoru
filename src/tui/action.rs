use crate::core::models::{Episode, Title};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    InsertChar(char),
    DeleteChar,
    FocusSearch,
    NextTab,
    PrevTab,
    MoveUp,
    MoveDown,
    ToggleFavorite,
    SubmitSearch,
    OpenSelectedTitle,
    PlaySelectedHistory,
    PlaySelectedEpisode,
    DeleteSelectedItem,
    ClearHistory,
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
    PlayHistoryEntry,
    PlayEpisode { title: Title, episode: Episode },
    ToggleFavoriteForSelectedTitle,
    DeleteSelectedLibraryItem,
    ClearHistoryLibrary,
}
