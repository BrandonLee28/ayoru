use crate::core::models::{Episode, StreamCandidate, Title};

#[async_trait::async_trait]
pub trait ProviderRuntime {
    async fn search(&self, query: &str) -> Result<Vec<Title>, String>;
    async fn episodes(&self, title_id: &str) -> Result<Vec<Episode>, String>;
    async fn streams(
        &self,
        title_id: &str,
        episode: u32,
        prefer_sub: bool,
    ) -> Result<Vec<StreamCandidate>, String>;
}
