use ayoru::core::models::Title;
use ayoru::tui::state::{Mode, Tab, TuiState};
use ayoru::tui::ui::render_to_buffer;
use ratatui::style::Color;

fn buffer_contains(buffer: &ratatui::buffer::Buffer, needle: &str) -> bool {
    let content = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    content.contains(needle)
}

fn first_fg_for_symbol(buffer: &ratatui::buffer::Buffer, needle: &str) -> Option<Color> {
    buffer
        .content
        .iter()
        .find(|cell| cell.symbol() == needle)
        .map(|cell| cell.fg)
}

fn row_contains_bg(buffer: &ratatui::buffer::Buffer, needle: &str, color: Color) -> bool {
    let width = buffer.area.width as usize;
    let content = &buffer.content;

    for row in content.chunks(width) {
        let row_text = row.iter().map(|cell| cell.symbol()).collect::<String>();
        if row_text.contains(needle) {
            return row.iter().any(|cell| cell.bg == color);
        }
    }

    false
}

#[test]
fn render_search_screen_shows_query_results_and_key_hints() {
    let state = TuiState {
        mode: Mode::Search,
        search_focused: false,
        active_tab: Tab::MediaBrowser,
        query: "fri".into(),
        is_loading: false,
        results: vec![
            Title {
                id: "show-1".into(),
                name: "Frieren".into(),
            },
            Title {
                id: "show-2".into(),
                name: "Fullmetal Alchemist".into(),
            },
        ],
        selected_result: 0,
        current_title: None,
        episodes: vec![],
        selected_episode: 0,
        favorites: vec![],
        selected_favorite: 0,
        history: vec![],
        selected_history: 0,
        recently_watched: vec![],
        message: Some("Ready".into()),
    };

    let buffer = render_to_buffer(&state, 80, 24);

    assert!(buffer_contains(&buffer, "AYORU"));
    assert!(buffer_contains(&buffer, "Media Browser"));
    assert!(buffer_contains(&buffer, "Search titles"));
    assert!(buffer_contains(&buffer, "Frieren"));
    assert!(buffer_contains(&buffer, "Ready"));
    assert!(buffer_contains(&buffer, "Favorites"));
    assert!(buffer_contains(&buffer, "History"));
    assert!(buffer_contains(&buffer, "Enter open"));
    assert!(row_contains_bg(
        &buffer,
        "Enter opens episodes",
        Color::Rgb(94, 116, 143)
    ));
    assert!(!buffer_contains(&buffer, "> Frieren"));
    assert!(buffer_contains(&buffer, "Ready"));
    assert_eq!(
        first_fg_for_symbol(&buffer, "A"),
        Some(Color::Rgb(231, 232, 234))
    );
}

#[test]
fn render_empty_search_screen_uses_branded_copy() {
    let buffer = render_to_buffer(&TuiState::default(), 80, 24);

    assert!(buffer_contains(&buffer, "AYORU"));
    assert!(buffer_contains(&buffer, "Media Browser"));
    assert!(buffer_contains(&buffer, "Start with a title"));
    assert!(buffer_contains(&buffer, "Search titles"));
    assert!(buffer_contains(&buffer, "Type to search, then press Enter"));
}

#[test]
fn render_history_tab_shows_play_and_open_hints() {
    let state = TuiState {
        mode: Mode::Search,
        active_tab: Tab::History,
        search_focused: false,
        history: vec![ayoru::tui::library::SavedWatch {
            title: ayoru::tui::library::SavedTitle {
                id: "show-1".into(),
                name: "Frieren".into(),
            },
            episode: 9,
            watched_at: 1,
        }],
        ..Default::default()
    };

    let buffer = render_to_buffer(&state, 100, 24);

    assert!(buffer_contains(&buffer, "Enter play"));
    assert!(buffer_contains(&buffer, "o open show"));
    assert!(buffer_contains(&buffer, "Episode 9"));
}

#[test]
fn render_search_screen_scrolls_selected_result_into_view() {
    let state = TuiState {
        mode: Mode::Search,
        search_focused: false,
        active_tab: Tab::MediaBrowser,
        query: "one piece".into(),
        results: (0..20)
            .map(|idx| Title {
                id: format!("show-{idx}"),
                name: format!("Title {idx}"),
            })
            .collect(),
        selected_result: 15,
        ..Default::default()
    };

    let buffer = render_to_buffer(&state, 100, 18);

    assert!(buffer_contains(&buffer, "Title 15"));
    assert!(!buffer_contains(&buffer, "Title 0"));
}
