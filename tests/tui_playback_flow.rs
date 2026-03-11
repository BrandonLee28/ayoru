use ani::app::{PlayerRuntime, ProviderRuntime};
use ani::core::models::{Episode, StreamCandidate, Title};
use ani::player::detect::{DetectError, Player};
use ani::tui::action::Action;
use ani::tui::controller::TuiController;
use ani::tui::state::Mode;

struct PlaybackProvider;

#[async_trait::async_trait]
impl ProviderRuntime for PlaybackProvider {
    async fn search(&self, _query: &str) -> Result<Vec<Title>, String> {
        Ok(vec![Title {
            id: "show-1".into(),
            name: "Frieren".into(),
        }])
    }

    async fn episodes(&self, _title_id: &str) -> Result<Vec<Episode>, String> {
        Ok(vec![Episode { number: 1 }])
    }

    async fn streams(
        &self,
        _title_id: &str,
        _episode: u32,
        _prefer_sub: bool,
    ) -> Result<Vec<StreamCandidate>, String> {
        Ok(vec![StreamCandidate {
            provider: "wixmp".into(),
            url: "https://example.test/stream".into(),
            is_sub: true,
            resolution: Some(1080),
        }])
    }
}

struct FailingPlayerRuntime;

#[async_trait::async_trait]
impl PlayerRuntime for FailingPlayerRuntime {
    fn detect(&self) -> Result<Player, DetectError> {
        Ok(Player::Mpv)
    }

    async fn launch_and_confirm(
        &self,
        _player: Player,
        _stream_url: &str,
        _title: &str,
        _episode: u32,
    ) -> Result<(), std::io::Error> {
        Err(std::io::Error::other("launch failed"))
    }
}

#[tokio::test]
async fn play_episode_returns_to_episodes_with_failure_message() {
    let mut app = TuiController::new(PlaybackProvider, FailingPlayerRuntime);

    for ch in "frieren".chars() {
        app.dispatch(Action::InsertChar(ch)).await.unwrap();
    }
    app.dispatch(Action::SubmitSearch).await.unwrap();
    app.dispatch(Action::OpenSelectedTitle).await.unwrap();
    app.dispatch(Action::PlaySelectedEpisode).await.unwrap();

    assert_eq!(app.state().mode, Mode::Episodes);
    assert_eq!(
        app.state().message.as_deref(),
        Some("Playback failed after trying all providers")
    );
}
