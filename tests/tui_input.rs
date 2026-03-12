use ayoru::tui::action::Action;
use ayoru::tui::runtime::{InputCommand, map_key_code_for_state};
use ayoru::tui::state::{Mode, Tab, TuiState};
use crossterm::event::KeyCode;

#[test]
fn slash_and_text_input_focus_search_and_append_query() {
    assert_eq!(
        map_key_code_for_state(&TuiState::default(), KeyCode::Char('/')),
        Some(InputCommand::FocusSearch)
    );
    assert_eq!(
        map_key_code_for_state(&TuiState::default(), KeyCode::Char('f')),
        Some(InputCommand::Action(Action::InsertChar('f')))
    );
    assert_eq!(
        map_key_code_for_state(&TuiState::default(), KeyCode::Backspace),
        Some(InputCommand::Action(Action::DeleteChar))
    );
}

#[test]
fn navigation_keys_map_to_selection_actions() {
    let state = TuiState {
        mode: Mode::Search,
        search_focused: false,
        active_tab: Tab::MediaBrowser,
        ..Default::default()
    };

    assert_eq!(
        map_key_code_for_state(&state, KeyCode::Down),
        Some(InputCommand::Action(Action::MoveDown))
    );
    assert_eq!(
        map_key_code_for_state(&state, KeyCode::Char('k')),
        Some(InputCommand::Action(Action::MoveUp))
    );
}

#[test]
fn search_focus_treats_j_and_k_as_query_text() {
    let state = TuiState::default();

    assert_eq!(
        map_key_code_for_state(&state, KeyCode::Char('j')),
        Some(InputCommand::Action(Action::InsertChar('j')))
    );
    assert_eq!(
        map_key_code_for_state(&state, KeyCode::Char('k')),
        Some(InputCommand::Action(Action::InsertChar('k')))
    );
}

#[test]
fn tab_moves_between_shell_panels() {
    let state = TuiState::default();

    assert_eq!(
        map_key_code_for_state(&state, KeyCode::Tab),
        Some(InputCommand::Action(Action::NextTab))
    );
}

#[test]
fn f_toggles_favorite_when_search_is_not_focused() {
    let state = TuiState {
        search_focused: false,
        active_tab: Tab::MediaBrowser,
        ..Default::default()
    };

    assert_eq!(
        map_key_code_for_state(&state, KeyCode::Char('f')),
        Some(InputCommand::Action(Action::ToggleFavorite))
    );
}

#[test]
fn d_removes_items_from_editable_tabs() {
    let favorite_state = TuiState {
        search_focused: false,
        active_tab: Tab::Favorites,
        ..Default::default()
    };
    let history_state = TuiState {
        search_focused: false,
        active_tab: Tab::History,
        ..Default::default()
    };

    assert_eq!(
        map_key_code_for_state(&favorite_state, KeyCode::Char('d')),
        Some(InputCommand::Action(Action::DeleteSelectedItem))
    );
    assert_eq!(
        map_key_code_for_state(&history_state, KeyCode::Char('d')),
        Some(InputCommand::Action(Action::DeleteSelectedItem))
    );
}

#[test]
fn shift_d_clears_history_only_on_history_tab() {
    let history_state = TuiState {
        search_focused: false,
        active_tab: Tab::History,
        ..Default::default()
    };
    let browser_state = TuiState {
        search_focused: false,
        active_tab: Tab::MediaBrowser,
        ..Default::default()
    };

    assert_eq!(
        map_key_code_for_state(&history_state, KeyCode::Char('D')),
        Some(InputCommand::Action(Action::ClearHistory))
    );
    assert_eq!(
        map_key_code_for_state(&browser_state, KeyCode::Char('D')),
        None
    );
}

#[test]
fn o_opens_show_from_history_tab() {
    let history_state = TuiState {
        search_focused: false,
        active_tab: Tab::History,
        ..Default::default()
    };

    assert_eq!(
        map_key_code_for_state(&history_state, KeyCode::Char('o')),
        Some(InputCommand::Action(Action::OpenSelectedTitle))
    );
}
