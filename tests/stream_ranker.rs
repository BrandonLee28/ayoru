use ayoru::core::models::StreamCandidate;
use ayoru::core::stream_ranker::rank_streams;

#[test]
fn ranks_by_provider_reliability_first() {
    let mut streams = vec![
        StreamCandidate {
            provider: "hianime".to_string(),
            url: "u1".to_string(),
            is_sub: true,
            resolution: Some(1080),
            referrer: None,
        },
        StreamCandidate {
            provider: "wixmp".to_string(),
            url: "u2".to_string(),
            is_sub: true,
            resolution: Some(720),
            referrer: None,
        },
    ];

    rank_streams(&mut streams);
    assert_eq!(streams[0].provider, "wixmp");
}

#[test]
fn prefers_subtitles_before_resolution() {
    let mut streams = vec![
        StreamCandidate {
            provider: "youtube".to_string(),
            url: "u1".to_string(),
            is_sub: false,
            resolution: Some(1080),
            referrer: None,
        },
        StreamCandidate {
            provider: "youtube".to_string(),
            url: "u2".to_string(),
            is_sub: true,
            resolution: Some(720),
            referrer: None,
        },
    ];

    rank_streams(&mut streams);
    assert!(streams[0].is_sub);
}
