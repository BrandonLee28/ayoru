use super::allanime_crypto::{self as crypto, API, ORIGIN};
use crate::app::ProviderRuntime;
use crate::core::models::{Episode, StreamCandidate, Title};
use reqwest::StatusCode;
use serde_json::{Value, json};

pub const ALLANIME_REFERER: &str = ORIGIN;
pub const ALLANIME_BASE: &str = "allanime.day";
pub const ALLANIME_API: &str = API;

const SEARCH_GQL: &str = "query( $search: SearchInput $limit: Int $page: Int $translationType: VaildTranslationTypeEnumType $countryOrigin: VaildCountryOriginEnumType ) { shows( search: $search limit: $limit page: $page translationType: $translationType countryOrigin: $countryOrigin ) { edges { _id name thumbnail availableEpisodes __typename } }}";
const EPISODES_GQL: &str =
    "query ($showId: String!) { show( _id: $showId ) { _id availableEpisodesDetail }}";
const STREAMS_GQL: &str = "query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) { episode( showId: $showId translationType: $translationType episodeString: $episodeString ) { episodeString sourceUrls show { _id countryOfOrigin } }}";

pub struct AllAnimeProvider {
    client: reqwest::Client,
}

impl Default for AllAnimeProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl AllAnimeProvider {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .user_agent("Mozilla/5.0")
                .timeout(std::time::Duration::from_secs(20))
                .build()
                .expect("reqwest client"),
        }
    }

    /// Probe the same GraphQL endpoint used for episodes and stream requests.
    pub async fn check_health(&self) -> Result<(), String> {
        let body = self.gql_request(json!({}), "query { __typename }").await?;
        validate_health_response(&body)
    }

    async fn gql_request(&self, variables: Value, query: &str) -> Result<String, String> {
        let response = self
            .client
            .post(format!("{ALLANIME_API}/api"))
            .header("Referer", ALLANIME_REFERER)
            .json(&json!({"query": query, "variables": variables}))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if let Some(message) = provider_status_error(response.status()) {
            return Err(message);
        }

        let body = response
            .error_for_status()
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())?;
        validate_graphql_response(&body)?;
        Ok(body)
    }
}

fn validate_health_response(raw: &str) -> Result<(), String> {
    validate_graphql_response(raw)?;
    let value: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    match value["data"]["__typename"].as_str() {
        Some(name) if !name.is_empty() => Ok(()),
        _ => Err("Provider returned an invalid health response".into()),
    }
}

fn validate_graphql_response(raw: &str) -> Result<(), String> {
    let value: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    if let Some(errors) = value["errors"].as_array()
        && let Some(error) = errors.first()
    {
        let message = error["message"]
            .as_str()
            .unwrap_or("Unknown provider error");
        if message.starts_with("AA_CRYPTO_") {
            return Err("The provider has changed its playback requirements. Episode streams are currently unavailable; the provider integration needs an update.".into());
        }
        return Err(format!("Provider request failed: {message}"));
    }
    Ok(())
}

fn provider_status_error(status: StatusCode) -> Option<String> {
    if status == StatusCode::FORBIDDEN {
        return Some(
            "Provider blocked by Cloudflare challenge (403 Forbidden). Try again later or use a different network."
                .to_string(),
        );
    }

    None
}

#[async_trait::async_trait]
impl ProviderRuntime for AllAnimeProvider {
    async fn search(&self, query: &str) -> Result<Vec<Title>, String> {
        let variables = json!({
            "search": {"allowAdult": false, "allowUnknown": false, "query": query},
            "limit": 40,
            "page": 1,
            "translationType": "sub",
            "countryOrigin": "ALL"
        });
        let body = self.gql_request(variables, SEARCH_GQL).await?;
        parse_search_titles(&body)
    }

    async fn episodes(&self, title_id: &str) -> Result<Vec<Episode>, String> {
        let variables = json!({ "showId": title_id });
        let body = self.gql_request(variables, EPISODES_GQL).await?;
        parse_episodes(&body)
    }

    async fn streams(
        &self,
        title_id: &str,
        episode: u32,
        prefer_sub: bool,
    ) -> Result<Vec<StreamCandidate>, String> {
        let variables = json!({
            "showId": title_id,
            "translationType": if prefer_sub { "sub" } else { "dub" },
            "episodeString": episode.to_string()
        });
        let mut response = Err("Could not resolve episode sources".to_owned());
        // Fetch current first-party bootstrap data on each attempt. A deployment
        // can rotate between the bootstrap and the episode request; retry once.
        for _ in 0..2 {
            let session = crypto::bootstrap(&self.client).await?;
            let extensions = session.extensions(STREAMS_GQL, crypto::now_ms()?)?;
            let result = self
                .client
                .get(format!("{ALLANIME_API}/api"))
                .header("Referer", ALLANIME_REFERER)
                .header("Origin", ORIGIN)
                .header("x-build-id", &session.build)
                .query(&[
                    ("query", STREAMS_GQL.to_owned()),
                    ("variables", variables.to_string()),
                    ("extensions", extensions.to_string()),
                ])
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?
                .text()
                .await
                .map_err(|e| e.to_string())?;
            response = session.decode(&result).and_then(|body| {
                validate_graphql_response(&body)?;
                Ok(body)
            });
            if response.is_ok() {
                break;
            }
        }
        let body = response?;

        let streams = parse_stream_candidates(&body, prefer_sub)?;
        Ok(expand_streams_with_fetch(streams, |url| async move {
            self.client
                .get(url)
                .header("Referer", ALLANIME_REFERER)
                .send()
                .await
                .map_err(|e| e.to_string())?
                .error_for_status()
                .map_err(|e| e.to_string())?
                .text()
                .await
                .map_err(|e| e.to_string())
        })
        .await)
    }
}

async fn expand_streams_with_fetch<F, Fut>(
    streams: Vec<StreamCandidate>,
    mut fetch_payload: F,
) -> Vec<StreamCandidate>
where
    F: FnMut(String) -> Fut,
    Fut: std::future::Future<Output = Result<String, String>>,
{
    let mut expanded = Vec::new();
    let mp4_pattern = regex::Regex::new(r#"(?:src|file)\s*:\s*["'](https?://[^"']+)["']"#).unwrap();
    for stream in streams {
        if stream.provider == "mp4upload" {
            let Ok(payload) = fetch_payload(stream.url.clone()).await else {
                continue;
            };
            if let Some(captures) = mp4_pattern.captures(&payload) {
                expanded.push(StreamCandidate {
                    url: captures[1].to_owned(),
                    ..stream
                });
            }
        } else if stream.url.contains("/clock.json") || stream.url.contains("/clock?") {
            let Ok(payload) = fetch_payload(stream.url.clone()).await else {
                continue;
            };
            if let Ok(value) = serde_json::from_str::<Value>(&payload) {
                collect_payload_streams(&value, &stream, &mut expanded);
            }
        } else if playable_url(&stream.url) {
            expanded.push(stream);
        }
    }
    expanded
}

fn collect_payload_streams(
    value: &Value,
    stream: &StreamCandidate,
    output: &mut Vec<StreamCandidate>,
) {
    match value {
        Value::Object(map) => {
            let mut template = stream.clone();
            if let Some(referrer) = value["headers"]["Referer"]
                .as_str()
                .filter(|url| playable_url(url))
            {
                template.referrer = Some(referrer.into());
            }
            if let Some(resolution) = value["resolutionStr"]
                .as_str()
                .and_then(|s| s.trim_end_matches('p').parse().ok())
            {
                template.resolution = Some(resolution);
            }
            for url in [value["link"].as_str(), value["hls"]["url"].as_str()]
                .into_iter()
                .flatten()
            {
                for url in expand_wix_url(url)
                    .into_iter()
                    .filter(|url| playable_url(url))
                {
                    output.push(StreamCandidate {
                        url,
                        ..template.clone()
                    });
                }
            }
            for child in map.values() {
                collect_payload_streams(child, &template, output);
            }
        }
        Value::Array(items) => {
            for child in items {
                collect_payload_streams(child, stream, output);
            }
        }
        _ => {}
    }
}

fn playable_url(url: &str) -> bool {
    reqwest::Url::parse(url)
        .is_ok_and(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
}

fn expand_wix_url(url: &str) -> Vec<String> {
    let Some(unwrapped) = url.strip_prefix("https://repackager.wixmp.com/") else {
        return vec![url.into()];
    };
    let base = unwrapped.split(".urlset").next().unwrap_or(unwrapped);
    let pieces: Vec<_> = base.split(',').collect();
    if pieces.len() < 3 {
        return vec![url.into()];
    }
    pieces[1..pieces.len() - 1]
        .iter()
        .filter(|quality| !quality.is_empty())
        .map(|quality| format!("{}{quality}{}", pieces[0], pieces[pieces.len() - 1]))
        .collect()
}

pub fn parse_search_titles(raw: &str) -> Result<Vec<Title>, String> {
    let v: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    let edges = v["data"]["shows"]["edges"]
        .as_array()
        .ok_or("missing edges")?;

    let mut out = Vec::with_capacity(edges.len());
    for edge in edges {
        let id = edge["_id"].as_str().ok_or("missing id")?.to_string();
        let name = edge["name"].as_str().ok_or("missing name")?.to_string();
        let poster = edge["thumbnail"].as_str().map(str::to_owned);
        out.push(Title { id, name, poster });
    }
    Ok(out)
}

pub fn parse_episodes(raw: &str) -> Result<Vec<Episode>, String> {
    let v: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    let eps = v["data"]["show"]["availableEpisodesDetail"]["sub"]
        .as_array()
        .ok_or("missing episodes")?;

    let mut out: Vec<Episode> = eps
        .iter()
        .filter_map(|e| e.as_str())
        .filter_map(|s| s.parse::<u32>().ok())
        .map(|number| Episode { number })
        .collect();
    out.sort_by_key(|e| e.number);
    out.dedup_by_key(|e| e.number);
    Ok(out)
}

pub fn parse_stream_candidates(
    raw: &str,
    prefer_sub: bool,
) -> Result<Vec<StreamCandidate>, String> {
    let v: Value = serde_json::from_str(raw).map_err(|e| e.to_string())?;
    if v["data"]["episode"].is_null() {
        return Ok(Vec::new());
    }

    let sources = v["data"]["episode"]["sourceUrls"]
        .as_array()
        .ok_or("missing sourceUrls")?;

    let mut out = Vec::new();
    for s in sources {
        let (Some(name), Some(raw_url)) = (s["sourceName"].as_str(), s["sourceUrl"].as_str())
        else {
            continue;
        };

        let provider = match name.to_ascii_lowercase().as_str() {
            "default" => "wixmp",
            "yt-mp4" => "youtube",
            "s-mp4" => "sharepoint",
            "luf-mp4" => "hianime",
            "uv-mp4" => "uv",
            "ak" => "ak",
            "mp4" => "mp4upload",
            _ => continue,
        }
        .to_string();
        // A malformed source must not discard other working providers.
        let Ok(url) = decode_source_url(raw_url) else {
            continue;
        };
        let referrer = if provider == "mp4upload" {
            "https://www.mp4upload.com/"
        } else {
            ALLANIME_REFERER
        };
        out.push(StreamCandidate {
            provider,
            url,
            is_sub: prefer_sub,
            resolution: None,
            referrer: Some(referrer.into()),
        });
    }

    Ok(out)
}

pub fn decode_source_url(raw_url: &str) -> Result<String, String> {
    if let Some(hex) = raw_url.strip_prefix("--") {
        return decode_obfuscated(hex).map(|url| normalize_decoded_url(&url));
    }
    Ok(normalize_decoded_url(raw_url))
}

pub fn parse_provider_payload_links(raw: &str) -> Vec<String> {
    let Ok(v) = serde_json::from_str::<Value>(raw) else {
        return Vec::new();
    };

    let mut links = Vec::new();
    collect_links_recursive(&v, &mut links);
    links
}

fn collect_links_recursive(value: &Value, links: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            if let Some(link) = map.get("link").and_then(Value::as_str) {
                links.push(link.to_string());
            }
            if let Some(hls) = map.get("hls").and_then(Value::as_object)
                && let Some(url) = hls.get("url").and_then(Value::as_str)
            {
                links.push(url.to_string());
            }
            for v in map.values() {
                collect_links_recursive(v, links);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                collect_links_recursive(item, links);
            }
        }
        _ => {}
    }
}

fn normalize_decoded_url(decoded: &str) -> String {
    let base = reqwest::Url::parse(&format!("https://{ALLANIME_BASE}/")).unwrap();
    let Ok(mut url) = base.join(decoded) else {
        return decoded.into();
    };
    if url.path().ends_with("/clock") {
        url.set_path(&format!("{}.json", url.path()));
    }
    url.into()
}

fn decode_obfuscated(hex: &str) -> Result<String, String> {
    if !hex.len().is_multiple_of(2) || !hex.is_ascii() {
        return Err("invalid encoded source length".into());
    }
    let bytes = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).map_err(|_| "invalid source encoding")?;
            u8::from_str_radix(pair, 16)
                .map(|byte| byte ^ 0x38)
                .map_err(|_| "invalid source encoding")
        })
        .collect::<Result<Vec<_>, _>>()?;
    String::from_utf8(bytes).map_err(|_| "invalid source encoding".into())
}

#[cfg(test)]
mod tests {
    use super::{
        collect_payload_streams, decode_source_url, expand_streams_with_fetch,
        parse_stream_candidates, provider_status_error, validate_graphql_response,
        validate_health_response,
    };
    use crate::core::models::StreamCandidate;
    use reqwest::StatusCode;

    #[test]
    fn health_requires_a_valid_graphql_reply() {
        assert!(validate_health_response(r#"{"data":{"__typename":"Query"}}"#).is_ok());
        for raw in [
            "<html>Blocked</html>",
            "{}",
            r#"{"data":null}"#,
            r#"{"errors":[{"message":"Unavailable"}],"data":{"__typename":"Query"}}"#,
        ] {
            assert!(validate_health_response(raw).is_err());
        }
    }

    #[test]
    fn reports_graphql_rejections_instead_of_silently_returning_no_streams() {
        let error = validate_graphql_response(
            r#"{"errors":[{"message":"AA_CRYPTO_MISSING"}],"data":{"episode":null}}"#,
        )
        .unwrap_err();
        assert!(error.contains("playback requirements"));
        assert!(validate_graphql_response(r#"{"data":{"episode":null}}"#).is_ok());
    }

    #[test]
    fn explains_cloudflare_forbidden_provider_response() {
        assert_eq!(
            provider_status_error(StatusCode::FORBIDDEN),
            Some("Provider blocked by Cloudflare challenge (403 Forbidden). Try again later or use a different network.".to_string())
        );
    }

    #[tokio::test]
    async fn skips_failed_clock_source_and_keeps_other_candidates() {
        let streams = vec![
            StreamCandidate {
                provider: "wixmp".to_string(),
                url: "https://allanime.day/apivtwo/clock.json?id=bad".to_string(),
                is_sub: true,
                resolution: None,
                referrer: None,
            },
            StreamCandidate {
                provider: "youtube".to_string(),
                url: "https://tools.fast4speed.rsvp/video.mp4".to_string(),
                is_sub: true,
                resolution: None,
                referrer: None,
            },
        ];

        let expanded = expand_streams_with_fetch(streams, |_| async {
            Err::<String, String>("500".to_string())
        })
        .await;

        assert_eq!(expanded.len(), 1);
        assert_eq!(expanded[0].provider, "youtube");
    }
    #[test]
    fn normalizes_clock_paths_once_and_handles_bad_encodings() {
        let url = "https://allanime.day/apivtwo/clock.json?id=42";
        assert_eq!(decode_source_url(url).unwrap(), url);
        assert_eq!(decode_source_url("/apivtwo/clock?id=42").unwrap(), url);
        assert!(decode_source_url("--é").is_err());
        assert!(decode_source_url("--zz").is_err());
        let raw = r#"{"data":{"episode":{"sourceUrls":[{"sourceName":"Default","sourceUrl":null},{"sourceName":"Default","sourceUrl":"--zz"},{"sourceName":"S-Mp4","sourceUrl":"https://example.com/video.mp4"}]}}}"#;
        let candidates = parse_stream_candidates(raw, true).unwrap();
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].provider, "sharepoint");
    }

    #[test]
    fn expands_clock_quality_paths_and_preserves_media_headers() {
        let source = StreamCandidate {
            provider: "wixmp".into(),
            url: "https://allanime.day/apivtwo/clock.json".into(),
            is_sub: true,
            resolution: None,
            referrer: None,
        };
        let payload = serde_json::json!({"links":[
            {"link":"javascript:bad"},
            {"link":"https://cdn.example/video.m3u8","headers":{"Referer":"https://player.example/"},"resolutionStr":"1080p"},
            {"link":"https://repackager.wixmp.com/https://cdn.example/,720p,1080p,/video.mp4.urlset/master.m3u8"}
        ]});
        let mut streams = vec![];
        collect_payload_streams(&payload, &source, &mut streams);
        assert_eq!(streams.len(), 3);
        assert_eq!(
            streams[0].referrer.as_deref(),
            Some("https://player.example/")
        );
        assert_eq!(streams[0].resolution, Some(1080));
        assert_eq!(streams[2].url, "https://cdn.example/1080p/video.mp4");
    }

    #[tokio::test]
    async fn resolves_mp4upload_page_into_video_and_keeps_referrer() {
        let source = StreamCandidate {
            provider: "mp4upload".into(),
            url: "https://www.mp4upload.com/embed-abc.html".into(),
            is_sub: true,
            resolution: None,
            referrer: Some("https://www.mp4upload.com/".into()),
        };
        let resolved = expand_streams_with_fetch(vec![source], |_| async {
            Ok("player({src: \"https://cdn.example/video.mp4\"})".into())
        })
        .await;
        assert_eq!(resolved[0].url, "https://cdn.example/video.mp4");
        assert_eq!(
            resolved[0].referrer.as_deref(),
            Some("https://www.mp4upload.com/")
        );
    }
}
