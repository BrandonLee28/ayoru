use crate::core::models::Title;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    InsertChar(char),
    SubmitSearch,
    SearchCompleted(Vec<Title>),
    SearchFailed(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    SearchTitles(String),
}
