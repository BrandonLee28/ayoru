use super::*;
use std::borrow::Cow;

// Adapted from Monocode theme.rs and shell/sidebar_list.rs at
// native-v0.16.0-preview.14 (MIT). See THIRD_PARTY_NOTICES.md.
const PANE: u32 = 0x0a0a0a;
const RAISED: u32 = 0x202020;
const TEXT: u32 = 0xebebeb;
const MUTED: u32 = 0x969696;
const EDGE: u32 = 0x303030;
const ACCENT: u32 = 0x4da3f5;
const SIDEBAR: f32 = 260.;

/// Small, embedded line icons keep the packaged app independent of asset paths.
pub(super) struct Assets;
impl AssetSource for Assets {
    fn load(&self, path: &str) -> gpui::Result<Option<Cow<'static, [u8]>>> {
        let copied: Option<&'static [u8]> = match path {
            "search" => Some(include_bytes!("../../assets/icons/search-01.svg")),
            "history" => Some(include_bytes!("../../assets/icons/clock.svg")),
            "heart" => Some(include_bytes!("../../assets/icons/pin.svg")),
            "play" => Some(include_bytes!("../../assets/icons/play.svg")),
            "home" => Some(include_bytes!("../../assets/icons/queue-play.svg")),
            "back" => Some(include_bytes!("../../assets/icons/arrow-left.svg")),
            _ => None,
        };
        if let Some(bytes) = copied {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        let geometry = match path {
            "home" => r#"<path d="m3 10 9-7 9 7v10H3Z"/><path d="M9 20v-7h6v7"/>"#,
            "search" => r#"<circle cx="10.5" cy="10.5" r="6.5"/><path d="m16 16 5 5"/>"#,
            "heart" => {
                r#"<path d="M20.5 5.5c-2.5-2.5-6-1.5-8.5 1-2.5-2.5-6-3.5-8.5-1S2 12 4 14l8 7 8-7c2-2 3-6 0.5-8.5Z"/>"#
            }
            "history" => r#"<path d="M3 11a9 9 0 1 1 2 7M3 4v7h7m2-5v6l4 2"/>"#,
            "play" => r#"<path d="m8 4 12 8-12 8Z"/>"#,
            "back" => r#"<path d="m14 5-7 7 7 7"/>"#,
            "close" => r#"<path d="m6 6 12 12M6 18 18 6"/>"#,
            "check" => r#"<path d="m5 12 4 4L19 6"/>"#,
            "chevron" => r#"<path d="m9 5 7 7-7 7"/>"#,
            "film" => {
                r#"<rect x="3" y="3" width="18" height="18" rx="3"/><path d="M7 3v18M17 3v18M3 8h4m10 0h4M3 16h4m10 0h4"/>"#
            }
            "sidebar" => r#"<rect x="3" y="4" width="18" height="16" rx="2"/><path d="M9 4v16"/>"#,
            "moon" => r#"<path d="M20 14a9 9 0 0 1-10-11A9 9 0 1 0 20 14Z"/>"#,
            _ => return Ok(None),
        };
        Ok(Some(Cow::Owned(format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none" stroke="white" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">{geometry}</svg>"#).into_bytes())))
    }
    fn list(&self, _: &str) -> gpui::Result<Vec<SharedString>> {
        Ok(vec![])
    }
}

// Monocode's content ink, icon and section-label helpers, with the default neutral palette.
fn content(alpha: f32) -> Hsla {
    let mut ink: Hsla = rgb(TEXT).into();
    ink.a = alpha;
    ink
}

fn icon(name: &'static str, size: f32) -> Svg {
    svg()
        .path(name)
        .size(px(size))
        .flex_none()
        .text_color(content(0.5))
}

fn button(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Stateful<Div> {
    div()
        .id(id)
        .tab_index(0)
        .cursor_pointer()
        .h(px(30.))
        .px_3()
        .flex_none()
        .flex()
        .items_center()
        .justify_center()
        .gap_2()
        .rounded_md()
        .border_1()
        .border_color(rgb(EDGE))
        .bg(rgb(RAISED))
        .text_size(px(12.))
        .text_color(rgb(TEXT))
        .hover(|s| s.bg(rgb(0x2a2a2a)).border_color(rgb(0x444444)))
        .focus(|s| s.border_color(rgb(ACCENT)))
        .child(label.into())
}

fn section_label(text: &'static str) -> Div {
    div()
        .h(px(28.))
        .px_2()
        .flex_none()
        .flex()
        .items_center()
        .gap_1()
        .text_size(px(12.))
        .font_weight(FontWeight::MEDIUM)
        .text_color(content(0.5))
        .child(div().flex_1().child(text))
}

fn empty_state(symbol: &'static str, title: &str, description: &str) -> Div {
    div()
        .w_full()
        .flex_1()
        .min_h(px(250.))
        .flex()
        .flex_col()
        .items_center()
        .justify_center()
        .gap_3()
        .child(
            div()
                .size(px(48.))
                .rounded_xl()
                .border_1()
                .border_color(rgb(EDGE))
                .flex()
                .items_center()
                .justify_center()
                .child(icon(symbol, 23.)),
        )
        .child(
            div()
                .mt_1()
                .text_size(px(18.))
                .font_weight(FontWeight::MEDIUM)
                .child(title.to_owned()),
        )
        .child(
            div()
                .max_w(px(340.))
                .text_center()
                .text_size(px(13.))
                .line_height(px(20.))
                .text_color(rgb(MUTED))
                .child(description.to_owned()),
        )
}

impl Page {
    fn icon(self) -> &'static str {
        match self {
            Self::WatchNow => "home",
            Self::Search => "search",
            Self::Favorites => "heart",
            Self::History => "history",
        }
    }
}

impl Desktop {
    fn poster(&self, title: &SavedTitle, width: f32, height: f32) -> AnyElement {
        let image = title.poster.as_ref().and_then(|url| self.artwork.get(url));
        div()
            .relative()
            .flex_none()
            .w(px(width))
            .h(px(height))
            .rounded_md()
            .overflow_hidden()
            .bg(rgb(RAISED))
            .child(
                div()
                    .size_full()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .items_center()
                    .justify_center()
                    .p_3()
                    .child(icon("film", if width > 60. { 25. } else { 16. }))
                    .when(width > 60., |view| {
                        view.child(
                            div()
                                .text_size(px(11.))
                                .text_center()
                                .text_color(rgb(MUTED))
                                .child(title.name.clone()),
                        )
                    }),
            )
            .when_some(image, |view, path| {
                view.child(
                    img(path.clone())
                        .absolute()
                        .inset_0()
                        .size_full()
                        .object_fit(ObjectFit::Cover),
                )
            })
            .into_any_element()
    }

    // Match Monocode Native's 32px navigation and 260px focused sidebar.
    fn sidebar_row(&self, page: Page, cx: &mut Context<Self>) -> AnyElement {
        let active = self.page == page;
        div()
            .id(page.label())
            .group("rail-action")
            .tab_index(0)
            .cursor_pointer()
            .h(px(32.))
            .px_2()
            .rounded_md()
            .flex()
            .items_center()
            .gap_2()
            .text_size(px(14.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(content(if active { 1. } else { 0.5 }))
            .when(active, |d| d.bg(content(0.1)))
            .hover(|s| s.bg(content(0.1)).text_color(content(1.)))
            .focus(|s| s.bg(content(0.1)))
            .child(
                icon(page.icon(), 16.)
                    .text_color(content(if active { 0.7 } else { 0.35 }))
                    .group_hover("rail-action", |s| s.text_color(content(0.7))),
            )
            .child(div().flex_1().child(page.label()))
            .on_click(cx.listener(move |view, _, window, cx| {
                view.navigate(page, cx);
                view.focus.focus(window);
            }))
            .into_any_element()
    }

    fn sidebar(&self, cx: &mut Context<Self>) -> AnyElement {
        let search = self.controls.clone();
        div()
            .w(px(SIDEBAR))
            .h_full()
            .flex_none()
            .min_h_0()
            .flex()
            .flex_col()
            .child(
                div()
                    .h(px(40.))
                    .flex_none()
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move()),
            )
            .child(
                div()
                    .h(px(39.))
                    .px_3()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_1p5()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .font_family("Menlo")
                    .child("ayoru")
                    .on_mouse_down(MouseButton::Left, |_, window, _| window.start_window_move()),
            )
            .child(
                div()
                    .px_2()
                    .pt_1()
                    .pb_3()
                    .flex_none()
                    .flex()
                    .flex_col()
                    .gap_px()
                    .child(
                        div()
                            .h(px(32.))
                            .mb_1()
                            .px_1p5()
                            .rounded_md()
                            .border_1()
                            .border_color(content(0.08))
                            .hover(|s| s.bg(content(0.1)))
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(icon("search", 16.).text_color(content(0.35)))
                            .child(
                                canvas(
                                    move |bounds, window, _| search.place(false, bounds, window),
                                    |_, _, _, _| {},
                                )
                                .flex_1()
                                .min_w_0()
                                .h(px(20.)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(content(0.4))
                                    .child("⌘K"),
                            ),
                    )
                    .children(
                        [Page::WatchNow, Page::Favorites, Page::History]
                            .into_iter()
                            .map(|page| self.sidebar_row(page, cx)),
                    ),
            )
            .child(
                div()
                    .id("sidebar-library")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .px_2()
                    .pb_3()
                    .child(
                        section_label("Favorites").child(self.library.favorites.len().to_string()),
                    )
                    .when(self.library.favorites.is_empty(), |d| {
                        d.child(
                            div()
                                .px_2()
                                .py_2()
                                .text_size(px(12.))
                                .text_color(content(0.35))
                                .child("No pinned series"),
                        )
                    })
                    .children(self.library.favorites.iter().enumerate().map(|(i, title)| {
                        let title = title.clone();
                        let active = self.selected.as_ref().is_some_and(|t| t.id == title.id);
                        div()
                            .id(("pinned", i))
                            .tab_index(0)
                            .cursor_pointer()
                            .h(px(32.))
                            .px_2()
                            .rounded_md()
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_size(px(13.))
                            .text_color(content(if active { 1. } else { 0.7 }))
                            .when(active, |d| d.bg(content(0.1)))
                            .hover(|s| s.bg(content(0.07)).text_color(content(1.)))
                            .focus(|s| s.bg(content(0.1)))
                            .child(icon("heart", 16.))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_ellipsis()
                                    .child(title.name.clone()),
                            )
                            .on_click(
                                cx.listener(move |view, _, _, cx| {
                                    view.open_title(title.clone(), cx)
                                }),
                            )
                    }))
                    .child(
                        section_label("Recently watched")
                            .mt_3()
                            .child(self.library.recently_watched.len().to_string()),
                    )
                    .when(self.library.recently_watched.is_empty(), |d| {
                        d.child(
                            div()
                                .px_2()
                                .py_2()
                                .text_size(px(12.))
                                .text_color(content(0.35))
                                .child("No recent episodes"),
                        )
                    })
                    .children(self.library.recently_watched.iter().enumerate().map(
                        |(i, watch)| {
                            let title = watch.title.clone();
                            let active = self.selected.as_ref().is_some_and(|t| t.id == title.id);
                            div()
                                .id(("recent", i))
                                .tab_index(0)
                                .cursor_pointer()
                                .px_2()
                                .py_2()
                                .mb_px()
                                .rounded_md()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .when(active, |d| d.bg(content(0.1)))
                                .hover(|s| s.bg(content(0.07)))
                                .focus(|s| s.bg(content(0.1)))
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_w_0()
                                                .text_size(px(13.))
                                                .text_ellipsis()
                                                .font_weight(FontWeight::MEDIUM)
                                                .child(title.name.clone()),
                                        )
                                        .child(
                                            div()
                                                .text_size(px(11.))
                                                .text_color(content(0.35))
                                                .child(watch_age(watch.watched_at)),
                                        ),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .items_center()
                                        .gap_1p5()
                                        .text_size(px(12.))
                                        .text_color(content(0.45))
                                        .child(icon("history", 12.))
                                        .child(format!("Episode {:02}", watch.episode)),
                                )
                                .on_click(cx.listener(move |view, _, _, cx| {
                                    view.open_title(title.clone(), cx)
                                }))
                        },
                    )),
            )
            .child(
                div()
                    .h(px(36.))
                    .px_4()
                    .flex_none()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(11.))
                    .text_color(content(0.35))
                    .child("Local library")
                    .child(div().flex_1())
                    .child(format!("v{}", env!("CARGO_PKG_VERSION"))),
            )
            .into_any_element()
    }

    fn card(
        &self,
        title: SavedTitle,
        episode: Option<u32>,
        id: usize,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let open = title.clone();
        let resume = title.clone();
        let remove = title.clone();
        let page = self.page;
        div()
            .w(px(148.))
            .flex_none()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .id(("poster", id))
                    .tab_index(0)
                    .cursor_pointer()
                    .p(px(2.))
                    .rounded_lg()
                    .border_1()
                    .border_color(rgba(0xebebeb0d))
                    .hover(|s| s.border_color(rgba(0xebebeb66)).bg(rgba(0xebebeb08)))
                    .focus(|s| s.border_color(rgb(ACCENT)))
                    .child(self.poster(&title, 142., 202.))
                    .on_click(cx.listener(move |view, _, _, cx| view.open_title(open.clone(), cx))),
            )
            .child(
                div()
                    .px_1()
                    .text_size(px(12.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_ellipsis()
                    .child(title.name.clone()),
            )
            .child(
                div()
                    .px_1()
                    .text_size(px(11.))
                    .text_color(rgb(MUTED))
                    .child(if episode.is_some() {
                        "Previously watched"
                    } else {
                        "Anime · Subtitled"
                    }),
            )
            .when_some(episode, |view, episode| {
                view.child(
                    button(("resume", id), format!("Play episode {episode}"))
                        .child(icon("play", 11.))
                        .on_click(cx.listener(move |view, _, _, cx| {
                            view.play(resume.clone(), episode, cx)
                        })),
                )
            })
            .when(matches!(page, Page::Favorites | Page::History), |view| {
                view.child(
                    button(("remove", id), "Remove")
                        .bg(rgba(0x00000000))
                        .border_color(rgba(0x00000000))
                        .text_color(rgb(MUTED))
                        .on_click(cx.listener(move |view, _, _, cx| {
                            if page == Page::Favorites {
                                view.library.favorites.retain(|t| t.id != remove.id);
                            } else {
                                view.library.remove_history_at(id);
                            }
                            view.persist();
                            cx.notify();
                        })),
                )
            })
            .into_any_element()
    }

    fn shelf(
        &self,
        label: &'static str,
        titles: Vec<(SavedTitle, Option<u32>)>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .mb_6()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(label)
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(MUTED))
                            .child(titles.len().to_string()),
                    ),
            )
            .child(
                div()
                    .id(label)
                    .flex()
                    .gap_4()
                    .pb_2()
                    .overflow_x_scroll()
                    .children(
                        titles
                            .into_iter()
                            .enumerate()
                            .map(|(id, (title, episode))| self.card(title, episode, id, cx)),
                    ),
            )
            .into_any_element()
    }

    fn browsing(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut content = div().flex_1().min_h_0().flex().flex_col();
        match self.page {
            Page::WatchNow => {
                let empty =
                    self.library.recently_watched.is_empty() && self.library.favorites.is_empty();
                if empty {
                    content = content.child(empty_state("moon", "A little space for your anime.", "Search for a series to get started. Your favorites and recent episodes will be right here.")
                        .child(button("get-started", "Find a series").mt_2().bg(rgb(TEXT)).text_color(rgb(PANE)).hover(|s| s.bg(rgb(0xffffff)))
                            .on_click(cx.listener(|view, _, _, cx| { view.navigate(Page::Search, cx); view.controls.focus_search(); }))))
                        .child(div().py_6().flex().justify_center().gap_2().text_size(px(11.)).text_color(rgb(MUTED))
                            .child("Search titles").child("·").child("Save favorites").child("·").child("Watch in Ayoru"));
                } else {
                    content = content.child(
                        div()
                            .id("home-shelves")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .p_6()
                            .child(
                                div()
                                    .text_size(px(20.))
                                    .font_weight(FontWeight::MEDIUM)
                                    .mb_1()
                                    .child("Watch now"),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(rgb(MUTED))
                                    .mb_6()
                                    .child("Pick up where you left off, or find something new."),
                            )
                            .when(!self.library.recently_watched.is_empty(), |view| {
                                view.child(
                                    self.shelf(
                                        "Recently watched",
                                        self.library
                                            .recently_watched
                                            .iter()
                                            .map(|w| (w.title.clone(), Some(w.episode)))
                                            .collect(),
                                        cx,
                                    ),
                                )
                            })
                            .when(!self.library.favorites.is_empty(), |view| {
                                view.child(
                                    self.shelf(
                                        "Favorites",
                                        self.library
                                            .favorites
                                            .iter()
                                            .map(|t| (t.clone(), None))
                                            .collect(),
                                        cx,
                                    ),
                                )
                            }),
                    );
                }
            }
            Page::Search => {
                if self.results.is_empty() {
                    content = content.child(if self.loading {
                        empty_state("search", "Searching…", "Looking for matching titles.")
                    } else if self.query.is_empty() {
                        empty_state(
                            "search",
                            "Find something to watch",
                            "Enter an anime title in the search field in the sidebar and press Return.",
                        )
                        .child(
                            button("focus-search", "Search anime  ⌘K").mt_2().on_click(
                                cx.listener(|view, _, _, _| view.controls.focus_search()),
                            ),
                        )
                    } else {
                        empty_state(
                            "search",
                            "No titles found",
                            "Try another title or a shorter search.",
                        )
                    });
                } else {
                    content =
                        content.child(
                            div()
                                .id("search-results")
                                .flex_1()
                                .min_h_0()
                                .overflow_y_scroll()
                                .p_6()
                                .child(
                                    div()
                                        .mb_5()
                                        .flex()
                                        .items_center()
                                        .gap_2()
                                        .text_size(px(13.))
                                        .child(format!("Results for “{}”", self.query))
                                        .child(
                                            div()
                                                .text_size(px(11.))
                                                .text_color(rgb(MUTED))
                                                .child(format!("{} titles", self.results.len())),
                                        ),
                                )
                                .child(div().flex().flex_wrap().gap_4().children(
                                    self.results.iter().enumerate().map(|(id, title)| {
                                        self.card(saved_title(title), None, id, cx)
                                    }),
                                )),
                        );
                }
            }
            Page::Favorites | Page::History => {
                let titles: Vec<_> = if self.page == Page::Favorites {
                    self.library
                        .favorites
                        .iter()
                        .map(|t| (t.clone(), None))
                        .collect()
                } else {
                    self.library
                        .history
                        .iter()
                        .map(|w| (w.title.clone(), Some(w.episode)))
                        .collect()
                };
                if titles.is_empty() {
                    content = content.child(empty_state(self.page.icon(), if self.page == Page::Favorites { "Keep your favorites close" } else { "Your watch history" },
                        if self.page == Page::Favorites { "Save a series from its title page. It will be waiting for you here." } else { "Episodes appear here after playback starts. You can replay them at any time." }));
                } else {
                    content = content.child(
                        div()
                            .id("library-grid")
                            .flex_1()
                            .min_h_0()
                            .overflow_y_scroll()
                            .p_6()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .mb_5()
                                    .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(
                                        format!(
                                            "{} {}",
                                            titles.len(),
                                            if self.page == Page::Favorites {
                                                "saved titles"
                                            } else {
                                                "watched episodes"
                                            }
                                        ),
                                    ))
                                    .when(self.page == Page::History, |view| {
                                        view.child(
                                            button("clear-history", "Clear history").on_click(
                                                cx.listener(|view, _, _, cx| {
                                                    view.library.clear_history();
                                                    view.persist();
                                                    cx.notify();
                                                }),
                                            ),
                                        )
                                    }),
                            )
                            .child(
                                div().flex().flex_wrap().gap_4().children(
                                    titles
                                        .into_iter()
                                        .enumerate()
                                        .map(|(id, (title, episode))| {
                                            self.card(title, episode, id, cx)
                                        }),
                                ),
                            ),
                    );
                }
            }
        }
        content.into_any_element()
    }

    fn episodes_panel(&self, title: &SavedTitle, cx: &mut Context<Self>) -> AnyElement {
        let compact = self.episode.is_some();
        div()
            .w_full()
            .max_w(px(880.))
            .mx_auto()
            .p_6()
            .font_family("Menlo")
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div()
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .gap_3()
                    .text_size(px(11.))
                    .text_color(content(0.5))
                    .child("EPISODES")
                    .child(div().flex_1())
                    .child(format!("{} available / SUB", self.episodes.len())),
            )
            .when(self.episodes.is_empty(), |d| {
                d.child(
                    div()
                        .py_3()
                        .text_size(px(12.))
                        .text_color(content(0.45))
                        .child(if self.loading {
                            "Loading episodes…"
                        } else {
                            "No episodes available."
                        }),
                )
            })
            .child(
                div().flex().flex_wrap().gap_1().children(
                    self.episodes
                        .iter()
                        .take(if self.episodes_expanded || compact {
                            usize::MAX
                        } else {
                            12
                        })
                        .map(|episode| {
                            let number = episode.number;
                            let title = title.clone();
                            let active = self.selected_episode == Some(number);
                            let watched = self
                                .library
                                .history
                                .iter()
                                .any(|w| w.title.id == title.id && w.episode == number);
                            div()
                                .id(("episode", number as usize))
                                .tab_index(0)
                                .cursor_pointer()
                                .w(if compact {
                                    px(58.).into()
                                } else {
                                    relative(1.)
                                })
                                .h(px(36.))
                                .px_3()
                                .rounded_sm()
                                .border_1()
                                .border_color(if active { content(0.2) } else { content(0.04) })
                                .bg(content(if active { 0.08 } else { 0.01 }))
                                .hover(|s| s.bg(content(0.08)))
                                .focus(|s| s.border_color(content(0.5)))
                                .flex()
                                .items_center()
                                .gap_3()
                                .text_size(px(12.))
                                .child(
                                    div()
                                        .text_color(content(if active { 1. } else { 0.5 }))
                                        .child(format!("{number:02}")),
                                )
                                .when(!compact, |d| {
                                    d.child(
                                        div()
                                            .flex_1()
                                            .text_color(content(0.8))
                                            .child(format!("Episode {number:02}")),
                                    )
                                    .child(
                                        div().text_size(px(10.)).text_color(content(0.35)).child(
                                            if watched {
                                                "WATCHED"
                                            } else if active {
                                                "SELECTED"
                                            } else {
                                                ""
                                            },
                                        ),
                                    )
                                    .child(if watched {
                                        icon("check", 13.)
                                    } else {
                                        icon("play", 13.)
                                    })
                                })
                                .on_click(cx.listener(move |view, _, _, cx| {
                                    view.selected_episode = Some(number);
                                    if compact {
                                        view.play(title.clone(), number, cx);
                                    } else {
                                        cx.notify();
                                    }
                                }))
                        }),
                ),
            )
            .when(!compact && self.episodes.len() > 12, |d| {
                d.child(
                    button(
                        "expand-episodes",
                        if self.episodes_expanded {
                            "Show fewer episodes".into()
                        } else {
                            format!("Show all {} episodes", self.episodes.len())
                        },
                    )
                    .w(px(210.))
                    .mt_2()
                    .border_0()
                    .bg(rgba(0))
                    .text_color(content(0.5))
                    .on_click(cx.listener(|view, _, _, cx| {
                        view.episodes_expanded = !view.episodes_expanded;
                        cx.notify();
                    })),
                )
            })
            .into_any_element()
    }

    fn details(&self, window: &Window, cx: &mut Context<Self>) -> AnyElement {
        let title = self.selected.as_ref().unwrap().clone();
        let start = self
            .selected_episode
            .filter(|n| self.episodes.iter().any(|e| e.number == *n));
        let controls = self.controls.clone();
        let height = ((f32::from(window.viewport_size().width)
            - if self.sidebar_visible { SIDEBAR } else { 0. }
            - 8.)
            * 9.
            / 16.)
            .min(f32::from(window.viewport_size().height) * 0.56)
            .max(180.);
        let play_title = title.clone();
        let replay = start.is_some_and(|n| {
            self.library
                .history
                .iter()
                .any(|w| w.title.id == title.id && w.episode == n)
        });
        let heading = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_3()
            .font_family("Menlo")
            .child(
                div()
                    .text_size(px(10.))
                    .text_color(content(0.4))
                    .child("SERIES / SUBTITLED"),
            )
            .child(
                div()
                    .text_size(px(23.))
                    .line_height(px(32.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(title.name.clone()),
            )
            .child(div().text_size(px(11.)).text_color(content(0.45)).child(
                if self.loading && self.episodes.is_empty() {
                    "Loading episodes…".into()
                } else {
                    format!("{} episodes", self.episodes.len())
                },
            ))
            .when_some(start, |d, n| {
                d.child(
                    button(
                        "play-title",
                        format!("{} episode {n:02}", if replay { "Replay" } else { "Play" }),
                    )
                    .w(px(210.))
                    .mt_2()
                    .h(px(32.))
                    .bg(rgb(TEXT))
                    .text_color(rgb(PANE))
                    .hover(|s| s.bg(rgb(0xffffff)))
                    .child(icon("play", 13.).text_color(rgb(PANE)))
                    .on_click(
                        cx.listener(move |view, _, _, cx| view.play(play_title.clone(), n, cx)),
                    ),
                )
            });
        let overview = div()
            .w_full()
            .max_w(px(880.))
            .mx_auto()
            .p_6()
            .pb_4()
            .flex()
            .items_start()
            .gap_6()
            .child(self.poster(&title, 112., 160.))
            .child(heading);
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .when(self.episode.is_some(), |d| {
                d.child(
                    canvas(
                        move |bounds, window, _| controls.place(true, bounds, window),
                        |_, _, _, _| {},
                    )
                    .w_full()
                    .h(px(height))
                    .flex_none(),
                )
                .child(
                    div()
                        .px_6()
                        .py_3()
                        .flex_none()
                        .flex()
                        .items_center()
                        .gap_3()
                        .border_b_1()
                        .border_color(content(0.07))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap_1()
                                .child(
                                    div()
                                        .text_ellipsis()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(title.name.clone()),
                                )
                                .child(
                                    div()
                                        .font_family("Menlo")
                                        .text_size(px(10.))
                                        .text_color(rgb(MUTED))
                                        .child(format!(
                                            "Episode {:02} · SUB",
                                            self.episode.unwrap_or(1)
                                        )),
                                ),
                        )
                        .child(button("retry", "Retry playback").on_click(cx.listener(
                            |view, _, _, cx| {
                                if let (Some(t), Some(n)) = (view.selected.clone(), view.episode) {
                                    view.play(t, n, cx);
                                }
                            },
                        ))),
                )
            })
            .child(
                div()
                    .id("series-content")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .when(self.episode.is_none(), |d| d.child(overview))
                    .child(self.episodes_panel(&title, cx)),
            )
            .into_any_element()
    }
}

fn watch_age(timestamp: u64) -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .saturating_sub(timestamp);
    match seconds {
        0..=59 => "now".into(),
        60..=3599 => format!("{}m", seconds / 60),
        3600..=86399 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    }
}

impl Render for Desktop {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body = if self.selected.is_some() {
            self.details(window, cx)
        } else {
            self.browsing(cx)
        };
        let selected = self.selected.clone();
        div()
            .size_full()
            .flex()
            .bg(rgba(0x171717d9))
            .text_color(rgb(TEXT))
            .font_family(".SystemUIFont")
            .text_size(px(13.))
            .track_focus(&self.focus)
            .on_action(cx.listener(Self::back))
            .on_action(|_: &FocusNext, window, _| window.focus_next())
            .on_action(|_: &FocusPrevious, window, _| window.focus_prev())
            .on_action(cx.listener(|view, _: &Search, _, cx| {
                view.sidebar_visible = true;
                view.controls.show_search(true);
                view.controls.focus_search();
                cx.notify();
            }))
            .on_action(cx.listener(Self::toggle_sidebar))
            .on_action(|_: &CloseWindow, window, _| window.remove_window())
            .when(self.sidebar_visible, |d| d.child(self.sidebar(cx)))
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .ml(px(if self.sidebar_visible { 0. } else { 6. }))
                    .mt(px(40.))
                    .mr(px(6.))
                    .mb(px(6.))
                    .rounded(px(10.))
                    .overflow_hidden()
                    .border_1()
                    .border_color(content(0.08))
                    .bg(rgb(PANE))
                    .child(
                        div()
                            .h(px(36.))
                            .px_4()
                            .flex_none()
                            .flex()
                            .items_center()
                            .gap_3()
                            .border_b_1()
                            .border_color(content(0.07))
                            .on_mouse_down(MouseButton::Left, |_, window, _| {
                                window.start_window_move()
                            })
                            .child(
                                button("toggle-sidebar", "")
                                    .size(px(24.))
                                    .px_0()
                                    .border_0()
                                    .bg(rgba(0))
                                    .child(icon("sidebar", 16.))
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation()
                                    })
                                    .on_click(cx.listener(|view, _, window, cx| {
                                        view.toggle_sidebar(&ToggleSidebar, window, cx);
                                    })),
                            )
                            .when(selected.is_some(), |d| {
                                d.child(
                                    button("back", "")
                                        .size(px(24.))
                                        .px_0()
                                        .border_0()
                                        .bg(rgba(0))
                                        .child(icon("back", 15.))
                                        .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                            cx.stop_propagation()
                                        })
                                        .on_click(cx.listener(|view, _, window, cx| {
                                            view.back(&Back, window, cx)
                                        })),
                                )
                            })
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(content(0.5))
                                    .child(self.page.label()),
                            )
                            .when(selected.is_some(), |d| {
                                d.child(div().text_color(content(0.25)).child("/"))
                            })
                            .when(selected.is_some(), |d| {
                                d.child(
                                    div()
                                        .text_size(px(13.))
                                        .text_color(content(0.75))
                                        .child("Series"),
                                )
                            })
                            .child(div().flex_1())
                            .when(self.loading, |d| {
                                d.child(div().size(px(5.)).rounded_full().bg(rgb(ACCENT)))
                            })
                            .when_some(selected, |d, title| {
                                let pinned =
                                    self.library.favorites.iter().any(|t| t.id == title.id);
                                d.child(
                                    button(
                                        "favorite",
                                        if pinned { "Pinned" } else { "Pin series" },
                                    )
                                    .h(px(26.))
                                    .border_0()
                                    .bg(rgba(0))
                                    .child(
                                        icon("heart", 14.)
                                            .when(pinned, |i| i.text_color(rgb(ACCENT))),
                                    )
                                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                                        cx.stop_propagation()
                                    })
                                    .on_click(cx.listener(
                                        move |view, _, _, cx| {
                                            view.library.toggle_favorite(title.clone());
                                            view.persist();
                                            cx.notify();
                                        },
                                    )),
                                )
                            }),
                    )
                    .when_some(self.message.clone(), |d, message| {
                        d.child(
                            div()
                                .flex_none()
                                .px_6()
                                .py_3()
                                .border_b_1()
                                .border_color(content(0.07))
                                .text_size(px(12.))
                                .line_height(px(18.))
                                .text_color(rgb(MUTED))
                                .child(message),
                        )
                    })
                    .child(body),
            )
    }
}
