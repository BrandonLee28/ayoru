use crate::app::ProviderRuntime;
use crate::core::models::{Episode, StreamCandidate, Title};
use serde_json::{Value, json};

pub const ALLANIME_REFERER: &str = "https://allmanga.to";
pub const ALLANIME_BASE: &str = "allanime.day";
pub const ALLANIME_API: &str = "https://api.allanime.day";

const SEARCH_GQL: &str = "query( $search: SearchInput $limit: Int $page: Int $translationType: VaildTranslationTypeEnumType $countryOrigin: VaildCountryOriginEnumType ) { shows( search: $search limit: $limit page: $page translationType: $translationType countryOrigin: $countryOrigin ) { edges { _id name availableEpisodes __typename } }}";
const EPISODES_GQL: &str =
    "query ($showId: String!) { show( _id: $showId ) { _id availableEpisodesDetail }}";
const STREAMS_GQL: &str = "query ($showId: String!, $translationType: VaildTranslationTypeEnumType!, $episodeString: String!) { episode( showId: $showId translationType: $translationType episodeString: $episodeString ) { episodeString sourceUrls }}";

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
                .build()
                .expect("reqwest client"),
        }
    }

    async fn gql_request(&self, variables: Value, query: &str) -> Result<String, String> {
        self.client
            .get(format!("{ALLANIME_API}/api"))
            .header("Referer", ALLANIME_REFERER)
            .query(&[
                ("variables", variables.to_string()),
                ("query", query.to_string()),
            ])
            .send()
            .await
            .map_err(|e| e.to_string())?
            .error_for_status()
            .map_err(|e| e.to_string())?
            .text()
            .await
            .map_err(|e| e.to_string())
    }
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
        let body = self.gql_request(variables, STREAMS_GQL).await?;

        let mut streams = parse_stream_candidates(&body, prefer_sub)?;
        let mut expanded = Vec::new();
        for s in streams.drain(..) {
            if s.url.contains("/clock.json") || s.url.contains("/clock?") {
                let payload = self
                    .client
                    .get(&s.url)
                    .header("Referer", ALLANIME_REFERER)
                    .send()
                    .await
                    .map_err(|e| e.to_string())?
                    .error_for_status()
                    .map_err(|e| e.to_string())?
                    .text()
                    .await
                    .map_err(|e| e.to_string())?;

                let urls = parse_provider_payload_links(&payload);
                if urls.is_empty() {
                    expanded.push(s);
                    continue;
                }

                for url in urls {
                    expanded.push(StreamCandidate {
                        provider: s.provider.clone(),
                        url,
                        is_sub: s.is_sub,
                        resolution: s.resolution,
                    });
                }
                continue;
            }
            expanded.push(s);
        }

        Ok(expanded)
    }
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
        out.push(Title { id, name });
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
    let sources = v["data"]["episode"]["sourceUrls"]
        .as_array()
        .ok_or("missing sourceUrls")?;

    let mut out = Vec::new();
    for s in sources {
        let name = s["sourceName"].as_str().ok_or("missing sourceName")?;
        let raw_url = s["sourceUrl"].as_str().ok_or("missing sourceUrl")?;

        let provider = match name {
            "Default" => "wixmp",
            "Yt-mp4" => "youtube",
            "S-mp4" => "sharepoint",
            "Luf-Mp4" => "hianime",
            _ => continue,
        }
        .to_string();

        let url = decode_source_url(raw_url)?;
        out.push(StreamCandidate {
            provider,
            url,
            is_sub: prefer_sub,
            resolution: None,
        });
    }

    Ok(out)
}

pub fn decode_source_url(raw_url: &str) -> Result<String, String> {
    if let Some(hex) = raw_url.strip_prefix("--") {
        return decode_obfuscated(hex).map(|url| normalize_decoded_url(&url));
    }
    Ok(raw_url.to_string())
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
    let fixed_clock = decoded.replace("/clock", "/clock.json");
    if fixed_clock.starts_with("http://") || fixed_clock.starts_with("https://") {
        return fixed_clock;
    }
    format!("https://{ALLANIME_BASE}{fixed_clock}")
}

fn decode_obfuscated(hex: &str) -> Result<String, String> {
    if !hex.len().is_multiple_of(2) {
        return Err("invalid encoded source length".to_string());
    }

    let mut out = String::with_capacity(hex.len() / 2);
    let mut i = 0;
    while i < hex.len() {
        let part = &hex[i..i + 2];
        out.push(decode_pair(part)?);
        i += 2;
    }
    Ok(out)
}

fn decode_pair(pair: &str) -> Result<char, String> {
    let c = match pair {
        "79" => 'A',
        "7a" => 'B',
        "7b" => 'C',
        "7c" => 'D',
        "7d" => 'E',
        "7e" => 'F',
        "7f" => 'G',
        "70" => 'H',
        "71" => 'I',
        "72" => 'J',
        "73" => 'K',
        "74" => 'L',
        "75" => 'M',
        "76" => 'N',
        "77" => 'O',
        "68" => 'P',
        "69" => 'Q',
        "6a" => 'R',
        "6b" => 'S',
        "6c" => 'T',
        "6d" => 'U',
        "6e" => 'V',
        "6f" => 'W',
        "60" => 'X',
        "61" => 'Y',
        "62" => 'Z',
        "59" => 'a',
        "5a" => 'b',
        "5b" => 'c',
        "5c" => 'd',
        "5d" => 'e',
        "5e" => 'f',
        "5f" => 'g',
        "50" => 'h',
        "51" => 'i',
        "52" => 'j',
        "53" => 'k',
        "54" => 'l',
        "55" => 'm',
        "56" => 'n',
        "57" => 'o',
        "48" => 'p',
        "49" => 'q',
        "4a" => 'r',
        "4b" => 's',
        "4c" => 't',
        "4d" => 'u',
        "4e" => 'v',
        "4f" => 'w',
        "40" => 'x',
        "41" => 'y',
        "42" => 'z',
        "08" => '0',
        "09" => '1',
        "0a" => '2',
        "0b" => '3',
        "0c" => '4',
        "0d" => '5',
        "0e" => '6',
        "0f" => '7',
        "00" => '8',
        "01" => '9',
        "15" => '-',
        "16" => '.',
        "67" => '_',
        "46" => '~',
        "02" => ':',
        "17" => '/',
        "07" => '?',
        "1b" => '#',
        "63" => '[',
        "65" => ']',
        "78" => '@',
        "19" => '!',
        "1c" => '$',
        "1e" => '&',
        "10" => '(',
        "11" => ')',
        "12" => '*',
        "13" => '+',
        "14" => ',',
        "03" => ';',
        "05" => '=',
        "1d" => '%',
        _ => return Err(format!("unknown encoded byte: {pair}")),
    };
    Ok(c)
}
