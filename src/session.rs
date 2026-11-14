use crate::core::{models::StreamCandidate, stream_ranker::rank_streams};
use std::collections::{HashSet, VecDeque};
use std::time::Duration;

pub const START_TIMEOUT: Duration = Duration::from_secs(6);

/// A single playback request tries each provider at most once, in ranked order.
pub struct PlaybackSession {
    remaining: VecDeque<StreamCandidate>,
    pub recorded: bool,
    pub confirmed: bool,
}

impl PlaybackSession {
    pub fn new(mut streams: Vec<StreamCandidate>) -> Self {
        rank_streams(&mut streams);
        let mut seen = HashSet::new();
        streams.retain(|stream| seen.insert(stream.provider.clone()));
        Self {
            remaining: streams.into(),
            recorded: false,
            confirmed: false,
        }
    }

    pub fn next_candidate(&mut self) -> Option<StreamCandidate> {
        self.confirmed = false;
        self.remaining.pop_front()
    }

    pub fn needs_fallback(&self, status: i32, elapsed: Duration) -> bool {
        status == -1 || (!self.confirmed && status != 1 && elapsed >= START_TIMEOUT)
    }

    pub fn confirm(&mut self) -> bool {
        self.confirmed = true;
        if self.recorded {
            return false;
        }
        self.recorded = true;
        true
    }
}
