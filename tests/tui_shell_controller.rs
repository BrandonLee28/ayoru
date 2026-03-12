use ayoru::app::{PlayerRuntime, ProviderRuntime};
use ayoru::core::models::{Episode, StreamCandidate, Title};
use ayoru::player::detect::{DetectError, Player};
use ayoru::tui::action::Action;
use ayoru::tui::controller::TuiController;
use ayoru::tui::library::{LibraryState, SavedTitle, SavedWatch};
use ayoru::tui::state::{Mode, Tab};
use ayoru::tui::storage::LibraryStorage;

struct SearchProvider;

#[async_trait::async_trait]
impl ProviderRuntime for SearchProvider {
    async fn search(&self, _query: &str) -> Result<Vec<Title>, String> {
        Ok(vec![Title {
            id: "show-1".into(),
            name: "Frieren".into(),
        }])
    }

    async fn episodes(&self, _title_id: &str) -> Result<Vec<Episode>, String> {
        Ok(vec![])
    }

    async fn streams(
        &self,
        _title_id: &str,
        _episode: u32,
        _prefer_sub: bool,
    ) -> Result<Vec<StreamCandidate>, String> {
        Ok(vec![])
    }
}

struct NoopPlayerRuntime;

#[async_trait::async_trait]
impl PlayerRuntime for NoopPlayerRuntime {
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
        Ok(())
    }
}

#[tokio::test]
async fn controller_loads_saved_library_state_on_startup() {
    let dir = tempfile::tempdir().unwrap();
    let storage = LibraryStorage::new(dir.path().join("library.json"));
    let seeded = LibraryState {
        favorites: vec![],
        history: vec![],
        recently_watched: vec![SavedWatch {
            title: SavedTitle {
                id: "show-1".into(),
                name: "Frieren".into(),
            },
            episode: 2,
            watched_at: 1,
        }],
    };
    storage.save(&seeded).unwrap();

    let app = TuiController::with_storage(SearchProvider, NoopPlayerRuntime, storage)
        .await
        .unwrap();

    assert_eq!(app.library().recently_watched.len(), 1);
}

#[tokio::test]
async fn toggling_favorite_updates_library_and_persists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.json");
    let storage = LibraryStorage::new(path.clone());
    let mut app = TuiController::with_storage(SearchProvider, NoopPlayerRuntime, storage.clone())
        .await
        .unwrap();

    app.dispatch(Action::InsertChar('f')).await.unwrap();
    app.dispatch(Action::SubmitSearch).await.unwrap();
    app.dispatch(Action::ToggleFavorite).await.unwrap();

    assert_eq!(app.library().favorites.len(), 1);

    let loaded = storage.load().unwrap();
    assert_eq!(loaded.favorites.len(), 1);
}

#[tokio::test]
async fn favorites_tab_opens_selected_title_into_episode_flow() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.json");
    let storage = LibraryStorage::new(path.clone());
    storage
        .save(&LibraryState {
            favorites: vec![SavedTitle {
                id: "show-1".into(),
                name: "Frieren".into(),
            }],
            history: vec![],
            recently_watched: vec![],
        })
        .unwrap();
    let mut app = TuiController::with_storage(SearchProvider, NoopPlayerRuntime, storage)
        .await
        .unwrap();

    app.dispatch(Action::NextTab).await.unwrap();
    app.dispatch(Action::OpenSelectedTitle).await.unwrap();

    assert_eq!(app.state().active_tab, Tab::Favorites);
    assert_eq!(app.state().mode, Mode::Episodes);
    assert_eq!(app.state().current_title.as_ref().unwrap().name, "Frieren");
}

#[tokio::test]
async fn deleting_selected_favorite_updates_library_and_persists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.json");
    let storage = LibraryStorage::new(path.clone());
    storage
        .save(&LibraryState {
            favorites: vec![SavedTitle {
                id: "show-1".into(),
                name: "Frieren".into(),
            }],
            history: vec![],
            recently_watched: vec![],
        })
        .unwrap();
    let mut app = TuiController::with_storage(SearchProvider, NoopPlayerRuntime, storage.clone())
        .await
        .unwrap();

    app.dispatch(Action::NextTab).await.unwrap();
    app.dispatch(Action::DeleteSelectedItem).await.unwrap();

    assert!(app.library().favorites.is_empty());
    assert!(storage.load().unwrap().favorites.is_empty());
}

#[tokio::test]
async fn clearing_history_removes_all_entries_and_persists() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.json");
    let storage = LibraryStorage::new(path.clone());
    storage
        .save(&LibraryState {
            favorites: vec![],
            history: vec![SavedWatch {
                title: SavedTitle {
                    id: "show-1".into(),
                    name: "Frieren".into(),
                },
                episode: 3,
                watched_at: 1,
            }],
            recently_watched: vec![SavedWatch {
                title: SavedTitle {
                    id: "show-1".into(),
                    name: "Frieren".into(),
                },
                episode: 3,
                watched_at: 1,
            }],
        })
        .unwrap();
    let mut app = TuiController::with_storage(SearchProvider, NoopPlayerRuntime, storage.clone())
        .await
        .unwrap();

    app.dispatch(Action::NextTab).await.unwrap();
    app.dispatch(Action::NextTab).await.unwrap();
    app.dispatch(Action::ClearHistory).await.unwrap();

    assert!(app.library().history.is_empty());
    assert!(storage.load().unwrap().history.is_empty());
}
