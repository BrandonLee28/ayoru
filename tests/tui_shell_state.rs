use ani::core::models::Title;
use ani::tui::action::{Action, Effect};
use ani::tui::state::{Panel, TuiState};

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
fn tab_moves_focus_from_search_to_context_rail() {
    let mut state = TuiState::default();

    state.apply(Action::FocusNextPanel);

    assert_eq!(state.focused_panel, Panel::ContextRail);
}

#[test]
fn favorite_action_marks_selected_result() {
    let mut state = search_state_with_results();

    let effect = state.apply(Action::ToggleFavorite);

    assert_eq!(effect, Some(Effect::ToggleFavoriteForSelectedTitle));
}
