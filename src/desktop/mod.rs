mod native;
mod ui;

use crate::{
    app::ProviderRuntime,
    core::models::{Episode, StreamCandidate, Title},
    library::{
        LibraryState, SavedTitle, SavedWatch,
        storage::{LibraryStorage, default_library_path},
    },
    provider::allanime::AllAnimeProvider,
    session::PlaybackSession,
};
use gpui::{prelude::*, *};
use native::Controls;
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::PathBuf,
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

actions!(
    ayoru,
    [
        Quit,
        Back,
        Search,
        ToggleSidebar,
        CloseWindow,
        FocusNext,
        FocusPrevious,
        Cut,
        Copy,
        Paste,
        SelectAll
    ]
);

#[derive(Clone, Copy, PartialEq)]
enum Page {
    WatchNow,
    Search,
    Favorites,
    History,
}

impl Page {
    fn label(self) -> &'static str {
        match self {
            Self::WatchNow => "Watch Now",
            Self::Search => "Search",
            Self::Favorites => "Favorites",
            Self::History => "History",
        }
    }
}

enum Reply {
    Search(u64, Result<Vec<Title>, String>),
    Episodes(u64, Result<Vec<Episode>, String>),
    Streams(u64, Result<Vec<StreamCandidate>, String>),
    Artwork(String, PathBuf),
    ApiStatus(bool),
}

struct Desktop {
    runtime: tokio::runtime::Runtime,
    sender: Sender<Reply>,
    receiver: Receiver<Reply>,
    job: Option<tokio::task::JoinHandle<()>>,
    generation: u64,
    controls: Rc<Controls>,
    focus: FocusHandle,
    page: Page,
    sidebar_visible: bool,
    api_online: Option<bool>,
    query: String,
    results: Vec<Title>,
    selected: Option<SavedTitle>,
    episodes: Vec<Episode>,
    episode: Option<u32>,
    selected_episode: Option<u32>,
    episodes_expanded: bool,
    session: Option<PlaybackSession>,
    attempted_at: Instant,
    loading: bool,
    message: Option<String>,
    library: LibraryState,
    storage: LibraryStorage,
    storage_healthy: bool,
    artwork: HashMap<String, PathBuf>,
    _ticker: Task<()>,
}

impl Desktop {
    fn new(cx: &mut Context<Self>) -> Self {
        let (sender, receiver) = mpsc::channel();
        let storage = LibraryStorage::new(default_library_path());
        let loaded = storage.load();
        let storage_healthy = loaded.is_ok();
        let message = loaded.as_ref().err().map(|e| format!("Could not read your library: {e}. Your file has been preserved; restart after fixing it."));
        let library = loaded.unwrap_or_default();
        let ticker = cx.spawn(async |view, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(100))
                    .await;
                if view.update(cx, |view, cx| view.tick(cx)).is_err() {
                    break;
                }
            }
        });
        let app = Self {
            runtime: tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("network runtime"),
            sender,
            receiver,
            job: None,
            generation: 0,
            controls: Rc::new(Controls::new()),
            focus: cx.focus_handle(),
            page: Page::WatchNow,
            sidebar_visible: true,
            api_online: None,
            query: String::new(),
            results: vec![],
            selected: None,
            episodes: vec![],
            episode: None,
            selected_episode: None,
            episodes_expanded: false,
            session: None,
            attempted_at: Instant::now(),
            loading: false,
            message,
            library,
            storage,
            storage_healthy,
            artwork: HashMap::new(),
            _ticker: ticker,
        };
        let titles = app
            .library
            .favorites
            .iter()
            .chain(app.library.recently_watched.iter().map(|w| &w.title))
            .cloned()
            .collect();
        app.fetch_artwork(titles);
        let sender = app.sender.clone();
        app.runtime.spawn(async move {
            let provider = AllAnimeProvider::new();
            loop {
                let online = matches!(
                    tokio::time::timeout(Duration::from_secs(10), provider.check_health()).await,
                    Ok(Ok(()))
                );
                if sender.send(Reply::ApiStatus(online)).is_err() {
                    break;
                }
                tokio::time::sleep(Duration::from_secs(60)).await;
            }
        });
        app
    }

    fn fetch_artwork(&self, titles: Vec<SavedTitle>) {
        let sender = self.sender.clone();
        self.runtime.spawn(async move {
            let Ok(client) = reqwest::Client::builder()
                .timeout(Duration::from_secs(12))
                .build()
            else {
                return;
            };
            for title in titles {
                let Some(url) = title.poster else {
                    continue;
                };
                if !url.starts_with("https://") {
                    continue;
                }
                let path = artwork_path(&url);
                if path.is_file() {
                    let _ = sender.send(Reply::Artwork(url, path));
                    continue;
                }
                let Ok(mut response) = client
                    .get(&url)
                    .send()
                    .await
                    .and_then(|r| r.error_for_status())
                else {
                    continue;
                };
                if response
                    .content_length()
                    .is_some_and(|n| n > 8 * 1024 * 1024)
                {
                    continue;
                }
                let mut bytes = Vec::new();
                let mut complete = false;
                while let Ok(chunk) = response.chunk().await {
                    let Some(chunk) = chunk else {
                        complete = true;
                        break;
                    };
                    bytes.extend_from_slice(&chunk);
                    if bytes.len() > 8 * 1024 * 1024 {
                        break;
                    }
                }
                if !complete || bytes.is_empty() {
                    continue;
                }
                if let Some(parent) = path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if std::fs::write(&path, &bytes).is_ok() {
                    let _ = sender.send(Reply::Artwork(url, path));
                }
            }
        });
    }

    fn cancel_request(&mut self) -> u64 {
        if let Some(job) = self.job.take() {
            job.abort();
        }
        self.generation += 1;
        self.loading = false;
        self.generation
    }

    fn stop_video(&mut self) {
        self.controls.stop();
        self.session = None;
        self.episode = None;
    }

    fn navigate(&mut self, page: Page, cx: &mut Context<Self>) {
        self.cancel_request();
        self.stop_video();
        self.page = page;
        self.selected = None;
        self.episodes.clear();
        if self.storage_healthy {
            self.message = None;
        }
        cx.notify();
    }

    fn search(&mut self, query: String, cx: &mut Context<Self>) {
        self.navigate(Page::Search, cx);
        self.query = query.trim().to_owned();
        self.results.clear();
        if self.query.is_empty() {
            return;
        }
        self.loading = true;
        let generation = self.generation;
        let query = self.query.clone();
        let sender = self.sender.clone();
        self.job = Some(self.runtime.spawn(async move {
            let result = AllAnimeProvider::new().search(&query).await;
            let _ = sender.send(Reply::Search(generation, result));
        }));
    }

    fn open_title(&mut self, title: SavedTitle, cx: &mut Context<Self>) {
        let generation = self.cancel_request();
        self.stop_video();
        self.selected_episode = self
            .library
            .recently_watched
            .iter()
            .find(|w| w.title.id == title.id)
            .map(|w| w.episode);
        self.episodes_expanded = false;
        self.selected = Some(title.clone());
        self.episodes.clear();
        self.loading = true;
        if self.storage_healthy {
            self.message = None;
        }
        let sender = self.sender.clone();
        self.job = Some(self.runtime.spawn(async move {
            let result = AllAnimeProvider::new().episodes(&title.id).await;
            let _ = sender.send(Reply::Episodes(generation, result));
        }));
        cx.notify();
    }

    fn play(&mut self, title: SavedTitle, episode: u32, cx: &mut Context<Self>) {
        let load_episodes = self
            .selected
            .as_ref()
            .is_none_or(|selected| selected.id != title.id)
            || self.episodes.is_empty();
        let generation = self.cancel_request();
        self.stop_video();
        if load_episodes {
            self.episodes.clear();
        }
        self.selected = Some(title.clone());
        self.episode = Some(episode);
        self.selected_episode = Some(episode);
        self.loading = true;
        self.message = Some(format!("Finding streams for episode {episode}…"));
        let sender = self.sender.clone();
        self.job = Some(self.runtime.spawn(async move {
            let provider = AllAnimeProvider::new();
            if load_episodes {
                let result = provider.episodes(&title.id).await;
                let _ = sender.send(Reply::Episodes(generation, result));
            }
            let result = provider.streams(&title.id, episode, true).await;
            let _ = sender.send(Reply::Streams(generation, result));
        }));
        cx.notify();
    }

    fn try_next_stream(&mut self) {
        self.controls.stop();
        while let Some(stream) = self
            .session
            .as_mut()
            .and_then(PlaybackSession::next_candidate)
        {
            if self
                .controls
                .load(&stream.url, stream.referrer.as_deref())
                .is_ok()
            {
                self.attempted_at = Instant::now();
                self.loading = true;
                self.message = Some(format!(
                    "Starting episode {} · {}…",
                    self.episode.unwrap_or(1),
                    stream.provider
                ));
                return;
            }
        }
        self.session = None;
        self.loading = false;
        self.message = Some(
            "Playback failed after trying all providers. Try another episode or retry this one."
                .into(),
        );
    }

    fn persist(&mut self) {
        if !self.storage_healthy {
            self.message = Some("Your library could not be read. Changes cannot be saved until the file is repaired.".into());
        } else if let Err(error) = self.storage.save(&self.library) {
            self.message = Some(format!(
                "Could not save library: {error}. Changes remain in memory."
            ));
        }
    }

    fn tick(&mut self, cx: &mut Context<Self>) {
        if let Some(query) = self.controls.take_search() {
            self.search(query, cx);
        }
        while let Ok(reply) = self.receiver.try_recv() {
            match reply {
                Reply::Search(generation, result) if generation == self.generation => {
                    self.loading = false;
                    match result {
                        Ok(titles) => {
                            self.fetch_artwork(titles.iter().map(saved_title).collect());
                            self.results = titles;
                        }
                        Err(error) => self.message = Some(error),
                    }
                }
                Reply::Episodes(generation, result) if generation == self.generation => {
                    if self.episode.is_none() {
                        self.loading = false;
                    }
                    match result {
                        Ok(episodes) => {
                            self.selected_episode =
                                preferred_episode(&episodes, self.selected_episode);
                            self.episodes = episodes;
                        }
                        Err(error) => self.message = Some(error),
                    }
                }
                Reply::Streams(generation, result) if generation == self.generation => match result
                {
                    Ok(streams) if !streams.is_empty() => {
                        self.session = Some(PlaybackSession::new(streams));
                        self.try_next_stream();
                    }
                    Ok(_) => {
                        self.loading = false;
                        self.message = Some("No playable streams found.".into());
                    }
                    Err(error) => {
                        self.loading = false;
                        self.message = Some(error);
                    }
                },
                Reply::ApiStatus(online) => {
                    self.api_online = Some(online);
                }
                Reply::Artwork(url, path) => {
                    self.artwork.insert(url, path);
                }
                _ => continue,
            }
            cx.notify();
        }
        if self.session.is_some() {
            let status = self.controls.status();
            if status == 1 {
                let session = self.session.as_mut().unwrap();
                let first_confirmation = !session.confirmed;
                if session.confirm()
                    && let (Some(title), Some(episode)) = (&self.selected, self.episode)
                {
                    self.library.record_watch(SavedWatch {
                        title: title.clone(),
                        episode,
                        watched_at: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs(),
                    });
                    self.persist();
                }
                if first_confirmation {
                    self.loading = false;
                    // Preserve a storage error if saving failed.
                    if self
                        .message
                        .as_ref()
                        .is_some_and(|m| m.starts_with("Starting episode"))
                    {
                        self.message = None;
                    }
                    cx.notify();
                }
            } else if self
                .session
                .as_ref()
                .unwrap()
                .needs_fallback(status, self.attempted_at.elapsed())
            {
                let error = self.controls.error();
                self.try_next_stream();
                if self.session.is_none() {
                    self.message = Some(format!(
                        "Playback failed after trying all providers. {error}"
                    ));
                }
                cx.notify();
            } else if status == 2 && !self.loading {
                self.message = Some("Episode finished. Choose another episode below.".into());
                self.session = None;
                cx.notify();
            }
        }
    }

    fn toggle_sidebar(&mut self, _: &ToggleSidebar, window: &mut Window, cx: &mut Context<Self>) {
        self.sidebar_visible = !self.sidebar_visible;
        self.controls.show_search(self.sidebar_visible);
        self.focus.focus(window);
        cx.notify();
    }

    fn back(&mut self, _: &Back, window: &mut Window, cx: &mut Context<Self>) {
        self.cancel_request();
        if self.episode.is_some() {
            let title = self.selected.clone();
            self.stop_video();
            if let Some(title) = title {
                self.open_title(title, cx);
            }
        } else {
            self.selected = None;
        }
        self.focus.focus(window);
        cx.notify();
    }
}

fn saved_title(title: &Title) -> SavedTitle {
    SavedTitle {
        id: title.id.clone(),
        name: title.name.clone(),
        poster: title.poster.clone(),
    }
}

fn artwork_path(url: &str) -> PathBuf {
    let mut hash = std::collections::hash_map::DefaultHasher::new();
    url.hash(&mut hash);
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("Library/Caches/ayoru/artwork")
        .join(format!("{:x}", hash.finish()))
}

pub fn run() {
    Application::new()
        .with_assets(ui::Assets)
        .run(|cx: &mut App| {
            native::set_app_icon();
            cx.bind_keys([
                KeyBinding::new("cmd-q", Quit, None),
                KeyBinding::new("cmd-w", CloseWindow, None),
                KeyBinding::new("cmd-f", Search, None),
                KeyBinding::new("cmd-k", Search, None),
                KeyBinding::new("cmd-b", ToggleSidebar, None),
                KeyBinding::new("escape", Back, None),
                KeyBinding::new("tab", FocusNext, None),
                KeyBinding::new("shift-tab", FocusPrevious, None),
                KeyBinding::new("cmd-x", Cut, None),
                KeyBinding::new("cmd-c", Copy, None),
                KeyBinding::new("cmd-v", Paste, None),
                KeyBinding::new("cmd-a", SelectAll, None),
            ]);
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.set_menus(vec![
                Menu {
                    name: "ayoru".into(),
                    items: vec![MenuItem::action("Quit ayoru", Quit)],
                },
                Menu {
                    name: "Edit".into(),
                    items: vec![
                        MenuItem::os_action("Cut", Cut, OsAction::Cut),
                        MenuItem::os_action("Copy", Copy, OsAction::Copy),
                        MenuItem::os_action("Paste", Paste, OsAction::Paste),
                        MenuItem::os_action("Select All", SelectAll, OsAction::SelectAll),
                    ],
                },
                Menu {
                    name: "Navigate".into(),
                    items: vec![
                        MenuItem::action("Search", Search),
                        MenuItem::action("Back", Back),
                        MenuItem::action("Close Window", CloseWindow),
                    ],
                },
            ]);
            cx.on_window_closed(|cx| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
            let bounds = Bounds::centered(None, size(px(1240.), px(820.)), cx);
            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(860.), px(620.))),
                    window_background: WindowBackgroundAppearance::Blurred,
                    titlebar: Some(TitlebarOptions {
                        title: Some("ayoru".into()),
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(16.), px(17.))),
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    cx.activate(true);
                    cx.new(|cx| {
                        let app = Desktop::new(cx);
                        app.focus.focus(window);
                        app
                    })
                },
            )
            .expect("open ayoru window");
        });
}

// A saved episode can disappear from a provider's current list.
fn preferred_episode(episodes: &[Episode], saved: Option<u32>) -> Option<u32> {
    saved
        .filter(|n| episodes.iter().any(|e| e.number == *n))
        .or_else(|| episodes.first().map(|e| e.number))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[::core::prelude::v1::test]
    fn episode_selection_uses_available_history_or_first_episode() {
        let episodes = vec![Episode { number: 2 }, Episode { number: 4 }];
        assert_eq!(preferred_episode(&episodes, Some(4)), Some(4));
        assert_eq!(preferred_episode(&episodes, Some(3)), Some(2));
        assert_eq!(preferred_episode(&episodes, None), Some(2));
        assert_eq!(preferred_episode(&[], Some(4)), None);
    }
}
