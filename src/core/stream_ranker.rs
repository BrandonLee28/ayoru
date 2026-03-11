use crate::core::models::StreamCandidate;

fn provider_rank(name: &str) -> u8 {
    match name {
        "wixmp" => 0,
        "youtube" => 1,
        "sharepoint" => 2,
        "hianime" => 3,
        _ => u8::MAX,
    }
}

pub fn rank_streams(streams: &mut [StreamCandidate]) {
    streams.sort_by(|a, b| {
        provider_rank(&a.provider)
            .cmp(&provider_rank(&b.provider))
            .then_with(|| b.is_sub.cmp(&a.is_sub))
            .then_with(|| b.resolution.unwrap_or(0).cmp(&a.resolution.unwrap_or(0)))
    });
}
