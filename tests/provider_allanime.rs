use ayoru::provider::allanime::{
    decode_source_url, parse_episodes, parse_provider_payload_links, parse_search_titles,
    parse_stream_candidates,
};

#[test]
fn parses_search_results_from_fixture() {
    let raw = std::fs::read_to_string("tests/fixtures/search.json").unwrap();
    let titles = parse_search_titles(&raw).unwrap();
    assert_eq!(titles.len(), 2);
    assert_eq!(titles[0].id, "show-1");
}

#[test]
fn parses_episode_list_and_sorts_ascending() {
    let raw = std::fs::read_to_string("tests/fixtures/episodes.json").unwrap();
    let episodes = parse_episodes(&raw).unwrap();
    assert_eq!(
        episodes.iter().map(|e| e.number).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
}

#[test]
fn parses_provider_candidates() {
    let raw = std::fs::read_to_string("tests/fixtures/streams.json").unwrap();
    let streams = parse_stream_candidates(&raw, true).unwrap();
    let providers = streams.into_iter().map(|s| s.provider).collect::<Vec<_>>();
    assert_eq!(providers, vec!["wixmp", "youtube", "sharepoint", "hianime"]);
}

#[test]
fn treats_missing_episode_sources_as_no_streams() {
    let raw = r#"{"data":{"episode":null}}"#;
    let streams = parse_stream_candidates(raw, true).unwrap();
    assert!(streams.is_empty());
}

#[test]
fn decodes_obfuscated_source_url() {
    let decoded = decode_source_url("--504c4c484b0217174c5757544b165e594b4c0c4b485d5d5c164a4b4e481717555d5c515901174e515c5d574b176a5d70757b0f6c69565b500b7b0e420052174b4d5a1709074e050a0a").unwrap();
    assert_eq!(
        decoded,
        "https://tools.fast4speed.rsvp//media9/videos/ReHMC7TQnch3C6z8j/sub/1?v=22"
    );
}

#[test]
fn extracts_links_from_clock_payload() {
    let raw = std::fs::read_to_string("tests/fixtures/clock_payload.json").unwrap();
    let links = parse_provider_payload_links(&raw);
    assert_eq!(
        links,
        vec![
            "https://cdn.example/video-1080.mp4",
            "https://cdn.example/master.m3u8"
        ]
    );
}

#[test]
fn parses_optional_poster_artwork() {
    let titles = parse_search_titles(r#"{"data":{"shows":{"edges":[{"_id":"1","name":"Frieren","thumbnail":"https://example.com/frieren.jpg"}]}}}"#).unwrap();
    assert_eq!(
        titles[0].poster.as_deref(),
        Some("https://example.com/frieren.jpg")
    );
    let legacy = parse_search_titles(include_str!("fixtures/search.json")).unwrap();
    assert_eq!(legacy[0].poster, None);
}
