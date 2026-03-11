use crate::app::{PlayerRuntime, ProviderRuntime};
use crate::errors::AppError;
use crate::tui::action::{Action, Effect};
use crate::tui::state::TuiState;

pub struct TuiController<P, R> {
    provider: P,
    _player: R,
    state: TuiState,
}

impl<P, R> TuiController<P, R>
where
    P: ProviderRuntime,
    R: PlayerRuntime,
{
    pub fn new(provider: P, player: R) -> Self {
        Self {
            provider,
            _player: player,
            state: TuiState::default(),
        }
    }

    pub fn state(&self) -> &TuiState {
        &self.state
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
            None => Ok(()),
        }
    }
}
