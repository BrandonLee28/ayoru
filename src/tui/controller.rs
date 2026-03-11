use crate::app::{PlayerRuntime, ProviderRuntime};
use crate::core::playback::{PlaybackError, attempt_with_fallback};
use crate::core::stream_ranker::rank_streams;
use crate::errors::AppError;
use crate::tui::action::{Action, Effect};
use crate::tui::library::{LibraryState, SavedTitle, SavedWatch};
use crate::tui::state::TuiState;
use crate::tui::storage::LibraryStorage;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct TuiController<P, R> {
    provider: P,
    player: R,
    state: TuiState,
    library: LibraryState,
    storage: Option<LibraryStorage>,
}

impl<P, R> TuiController<P, R>
where
    P: ProviderRuntime,
    R: PlayerRuntime,
{
    pub fn new(provider: P, player: R) -> Self {
        let mut controller = Self {
            provider,
            player,
            state: TuiState::default(),
            library: LibraryState::default(),
            storage: None,
        };
        controller.sync_library_to_state();
        controller
    }

    pub async fn with_storage(
        provider: P,
        player: R,
        storage: LibraryStorage,
    ) -> Result<Self, AppError> {
        let library = storage
            .load()
            .map_err(|err| AppError::Provider(err.to_string()))?;

        let mut controller = Self {
            provider,
            player,
            state: TuiState::default(),
            library,
            storage: Some(storage),
        };
        controller.sync_library_to_state();
        Ok(controller)
    }

    pub fn state(&self) -> &TuiState {
        &self.state
    }

    pub fn library(&self) -> &LibraryState {
        &self.library
    }

    pub async fn dispatch(&mut self, action: Action) -> Result<(), AppError> {
        let effect = self.state.apply(action);

        match effect {
            Some(Effect::SearchTitles(query)) => match self.provider.search(&query).await {
                Ok(results) => {
                    self.state.apply(Action::SearchCompleted(results));
                    Ok(())
                }
                Err(err) => {
                    self.state.apply(Action::SearchFailed(err));
                    Ok(())
                }
            },
            Some(Effect::LoadEpisodes(title)) => match self.provider.episodes(&title.id).await {
                Ok(episodes) => {
                    self.state
                        .apply(Action::EpisodesCompleted { title, episodes });
                    Ok(())
                }
                Err(err) => {
                    self.state.apply(Action::EpisodesFailed(err));
                    Ok(())
                }
            },
            Some(Effect::PlayEpisode { title, episode }) => {
                let mut streams = self
                    .provider
                    .streams(&title.id, episode.number, true)
                    .await
                    .map_err(AppError::Provider)?;
                if streams.is_empty() {
                    self.state.apply(Action::PlaybackFailed(
                        "No playable streams found".to_string(),
                    ));
                    return Ok(());
                }

                rank_streams(&mut streams);

                let player = match self.player.detect() {
                    Ok(player) => player,
                    Err(err) => {
                        self.state.apply(Action::PlaybackFailed(err.to_string()));
                        return Ok(());
                    }
                };

                let title_name = title.name.clone();
                let episode_number = episode.number;
                let player_runtime = &self.player;
                let playback_result =
                    attempt_with_fallback(&streams, Duration::from_secs(6), |stream| {
                        let url = stream.url.clone();
                        let title_name = title_name.clone();
                        async move {
                            player_runtime
                                .launch_and_confirm(player, &url, &title_name, episode_number)
                                .await
                        }
                    })
                    .await;

                match playback_result {
                    Ok(()) => {
                        let saved_title = SavedTitle {
                            id: title.id.clone(),
                            name: title.name.clone(),
                        };
                        self.record_watch(&saved_title, episode.number)?;
                        self.state.apply(Action::PlaybackStarted);
                        Ok(())
                    }
                    Err(PlaybackError::AllFailed) => {
                        self.state.apply(Action::PlaybackFailed(
                            "Playback failed after trying all providers".to_string(),
                        ));
                        Ok(())
                    }
                }
            }
            Some(Effect::ToggleFavoriteForSelectedTitle) => {
                if let Some(title) = self.selected_title() {
                    self.library.toggle_favorite(title);
                    self.sync_library_to_state();
                    self.persist_library()?;
                }
                Ok(())
            }
            None => Ok(()),
        }
    }

    fn selected_title(&self) -> Option<SavedTitle> {
        if let Some(title) = &self.state.current_title {
            return Some(SavedTitle {
                id: title.id.clone(),
                name: title.name.clone(),
            });
        }

        self.state
            .results
            .get(self.state.selected_result)
            .map(|title| SavedTitle {
                id: title.id.clone(),
                name: title.name.clone(),
            })
    }

    fn record_watch(&mut self, title: &SavedTitle, episode: u32) -> Result<(), AppError> {
        self.library.record_watch(SavedWatch {
            title: title.clone(),
            episode,
            watched_at: current_unix_timestamp(),
        });
        self.sync_library_to_state();
        self.persist_library()
    }

    fn persist_library(&self) -> Result<(), AppError> {
        if let Some(storage) = &self.storage {
            storage
                .save(&self.library)
                .map_err(|err| AppError::Provider(err.to_string()))?;
        }

        Ok(())
    }

    fn sync_library_to_state(&mut self) {
        self.state.favorites = self.library.favorites.clone();
        self.state.recently_watched = self.library.recently_watched.clone();
        self.state.history = self.library.history.clone();
    }
}

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}
