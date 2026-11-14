use ayoru::{app::ProviderRuntime, provider::allanime::AllAnimeProvider};

#[tokio::test]
#[ignore = "requires live provider access"]
async fn searches_artwork_episodes_and_playable_streams() {
    let provider = AllAnimeProvider::new();
    let titles = provider.search("Frieren").await.expect("search provider");
    let title = titles
        .iter()
        .find(|title| title.name == "Sousou no Frieren" && title.poster.is_some())
        .expect("title with artwork");
    let episodes = provider.episodes(&title.id).await.expect("episodes");
    assert!(!episodes.is_empty());
    let streams = provider
        .streams(&title.id, episodes[0].number, true)
        .await
        .expect("streams");
    assert!(!streams.is_empty(), "Provider returned no resolved streams");
    assert!(
        streams
            .iter()
            .all(|stream| stream.url.starts_with("https://"))
    );
    if let Some(path) = std::env::var_os("AYORU_LIVE_MEDIA_OUT") {
        let stream = streams
            .iter()
            .find(|stream| stream.provider == "youtube")
            .unwrap_or(&streams[0]);
        std::fs::write(
            path,
            serde_json::to_vec(&serde_json::json!({"url":stream.url,"referrer":stream.referrer}))
                .unwrap(),
        )
        .unwrap();
    }
}
