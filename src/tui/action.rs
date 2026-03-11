use crate::core::models::{Episode, Title};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    InsertChar(char),
    SubmitSearch,
    OpenSelectedTitle,
    Back,
    SearchCompleted(Vec<Title>),
    SearchFailed(String),
    EpisodesCompleted { title: Title, episodes: Vec<Episode> },
    EpisodesFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    SearchTitles(String),
    LoadEpisodes(Title),
}
