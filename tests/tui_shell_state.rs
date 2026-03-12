use ayoru::core::models::Title;
use ayoru::tui::action::{Action, Effect};
use ayoru::tui::library::{SavedTitle, SavedWatch};
use ayoru::tui::state::{Tab, TuiState};

fn search_state_with_results() -> TuiState {
    TuiState {
        search_focused: false,
        results: vec![Title {
            id: "show-1".into(),
            name: "Frieren".into(),
        }],
        ..Default::default()
    }
}

#[test]
fn tab_cycles_between_top_level_tabs() {
    let mut state = TuiState::default();

    state.apply(Action::NextTab);
    assert_eq!(state.active_tab, Tab::Favorites);

    state.apply(Action::NextTab);
    assert_eq!(state.active_tab, Tab::History);

    state.apply(Action::NextTab);
    assert_eq!(state.active_tab, Tab::MediaBrowser);
}

#[test]
fn favorite_action_marks_selected_result() {
    let mut state = search_state_with_results();

    let effect = state.apply(Action::ToggleFavorite);

    assert_eq!(effect, Some(Effect::ToggleFavoriteForSelectedTitle));
}

#[test]
fn opening_selected_title_uses_active_tab_selection() {
    let mut state = TuiState {
        search_focused: false,
        active_tab: Tab::Favorites,
        favorites: vec![SavedTitle {
            id: "show-1".into(),
            name: "Frieren".into(),
        }],
        ..Default::default()
    };

    let effect = state.apply(Action::OpenSelectedTitle);

    assert_eq!(
        effect,
        Some(Effect::LoadEpisodes(Title {
            id: "show-1".into(),
            name: "Frieren".into(),
        }))
    );
}

#[test]
fn deleting_selected_history_item_emits_effect() {
    let mut state = TuiState {
        search_focused: false,
        active_tab: Tab::History,
        history: vec![SavedWatch {
            title: SavedTitle {
                id: "show-1".into(),
                name: "Frieren".into(),
            },
            episode: 3,
            watched_at: 1,
        }],
        ..Default::default()
    };

    let effect = state.apply(Action::DeleteSelectedItem);

    assert_eq!(effect, Some(Effect::DeleteSelectedLibraryItem));
}
