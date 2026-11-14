use ayoru::{
    core::models::StreamCandidate,
    session::{PlaybackSession, START_TIMEOUT},
};
use std::time::Duration;

fn stream(provider: &str) -> StreamCandidate {
    StreamCandidate {
        provider: provider.into(),
        url: format!("https://{provider}.example/episode.m3u8"),
        is_sub: true,
        resolution: None,
        referrer: None,
    }
}

#[test]
fn embedded_playback_ranks_retries_and_records_only_once() {
    let mut session = PlaybackSession::new(vec![
        stream("hianime"),
        stream("wixmp"),
        stream("wixmp"),
        stream("youtube"),
    ]);
    assert_eq!(session.next_candidate().unwrap().provider, "wixmp");
    assert!(!session.needs_fallback(0, Duration::from_secs(5)));
    assert!(session.needs_fallback(0, START_TIMEOUT));
    assert_eq!(session.next_candidate().unwrap().provider, "youtube");
    assert!(session.needs_fallback(-1, Duration::ZERO));
    assert_eq!(session.next_candidate().unwrap().provider, "hianime");
    assert!(session.confirm());
    assert!(!session.confirm());
    // A paused, already-started video must not time out.
    assert!(!session.needs_fallback(0, Duration::from_secs(60)));
    assert!(session.next_candidate().is_none());
    assert!(!session.confirm());
}

#[test]
fn playback_cannot_confirm_history_while_buffering() {
    let mut session = PlaybackSession::new(vec![stream("wixmp")]);
    session.next_candidate();
    assert!(!session.recorded);
    assert!(!session.needs_fallback(1, START_TIMEOUT));
    assert!(session.confirm());
    assert!(session.recorded);
}
