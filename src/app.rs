use crate::core::models::{Episode, StreamCandidate, Title};
use crate::player::detect::{DetectError, Player, detect_player};
use crate::player::launch::spawn_player;

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

#[async_trait::async_trait]
pub trait PlayerRuntime {
    fn detect(&self) -> Result<Player, DetectError>;
    async fn launch_and_confirm(
        &self,
        player: Player,
        stream_url: &str,
        title: &str,
        episode: u32,
    ) -> Result<(), std::io::Error>;
}

pub struct SystemPlayerRuntime;

#[async_trait::async_trait]
impl PlayerRuntime for SystemPlayerRuntime {
    fn detect(&self) -> Result<Player, DetectError> {
        detect_player()
    }

    async fn launch_and_confirm(
        &self,
        player: Player,
        stream_url: &str,
        title: &str,
        episode: u32,
    ) -> Result<(), std::io::Error> {
        let media_title = format!("{title} Episode {episode}");
        spawn_player(player, stream_url, &media_title)
    }
}
