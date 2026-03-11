use crate::tui::state::{Mode, Panel, TuiState};
use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::layout::Alignment;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

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
    let view = build_view(state, area);
    let content_sections =
        Layout::horizontal([Constraint::Percentage(68), Constraint::Percentage(32)])
            .split(view.sections[1]);
    let rail_sections = Layout::vertical([
        Constraint::Percentage(34),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ])
    .split(content_sections[1]);

    Paragraph::new(view.header)
        .block(styled_block("", state.focused_panel == Panel::Search))
        .alignment(Alignment::Left)
        .style(Style::default().fg(WARM_SILVER).bg(GRAPHITE))
        .render(view.sections[0], buffer);
    body_paragraph(state, view.body, &view.body_title, content_sections[0])
        .render(content_sections[0], buffer);
    let recent = recent_lines(state);
    rail_paragraph(
        "Recently watched",
        state.focused_panel == Panel::ContextRail,
        recent,
        rail_sections[0],
    )
    .render(rail_sections[0], buffer);
    let favorites = favorite_lines(state);
    rail_paragraph(
        "Favorites",
        state.focused_panel == Panel::ContextRail,
        favorites,
        rail_sections[1],
    )
    .render(rail_sections[1], buffer);
    let history = history_lines(state);
    rail_paragraph(
        "History",
        state.focused_panel == Panel::ContextRail,
        history,
        rail_sections[2],
    )
    .render(rail_sections[2], buffer);
    Paragraph::new(view.footer)
        .block(styled_block("Status", false))
        .style(Style::default().fg(QUIET_GRAY).bg(GRAPHITE))
        .render(view.sections[2], buffer);
}

pub fn draw(frame: &mut Frame<'_>, state: &TuiState) {
    let area = frame.area();
    let view = build_view(state, area);
    let content_sections =
        Layout::horizontal([Constraint::Percentage(68), Constraint::Percentage(32)])
            .split(view.sections[1]);
    let rail_sections = Layout::vertical([
        Constraint::Percentage(34),
        Constraint::Percentage(33),
        Constraint::Percentage(33),
    ])
    .split(content_sections[1]);
    frame.render_widget(
        Paragraph::new(view.header)
            .block(styled_block("", state.focused_panel == Panel::Search))
            .style(Style::default().fg(WARM_SILVER).bg(GRAPHITE)),
        view.sections[0],
    );
    frame.render_widget(
        body_paragraph(state, view.body, &view.body_title, content_sections[0]),
        content_sections[0],
    );
    let recent = recent_lines(state);
    frame.render_widget(
        rail_paragraph(
            "Recently watched",
            state.focused_panel == Panel::ContextRail,
            recent,
            rail_sections[0],
        ),
        rail_sections[0],
    );
    let favorites = favorite_lines(state);
    frame.render_widget(
        rail_paragraph(
            "Favorites",
            state.focused_panel == Panel::ContextRail,
            favorites,
            rail_sections[1],
        ),
        rail_sections[1],
    );
    let history = history_lines(state);
    frame.render_widget(
        rail_paragraph(
            "History",
            state.focused_panel == Panel::ContextRail,
            history,
            rail_sections[2],
        ),
        rail_sections[2],
    );
    frame.render_widget(
        Paragraph::new(view.footer)
            .block(styled_block("Status", false))
            .style(Style::default().fg(QUIET_GRAY).bg(GRAPHITE)),
        view.sections[2],
    );
}

fn search_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.results.is_empty() {
        return vec![
            Line::from(Span::styled(
                "Search, choose, watch.",
                Style::default()
                    .fg(WARM_SILVER)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(""),
            Line::from(Span::styled(
                "Type a title, then press Enter",
                Style::default().fg(QUIET_GRAY),
            )),
        ];
    }

    state
        .results
        .iter()
        .enumerate()
        .map(|(idx, title)| {
            if idx == state.selected_result {
                Line::from(vec![
                    Span::styled(
                        "> ",
                        Style::default().fg(STEEL_BLUE).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        title.name.clone(),
                        Style::default()
                            .fg(WARM_SILVER)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(Span::styled(
                    format!("  {}", title.name),
                    Style::default().fg(QUIET_GRAY),
                ))
            }
        })
        .collect()
}

fn episode_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.episodes.is_empty() {
        return vec![Line::from("No episodes loaded")];
    }

    state
        .episodes
        .iter()
        .enumerate()
        .map(|(idx, episode)| {
            let label = format!("Episode {}", episode.number);
            if idx == state.selected_episode {
                Line::from(vec![
                    Span::styled(
                        "> ",
                        Style::default().fg(STEEL_BLUE).add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        label,
                        Style::default()
                            .fg(WARM_SILVER)
                            .add_modifier(Modifier::BOLD),
                    ),
                ])
            } else {
                Line::from(Span::styled(
                    format!("  {label}"),
                    Style::default().fg(QUIET_GRAY),
                ))
            }
        })
        .collect()
}

fn favorite_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.favorites.is_empty() {
        return vec![Line::from(Span::styled(
            "No favorites yet",
            Style::default().fg(QUIET_GRAY),
        ))];
    }

    state
        .favorites
        .iter()
        .map(|title| {
            Line::from(Span::styled(
                title.name.clone(),
                Style::default().fg(QUIET_GRAY),
            ))
        })
        .collect()
}

fn recent_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.recently_watched.is_empty() {
        return vec![Line::from(Span::styled(
            "Nothing recent",
            Style::default().fg(QUIET_GRAY),
        ))];
    }

    state
        .recently_watched
        .iter()
        .map(|item| {
            Line::from(Span::styled(
                format!("{}  E{}", item.title.name, item.episode),
                Style::default().fg(QUIET_GRAY),
            ))
        })
        .collect()
}

fn history_lines(state: &TuiState) -> Vec<Line<'static>> {
    if state.history.is_empty() {
        return vec![Line::from(Span::styled(
            "No history yet",
            Style::default().fg(QUIET_GRAY),
        ))];
    }

    state
        .history
        .iter()
        .map(|item| {
            Line::from(Span::styled(
                format!("{}  E{}", item.title.name, item.episode),
                Style::default().fg(QUIET_GRAY),
            ))
        })
        .collect()
}

struct View {
    header: Vec<Line<'static>>,
    body_title: String,
    body: Vec<Line<'static>>,
    footer: String,
    sections: Vec<Rect>,
}

fn build_view(state: &TuiState, area: Rect) -> View {
    let sections = Layout::vertical([
        Constraint::Length(5),
        Constraint::Min(1),
        Constraint::Length(3),
    ])
    .split(area)
    .to_vec();

    let title = match state.mode {
        Mode::Search => "Search",
        Mode::Episodes => "Episodes",
        Mode::Launching => "Launching",
    };

    let body = match state.mode {
        Mode::Search => search_lines(state),
        Mode::Episodes => episode_lines(state),
        Mode::Launching => vec![Line::from("Launching playback...")],
    };

    let footer = match &state.message {
        Some(message) => format!("{message}  / search  Enter confirm  Esc back  q quit"),
        None => "Ready  / search  Enter confirm  Esc back  q quit".to_string(),
    };

    View {
        header: vec![
            Line::from(Span::styled(
                "AYORU",
                Style::default()
                    .fg(WARM_SILVER)
                    .add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "A quieter way to watch anime.",
                Style::default().fg(MUTED_SILVER),
            )),
            Line::from(Span::styled(
                format!("{title}  Query: {}", state.query),
                Style::default().fg(QUIET_GRAY),
            )),
        ],
        body_title: title.to_string(),
        body,
        footer,
        sections,
    }
}

fn styled_block(title: &str, active: bool) -> Block<'static> {
    let border = if active { STEEL_BLUE } else { MIST_BLUE };
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border).bg(SMOKE))
        .style(Style::default().bg(SMOKE));

    if title.is_empty() {
        block
    } else {
        block.title(Span::styled(
            title.to_string(),
            Style::default().fg(QUIET_GRAY),
        ))
    }
}

fn rail_paragraph(
    title: &str,
    active: bool,
    lines: Vec<Line<'static>>,
    area: Rect,
) -> Paragraph<'static> {
    Paragraph::new(lines.clone())
        .block(styled_block(title, active))
        .scroll((rail_scroll_offset(lines.len(), area), 0))
        .style(Style::default().fg(WARM_SILVER).bg(SOFT_CHARCOAL))
}

fn rail_scroll_offset(line_count: usize, area: Rect) -> u16 {
    let visible_lines = area.height.saturating_sub(2) as usize;
    line_count.saturating_sub(visible_lines) as u16
}

fn body_paragraph(
    state: &TuiState,
    lines: Vec<Line<'static>>,
    title: &str,
    area: Rect,
) -> Paragraph<'static> {
    Paragraph::new(lines)
        .block(styled_block(title, state.focused_panel == Panel::Main))
        .scroll((body_scroll_offset(state, area), 0))
        .style(Style::default().fg(WARM_SILVER).bg(SOFT_CHARCOAL))
}

fn body_scroll_offset(state: &TuiState, area: Rect) -> u16 {
    let visible_lines = area.height.saturating_sub(2) as usize;

    if visible_lines == 0 {
        return 0;
    }

    let selected_index = match state.mode {
        Mode::Search => state.selected_result,
        Mode::Episodes => state.selected_episode,
        Mode::Launching => 0,
    };

    selected_index.saturating_sub(visible_lines.saturating_sub(1)) as u16
}
