use ayoru::core::models::Title;
use ayoru::tui::library::{SavedTitle, SavedWatch};
use ayoru::tui::state::{Tab, TuiState};
use ayoru::tui::ui::render_to_buffer;

fn buffer_contains(buffer: &ratatui::buffer::Buffer, needle: &str) -> bool {
    let content = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    content.contains(needle)
}

fn shell_fixture_state() -> TuiState {
    TuiState {
        search_focused: false,
        active_tab: Tab::History,
        query: "frieren".into(),
        results: vec![Title {
            id: "show-1".into(),
            name: "Frieren".into(),
        }],
        favorites: vec![SavedTitle {
            id: "show-1".into(),
            name: "Frieren".into(),
        }],
        recently_watched: vec![SavedWatch {
            title: SavedTitle {
                id: "show-1".into(),
                name: "Frieren".into(),
            },
            episode: 3,
            watched_at: 1,
        }],
        history: vec![SavedWatch {
            title: SavedTitle {
                id: "show-2".into(),
                name: "Dungeon Meshi".into(),
            },
            episode: 8,
            watched_at: 1,
        }],
        ..Default::default()
    }
}

#[test]
fn shell_render_shows_search_header_and_context_sections() {
    let buffer = render_to_buffer(&shell_fixture_state(), 120, 32);

    assert!(buffer_contains(&buffer, "AYORU"));
    assert!(buffer_contains(&buffer, "Media Browser"));
    assert!(buffer_contains(&buffer, "Favorites"));
    assert!(buffer_contains(&buffer, "History"));
    assert!(buffer_contains(&buffer, "Dungeon Meshi"));
    assert!(!buffer_contains(&buffer, "Up Next"));
}

#[test]
fn shell_render_shows_latest_history_entries_when_panel_overflows() {
    let mut state = shell_fixture_state();
    state.history = (0..8)
        .map(|idx| SavedWatch {
            title: SavedTitle {
                id: format!("show-{idx}"),
                name: if idx == 0 {
                    "Oldest".into()
                } else if idx == 7 {
                    "Newest".into()
                } else {
                    format!("Entry {idx}")
                },
            },
            episode: idx + 1,
            watched_at: idx as u64 + 1,
        })
        .collect();

    let buffer = render_to_buffer(&state, 120, 20);

    assert!(buffer_contains(&buffer, "History"));
    assert!(buffer_contains(&buffer, "Newest"));
    assert!(!buffer_contains(&buffer, "Oldest"));
}
