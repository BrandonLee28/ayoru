use ayoru::library::storage::LibraryStorage;
use ayoru::library::{LibraryState, SavedTitle, SavedWatch};

fn saved_watch(id: &str, name: &str, episode: u32) -> SavedWatch {
    SavedWatch {
        title: SavedTitle {
            id: id.to_string(),
            name: name.to_string(),
            poster: None,
        },
        episode,
        watched_at: 1,
    }
}

#[test]
fn saves_and_loads_library_state_as_json() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.json");
    let storage = LibraryStorage::new(path);

    let mut library = LibraryState::default();
    library.record_watch(saved_watch("show-1", "Frieren", 1));

    storage.save(&library).unwrap();
    let loaded = storage.load().unwrap();

    assert_eq!(loaded.history.len(), 1);
    assert_eq!(loaded.recently_watched.len(), 1);
}

#[test]
fn missing_library_file_loads_as_empty_state() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing.json");
    let storage = LibraryStorage::new(path);

    let loaded = storage.load().unwrap();

    assert_eq!(loaded, LibraryState::default());
}

#[test]
fn imports_existing_terminal_library_without_changing_the_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.json");
    let legacy = r#"{"favorites":[{"id":"show-1","name":"Frieren"}],"history":[{"title":{"id":"show-1","name":"Frieren"},"episode":3,"watched_at":42}],"recently_watched":[]}"#;
    std::fs::write(&path, legacy).unwrap();
    let storage = LibraryStorage::new(path.clone());
    let mut library = storage.load().unwrap();
    assert_eq!(library.favorites[0].poster, None);
    assert_eq!(library.history[0].episode, 3);
    assert_eq!(std::fs::read_to_string(&path).unwrap(), legacy);
    library.favorites[0].poster = Some("https://example.com/poster.jpg".into());
    storage.save(&library).unwrap();
    assert_eq!(storage.load().unwrap(), library);
    assert!(!path.with_extension("json.tmp").exists());
}

#[test]
fn corrupt_library_returns_an_error_and_stays_intact() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("library.json");
    std::fs::write(&path, "broken JSON").unwrap();
    assert!(LibraryStorage::new(path.clone()).load().is_err());
    assert_eq!(std::fs::read_to_string(path).unwrap(), "broken JSON");
}
