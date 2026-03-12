use crate::tui::state::{Mode, Tab, TuiState};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget};

const GRAPHITE: Color = Color::Rgb(22, 24, 28);
const SOFT_CHARCOAL: Color = Color::Rgb(29, 33, 39);
const SMOKE: Color = Color::Rgb(38, 44, 52);
const WARM_SILVER: Color = Color::Rgb(231, 232, 234);
const MUTED_SILVER: Color = Color::Rgb(184, 190, 199);
const QUIET_GRAY: Color = Color::Rgb(139, 147, 161);
const STEEL_BLUE: Color = Color::Rgb(94, 116, 143);
const MIST_BLUE: Color = Color::Rgb(142, 163, 186);

pub fn render_to_buffer(state: &TuiState, width: u16, height: u16) -> Buffer {
    let area = Rect::new(0, 0, width, height);
    let mut buffer = Buffer::empty(area);
    render(state, area, &mut buffer);
    buffer
}

pub fn render(state: &TuiState, area: Rect, buffer: &mut Buffer) {
    let view = build_view(state);
    let sections = layout_sections(area);

    Paragraph::new(view.top_bar)
        .block(top_bar_block())
        .style(Style::default().fg(WARM_SILVER).bg(GRAPHITE))
        .render(sections[0], buffer);
    Paragraph::new(view.tabs)
        .block(tab_bar_block())
        .style(Style::default().fg(WARM_SILVER).bg(GRAPHITE))
        .render(sections[1], buffer);
    body_paragraph(state, view.body, &view.body_title, sections[2]).render(sections[2], buffer);
    Paragraph::new(view.footer)
        .block(footer_block())
        .style(Style::default().fg(MUTED_SILVER).bg(GRAPHITE))
        .render(sections[3], buffer);
}

pub fn draw(frame: &mut Frame<'_>, state: &TuiState) {
    let area = frame.area();
    let view = build_view(state);
    let sections = layout_sections(area);

    frame.render_widget(
        Paragraph::new(view.top_bar)
            .block(top_bar_block())
            .style(Style::default().fg(WARM_SILVER).bg(GRAPHITE)),
        sections[0],
    );
    frame.render_widget(
        Paragraph::new(view.tabs)
            .block(tab_bar_block())
            .style(Style::default().fg(WARM_SILVER).bg(GRAPHITE)),
        sections[1],
    );
    frame.render_widget(
        body_paragraph(state, view.body, &view.body_title, sections[2]),
        sections[2],
    );
    frame.render_widget(
        Paragraph::new(view.footer)
            .block(footer_block())
            .style(Style::default().fg(MUTED_SILVER).bg(GRAPHITE)),
        sections[3],
    );
}

struct View {
    top_bar: Vec<Line<'static>>,
    tabs: Line<'static>,
    body_title: String,
    body: Vec<Line<'static>>,
    footer: Line<'static>,
}

fn build_view(state: &TuiState) -> View {
    let body_title = match state.mode {
        Mode::Search => match state.active_tab {
            Tab::MediaBrowser => "Search titles".to_string(),
            Tab::Favorites => "Favorites".to_string(),
            Tab::History => "History".to_string(),
        },
        Mode::Episodes => state
            .current_title
            .as_ref()
            .map(|title| format!("{}  Episode guide", title.name))
            .unwrap_or_else(|| "Episode guide".to_string()),
        Mode::Launching => "Launching playback".to_string(),
    };

    let body = match state.mode {
        Mode::Search => match state.active_tab {
            Tab::MediaBrowser => media_browser_lines(state),
            Tab::Favorites => favorite_lines(state),
            Tab::History => history_lines(state),
        },
        Mode::Episodes => episode_lines(state),
        Mode::Launching => vec![
            Line::from(""),
            Line::from(Span::styled(
                "Preparing your player and stream handoff.",
                Style::default().fg(MUTED_SILVER),
            )),
        ],
    };

    let status = state.message.clone().unwrap_or_else(|| "Ready".to_string());

    View {
        top_bar: vec![
            Line::from(vec![
                Span::styled(
                    "AYORU",
                    Style::default()
                        .fg(WARM_SILVER)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  ", Style::default().bg(GRAPHITE)),
                Span::styled(
                    active_tab_label(state.active_tab),
                    Style::default().fg(MIST_BLUE),
                ),
            ]),
            Line::from(vec![
                Span::styled(
                    "A quieter way to watch anime.",
                    Style::default().fg(MUTED_SILVER),
                ),
                Span::styled("  ", Style::default().bg(GRAPHITE)),
                Span::styled(search_label(state), Style::default().fg(QUIET_GRAY)),
            ]),
        ],
        tabs: tab_line(state.active_tab),
        body_title,
        body,
        footer: footer_line(state, status),
    }
}

fn layout_sections(area: Rect) -> Vec<Rect> {
    Layout::vertical([
        Constraint::Length(4),
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(3),
    ])
    .split(area)
    .to_vec()
}

fn top_bar_block() -> Block<'static> {
    Block::default()
        .borders(Borders::BOTTOM)
        .border_style(Style::default().fg(SMOKE).bg(GRAPHITE))
        .style(Style::default().bg(GRAPHITE))
        .padding(Padding::new(1, 1, 0, 0))
}

fn tab_bar_block() -> Block<'static> {
    Block::default()
        .style(Style::default().bg(GRAPHITE))
        .padding(Padding::new(1, 1, 0, 0))
}

fn footer_block() -> Block<'static> {
    Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(SMOKE).bg(GRAPHITE))
        .style(Style::default().bg(GRAPHITE))
        .padding(Padding::new(1, 1, 0, 0))
}

fn body_paragraph(
    state: &TuiState,
    lines: Vec<Line<'static>>,
    title: &str,
    area: Rect,
) -> Paragraph<'static> {
    Paragraph::new(lines)
        .block(panel_block(title))
        .scroll((body_scroll_offset(state, area), 0))
        .style(Style::default().fg(WARM_SILVER).bg(SOFT_CHARCOAL))
}

fn panel_block(title: &str) -> Block<'static> {
    Block::default()
        .title(Span::styled(
            format!(" {title} "),
            Style::default()
                .fg(WARM_SILVER)
                .add_modifier(Modifier::BOLD),
        ))
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(SMOKE).bg(SOFT_CHARCOAL))
        .style(Style::default().bg(SOFT_CHARCOAL))
        .padding(Padding::new(1, 1, 1, 0))
}

fn body_scroll_offset(state: &TuiState, area: Rect) -> u16 {
    let visible_lines = area.height.saturating_sub(3) as usize;
    if visible_lines == 0 {
        return 0;
    }

    if state.mode == Mode::Search && state.active_tab == Tab::History {
        return state.history.len().saturating_sub(visible_lines) as u16;
    }

    let selected_index = match state.mode {
        Mode::Search => match state.active_tab {
            Tab::MediaBrowser => state.selected_result,
            Tab::Favorites => state.selected_favorite,
            Tab::History => state.selected_history,
        },
        Mode::Episodes => state.selected_episode,
        Mode::Launching => 0,
    };

    selected_index.saturating_sub(visible_lines.saturating_sub(2)) as u16
}

fn media_browser_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.is_loading {
        return vec![
            Line::from(""),
            Line::from(Span::styled(
                "Searching the catalog...",
                Style::default().fg(MUTED_SILVER),
            )),
        ];
    }

    if state.results.is_empty() {
        return vec![
            Line::from(Span::styled(
                "Start with a title",
                Style::default()
                    .fg(WARM_SILVER)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Type to search, then press Enter to build your watchlist.",
                Style::default().fg(MUTED_SILVER),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Tab switches between Media Browser, Favorites, and History.",
                Style::default().fg(QUIET_GRAY),
            )),
        ];
    }

    state
        .results
        .iter()
        .enumerate()
        .map(|(idx, title)| {
            media_row(
                idx == state.selected_result,
                title.name.clone(),
                if state.search_focused {
                    "Press Enter to search".to_string()
                } else {
                    "Enter opens episodes".to_string()
                },
            )
        })
        .collect()
}

fn favorite_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.favorites.is_empty() {
        return vec![
            Line::from(Span::styled(
                "No favorites yet.",
                Style::default()
                    .fg(WARM_SILVER)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Press f on a title in Media Browser to save it here.",
                Style::default().fg(MUTED_SILVER),
            )),
        ];
    }

    state
        .favorites
        .iter()
        .enumerate()
        .map(|(idx, title)| {
            media_row(
                idx == state.selected_favorite,
                title.name.clone(),
                "Enter opens episodes  •  d remove".to_string(),
            )
        })
        .collect()
}

fn history_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.history.is_empty() {
        return vec![
            Line::from(Span::styled(
                "History is empty.",
                Style::default()
                    .fg(WARM_SILVER)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Played episodes will appear here automatically.",
                Style::default().fg(MUTED_SILVER),
            )),
        ];
    }

    state
        .history
        .iter()
        .enumerate()
        .map(|(idx, watch)| {
            media_row(
                idx == state.selected_history,
                watch.title.name.clone(),
                format!("Episode {}  •  Enter play  •  o open show", watch.episode),
            )
        })
        .collect()
}

fn episode_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.episodes.is_empty() {
        return vec![Line::from(Span::styled(
            "No episodes loaded yet.",
            Style::default().fg(MUTED_SILVER),
        ))];
    }

    state
        .episodes
        .iter()
        .enumerate()
        .map(|(idx, episode)| {
            media_row(
                idx == state.selected_episode,
                format!("Episode {}", episode.number),
                "Enter plays episode".to_string(),
            )
        })
        .collect()
}

fn media_row(selected: bool, title: String, meta: String) -> Line<'static> {
    if selected {
        Line::from(vec![
            Span::styled(" ", Style::default().bg(STEEL_BLUE)),
            Span::styled(
                title,
                Style::default()
                    .fg(WARM_SILVER)
                    .bg(STEEL_BLUE)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default().bg(STEEL_BLUE)),
            Span::styled(meta, Style::default().fg(WARM_SILVER).bg(STEEL_BLUE)),
            Span::styled(" ", Style::default().bg(STEEL_BLUE)),
        ])
    } else {
        Line::from(vec![
            Span::styled(
                title,
                Style::default()
                    .fg(WARM_SILVER)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("  ", Style::default().bg(SOFT_CHARCOAL)),
            Span::styled(meta, Style::default().fg(QUIET_GRAY)),
        ])
    }
}

fn active_tab_label(tab: Tab) -> &'static str {
    match tab {
        Tab::MediaBrowser => "Media Browser",
        Tab::Favorites => "Favorites",
        Tab::History => "History",
    }
}

fn search_label(state: &TuiState) -> String {
    if state.active_tab != Tab::MediaBrowser {
        return "Tab to switch views".to_string();
    }

    if state.query.trim().is_empty() {
        "Search titles".to_string()
    } else {
        format!("Search titles  {}", state.query)
    }
}

fn tab_line(active: Tab) -> Line<'static> {
    Line::from(vec![
        tab_chip("Media Browser", active == Tab::MediaBrowser),
        Span::styled("  ", Style::default().bg(GRAPHITE)),
        tab_chip("Favorites", active == Tab::Favorites),
        Span::styled("  ", Style::default().bg(GRAPHITE)),
        tab_chip("History", active == Tab::History),
    ])
}

fn tab_chip(label: &'static str, active: bool) -> Span<'static> {
    if active {
        Span::styled(
            format!(" {label} "),
            Style::default()
                .fg(WARM_SILVER)
                .bg(STEEL_BLUE)
                .add_modifier(Modifier::BOLD),
        )
    } else {
        Span::styled(
            format!(" {label} "),
            Style::default().fg(MUTED_SILVER).bg(SMOKE),
        )
    }
}

fn footer_line(state: &TuiState, status: String) -> Line<'static> {
    let commands = match state.mode {
        Mode::Search => match state.active_tab {
            Tab::MediaBrowser => {
                if state.search_focused {
                    "/ search   Enter search   Tab switch tabs   q quit".to_string()
                } else {
                    "Enter open   f favorite   Tab switch tabs   / search   q quit".to_string()
                }
            }
            Tab::Favorites => {
                "Enter open   d remove   f unfavorite   Tab switch tabs   q quit".to_string()
            }
            Tab::History => {
                "Enter play   o open show   d remove   D clear all   f favorite   Tab switch tabs   q quit"
                    .to_string()
            }
        },
        Mode::Episodes => "Enter play   f favorite   Esc back   q quit".to_string(),
        Mode::Launching => "Esc back   q quit".to_string(),
    };

    Line::from(vec![
        Span::styled(status, Style::default().fg(WARM_SILVER)),
        Span::styled("   ", Style::default()),
        Span::styled(commands, Style::default().fg(QUIET_GRAY)),
    ])
}
