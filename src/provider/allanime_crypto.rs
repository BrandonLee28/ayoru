//! The public frontend publishes mask fragments and bootstrap parameters.
//! Read that data without evaluating downloaded JavaScript; obtain each epoch's
//! authenticated session key from the provider itself, rather than a stale key feed.
use base64::{Engine, engine::general_purpose::STANDARD};
use regex::Regex;
use ring::{aead, digest, hmac};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

pub(super) const ORIGIN: &str = "https://mkissa.to";
pub(super) const API: &str = "https://api.mkissa.net";
const CDN: &str = "https://cdn.mkissa.net/all/mk/_app/immutable";
const FORMAT_ERROR: &str =
    "The provider's frontend format has changed. Its playback integration needs an update.";

#[derive(Debug)]
struct Manifest {
    build: String,
    mask: [u8; 32],
    prefix: String,
    separator: String,
    parts: Vec<String>,
    epoch_ms: u64,
    grace_ms: u64,
}

#[derive(Deserialize)]
struct Bootstrap {
    epoch: u64,
    #[serde(rename = "partB")]
    part_b: String,
    k: String,
}

#[derive(Serialize)]
struct RequestPayload<'a> {
    v: u8,
    ts: u64,
    epoch: u64,
    #[serde(rename = "buildId")]
    build_id: &'a str,
    qh: &'a str,
    k: &'a str,
}

pub(super) struct SessionKey {
    pub build: String,
    epoch: u64,
    key: [u8; 32],
}

async fn fetch_text(client: &reqwest::Client, url: &str) -> Result<String, String> {
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|e| e.to_string())? {
        if bytes.len() + chunk.len() > 4 * 1024 * 1024 {
            return Err("Provider configuration is too large".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    String::from_utf8(bytes).map_err(|_| FORMAT_ERROR.into())
}

pub(super) async fn bootstrap(client: &reqwest::Client) -> Result<SessionKey, String> {
    let home = fetch_text(client, ORIGIN).await?;
    let entry = first(r"/entry/app\.[A-Za-z0-9_.-]+\.js", &home)?;
    let app = fetch_text(client, &format!("{CDN}{entry}")).await?;
    let imports = Regex::new(r#""\.\./(chunks/[A-Za-z0-9_.-]+\.js)""#).unwrap();
    // The shared API module is among the entry's first imports. Bound the work
    // if a deployment changes the bundle structure instead of crawling the site.
    let mut manifest = None;
    for captures in imports.captures_iter(&app).take(8) {
        let chunk = fetch_text(client, &format!("{CDN}/{}", &captures[1])).await?;
        if chunk.contains("saltMul:") && chunk.contains("sourceUrls") {
            manifest = Some(parse_manifest(&chunk)?);
            break;
        }
    }
    let manifest = manifest.ok_or(FORMAT_ERROR)?;
    let now = now_ms()?;
    let current = now / manifest.epoch_ms;
    let previous = if now % manifest.epoch_ms < manifest.grace_ms {
        current.saturating_sub(1)
    } else {
        current
    };
    let mut error = "Provider crypto bootstrap failed".to_owned();
    for epoch in [previous, current] {
        let values = HashMap::from([
            ("buildId", manifest.build.clone()),
            ("epoch", epoch.to_string()),
            ("group", "mkissa".into()),
            ("host", "mkissa.to".into()),
            ("lane", "k7".into()),
        ]);
        let message = manifest
            .parts
            .iter()
            .map(|part| values[part.as_str()].as_str())
            .collect::<Vec<_>>()
            .join(&manifest.separator);
        let intermediate = sign(
            &manifest.mask,
            format!("{}{}", manifest.prefix, manifest.build).as_bytes(),
        );
        let boot = hex(&sign(&intermediate, message.as_bytes()));
        let response = client
            .get(format!("{API}/client-crypto/v1/bootstrap"))
            .query(&[("buildId", manifest.build.as_str()), ("k", "k7")])
            .header("Referer", ORIGIN)
            .header("Origin", ORIGIN)
            .header("x-build-id", &manifest.build)
            .header("x-aa-boot", boot)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if !response.status().is_success() {
            error = format!("Provider crypto bootstrap returned {}", response.status());
            if current == previous {
                break;
            }
            continue;
        }
        let value: Value = response.json().await.map_err(|e| e.to_string())?;
        if let Ok(data) = serde_json::from_value::<Bootstrap>(value.clone()) {
            let part_b = STANDARD.decode(data.part_b).map_err(|_| FORMAT_ERROR)?;
            if part_b.len() != 32 || data.k != "k7" {
                return Err(FORMAT_ERROR.into());
            }
            let key = std::array::from_fn(|i| part_b[i] ^ manifest.mask[i]);
            return Ok(SessionKey {
                build: manifest.build,
                epoch: data.epoch,
                key,
            });
        }
        error = format!(
            "Provider crypto bootstrap failed: {}",
            value
                .get("error")
                .or_else(|| value.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("invalid response")
        );
        if current == previous {
            break;
        }
    }
    Err(error)
}

impl SessionKey {
    pub fn extensions(&self, query: &str, now: u64) -> Result<Value, String> {
        let hash = hex(digest::digest(&digest::SHA256, query.as_bytes()).as_ref());
        let timestamp = now / 300_000 * 300_000;
        let nonce_hash = digest::digest(
            &digest::SHA256,
            format!("{}:{}:{hash}:{timestamp}:k7", self.epoch, self.build).as_bytes(),
        );
        let nonce: [u8; 12] = nonce_hash.as_ref()[..12].try_into().unwrap();
        let mut payload = serde_json::to_vec(&RequestPayload {
            v: 1,
            ts: timestamp,
            epoch: self.epoch,
            build_id: &self.build,
            qh: &hash,
            k: "k7",
        })
        .map_err(|e| e.to_string())?;
        // An identical five-minute payload produces the identical token; the
        // nonce incorporates every payload field that can change.
        cipher(&self.key)?
            .seal_in_place_append_tag(
                aead::Nonce::assume_unique_for_key(nonce),
                aead::Aad::empty(),
                &mut payload,
            )
            .map_err(|_| "Could not encode provider request")?;
        let mut token = vec![1];
        token.extend_from_slice(&nonce);
        token.extend(payload);
        Ok(
            json!({"persistedQuery":{"version":1,"sha256Hash":hash},"aaReq":STANDARD.encode(token),"k":"k7"}),
        )
    }

    pub fn decode(&self, body: &str) -> Result<String, String> {
        let mut value: Value = serde_json::from_str(body).map_err(|e| e.to_string())?;
        let Some(encoded) = value["data"]["tobeparsed"].as_str() else {
            return Ok(body.into());
        };
        let raw = STANDARD
            .decode(encoded)
            .map_err(|_| "Invalid encrypted provider response")?;
        if raw.len() < 29 || raw[0] != 1 {
            return Err("Unsupported encrypted provider response".into());
        }
        let nonce: [u8; 12] = raw[1..13].try_into().unwrap();
        let legacy: [u8; 32] = digest::digest(&digest::SHA256, b"Xot36i3lK3:v1")
            .as_ref()
            .try_into()
            .unwrap();
        for key in [&self.key, &legacy] {
            let mut ciphertext = raw[13..].to_vec();
            if let Ok(plaintext) = cipher(key)?.open_in_place(
                aead::Nonce::assume_unique_for_key(nonce),
                aead::Aad::empty(),
                &mut ciphertext,
            ) {
                value["data"] = serde_json::from_slice(plaintext)
                    .map_err(|_| "Provider response was not valid JSON")?;
                return serde_json::to_string(&value).map_err(|e| e.to_string());
            }
        }
        Err("The provider's playback key changed. Please retry the episode.".into())
    }
}

pub(super) fn now_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .map_err(|_| "Check your system clock".into())
}

fn cipher(key: &[u8]) -> Result<aead::LessSafeKey, String> {
    aead::UnboundKey::new(&aead::AES_256_GCM, key)
        .map(aead::LessSafeKey::new)
        .map_err(|_| "Invalid provider crypto key".into())
}
fn sign(key: &[u8], text: &[u8]) -> Vec<u8> {
    hmac::sign(&hmac::Key::new(hmac::HMAC_SHA256, key), text)
        .as_ref()
        .to_vec()
}
fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn first(pattern: &str, text: &str) -> Result<String, String> {
    Regex::new(pattern)
        .unwrap()
        .find(text)
        .map(|m| m.as_str().to_owned())
        .ok_or_else(|| FORMAT_ERROR.into())
}
fn captures(pattern: &str, text: &str) -> Result<Vec<String>, String> {
    Regex::new(pattern)
        .unwrap()
        .captures(text)
        .map(|c| {
            c.iter()
                .skip(1)
                .map(|m| m.unwrap().as_str().to_owned())
                .collect()
        })
        .ok_or_else(|| FORMAT_ERROR.into())
}

/// The parser only accepts literal string-table lookups, concatenations, and
/// integer arithmetic. Nothing downloaded is executed or imported as code.
fn parse_manifest(source: &str) -> Result<Manifest, String> {
    let seed_end = source.find("saltMul:").ok_or(FORMAT_ERROR)?;
    let seed_start = source[..seed_end].rfind("const ").ok_or(FORMAT_ERROR)?;
    let seed = &source[seed_start..];
    let root_call = captures(r"^const \w+=(\w+)\(", seed)?;
    let root_wrapper = captures(
        &format!(
            r"function {}\(\w+,\w+\)\{{return (\w+)\(\w+-\{{\w+:\d+\}}\.\w+\)\}}",
            regex::escape(&root_call[0])
        ),
        source,
    )?;
    let accessor = captures(
        &format!(
            r"function ({})\(\w+,\w+\)\{{return \w+=\w+-(\d+),(\w+)\(\)\[\w+\]\}}",
            regex::escape(&root_wrapper[0])
        ),
        source,
    )?;
    let base: usize = accessor[1].parse().map_err(|_| FORMAT_ERROR)?;
    let table_text = captures(
        &format!(
            r"(?s)function {}\(\)\{{const \w+=\[(.*?)\];return",
            regex::escape(&accessor[2])
        ),
        source,
    )?;
    let mut table = split_expressions(&table_text[0])?
        .iter()
        .map(|s| string_literal(s))
        .collect::<Result<Vec<_>, _>>()?;
    if table.is_empty() || table.len() > 2048 {
        return Err(FORMAT_ERROR.into());
    }
    let wrapper = Regex::new(&format!(
        r"function (\w+)\(\w+,\w+\)\{{return {}\(\w+-\{{\w+:(\d+)\}}\.\w+\)\}}",
        regex::escape(&accessor[0])
    ))
    .unwrap();
    let offsets: HashMap<String, usize> = wrapper
        .captures_iter(source)
        .filter_map(|c| {
            Some((
                c[1].to_owned(),
                base.checked_add(c[2].parse::<usize>().ok()?)?,
            ))
        })
        .collect();
    let build_expression = captures(r"^const \w+=([^,]+,[^)]*\))", seed)?;
    let fragments = captures(r"\w+=\[(.*?)\],\w+=\{v:", seed)?;
    let fragment_expressions = split_expressions(&fragments[0])?;
    if fragment_expressions.len() != 4 {
        return Err(FORMAT_ERROR.into());
    }
    let prefix_expression = captures(r"bootPrefix:(.*?),join:", seed)?;
    let separator_expression = captures(r"join:([^,]+),parts:", seed)?;
    let parts_expression = captures(r"parts:\[(.*?)\],omitEmptyLane:", seed)?;
    let salt_mul = byte_field(seed, "saltMul")?;
    let salt_add = byte_field(seed, "saltAdd")?;
    let frag_mul = byte_field(seed, "fragMul")?;
    let frag_add = byte_field(seed, "fragAdd")?;
    let periods = Regex::new(r"Number\(([^()]+)\)")
        .unwrap()
        .captures_iter(seed)
        .take(2)
        .map(|c| arithmetic(&c[1]))
        .collect::<Result<Vec<_>, _>>()?;
    if periods.len() != 2
        || !(86_400_000..=2_592_000_000).contains(&periods[0])
        || periods[1] >= periods[0]
    {
        return Err(FORMAT_ERROR.into());
    }
    for _ in 0..table.len() {
        if let Ok(build) = string_expression(&build_expression[0], &table, &offsets)
            && !build.is_empty()
            && build.len() <= 8
            && build.bytes().all(|b| b.is_ascii_digit())
        {
            let decoded = fragment_expressions
                .iter()
                .map(|e| {
                    string_expression(e, &table, &offsets)
                        .and_then(|s| STANDARD.decode(s).map_err(|_| FORMAT_ERROR.to_owned()))
                })
                .collect::<Result<Vec<_>, _>>();
            if let Ok(decoded) = decoded
                && decoded.iter().all(|part| part.len() == 8)
            {
                let prefix = string_expression(&prefix_expression[0], &table, &offsets)?;
                let separator = string_literal(&separator_expression[0])?;
                let parts = split_expressions(&parts_expression[0])?
                    .iter()
                    .map(|e| string_expression(e, &table, &offsets))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut sorted = parts.clone();
                sorted.sort();
                if sorted != ["buildId", "epoch", "group", "host", "lane"]
                    || prefix.len() > 64
                    || separator.len() > 4
                {
                    return Err(FORMAT_ERROR.into());
                }
                let mask = std::array::from_fn(|i| {
                    decoded[i / 8][i % 8]
                        ^ build.as_bytes()[i % build.len()]
                        ^ (i as u8).wrapping_mul(salt_mul).wrapping_add(salt_add)
                        ^ (i as u8 / 8)
                            .wrapping_mul(frag_mul)
                            .wrapping_add((i as u8 % 8).wrapping_mul(frag_add))
                });
                return Ok(Manifest {
                    build,
                    mask,
                    prefix,
                    separator,
                    parts,
                    epoch_ms: periods[0],
                    grace_ms: periods[1],
                });
            }
        }
        table.rotate_left(1);
    }
    Err(FORMAT_ERROR.into())
}

fn byte_field(text: &str, name: &str) -> Result<u8, String> {
    captures(&format!(r"{name}:(\d+)"), text)?[0]
        .parse()
        .map_err(|_| FORMAT_ERROR.into())
}
fn string_literal(text: &str) -> Result<String, String> {
    let text = text.trim();
    if text.starts_with('"') {
        serde_json::from_str(text).map_err(|_| FORMAT_ERROR.into())
    } else if text.starts_with('\'') && text.ends_with('\'') && text.len() >= 2 {
        Ok(text[1..text.len() - 1]
            .replace("\\'", "'")
            .replace("\\\\", "\\"))
    } else {
        Err(FORMAT_ERROR.into())
    }
}
fn split_expressions(text: &str) -> Result<Vec<&str>, String> {
    split_on(text, b',')
}
fn split_on(text: &str, separator: u8) -> Result<Vec<&str>, String> {
    let mut output = vec![];
    let mut start = 0;
    let mut depth = 0_i32;
    let mut quote = 0;
    let mut escaped = false;
    for (i, b) in text.bytes().enumerate() {
        if quote != 0 {
            if escaped {
                escaped = false;
            } else if b == b'\\' {
                escaped = true;
            } else if b == quote {
                quote = 0;
            }
            continue;
        }
        match b {
            b'\'' | b'"' => quote = b,
            b'(' => depth += 1,
            b')' => depth -= 1,
            _ => {}
        }
        if depth < 0 {
            return Err(FORMAT_ERROR.into());
        }
        if depth == 0 && b == separator {
            output.push(text[start..i].trim());
            start = i + 1;
        }
    }
    if depth != 0 || quote != 0 {
        return Err(FORMAT_ERROR.into());
    }
    output.push(text[start..].trim());
    Ok(output)
}
fn string_expression(
    text: &str,
    table: &[String],
    offsets: &HashMap<String, usize>,
) -> Result<String, String> {
    split_on(text, b'+')?
        .iter()
        .map(|part| {
            if part.starts_with(['\'', '"']) {
                return string_literal(part);
            }
            let (name, arguments) = part.split_once('(').ok_or(FORMAT_ERROR)?;
            let (ignored, index) = arguments
                .strip_suffix(')')
                .ok_or(FORMAT_ERROR)?
                .split_once(',')
                .ok_or(FORMAT_ERROR)?;
            ignored.parse::<u64>().map_err(|_| FORMAT_ERROR)?;
            let position = index
                .parse::<usize>()
                .map_err(|_| FORMAT_ERROR)?
                .checked_sub(*offsets.get(name).ok_or(FORMAT_ERROR)?)
                .ok_or(FORMAT_ERROR)?;
            table
                .get(position)
                .cloned()
                .ok_or_else(|| FORMAT_ERROR.into())
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|s| s.concat())
}
fn arithmetic(text: &str) -> Result<u64, String> {
    let mut total = 0_i64;
    let mut product = 1_i64;
    let mut position = 0;
    let bytes = text.as_bytes();
    while position < bytes.len() {
        let mut sign = 1;
        while position < bytes.len() && matches!(bytes[position], b'+' | b'-') {
            if bytes[position] == b'-' {
                sign = -sign;
            }
            position += 1;
        }
        let start = position;
        while position < bytes.len() && bytes[position].is_ascii_digit() {
            position += 1;
        }
        let value = text[start..position]
            .parse::<i64>()
            .map_err(|_| FORMAT_ERROR)?;
        product = product
            .checked_mul(value)
            .and_then(|v| v.checked_mul(sign))
            .ok_or(FORMAT_ERROR)?;
        if position == bytes.len() || bytes[position] != b'*' {
            total = total.checked_add(product).ok_or(FORMAT_ERROR)?;
            product = 1;
        } else {
            position += 1;
            if position == bytes.len() {
                return Err(FORMAT_ERROR.into());
            }
        }
        if position < bytes.len() && !matches!(bytes[position], b'+' | b'-' | b'0'..=b'9') {
            return Err(FORMAT_ERROR.into());
        }
    }
    u64::try_from(total).map_err(|_| FORMAT_ERROR.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn session() -> SessionKey {
        SessionKey {
            build: "189".into(),
            epoch: 2967,
            key: std::array::from_fn(|i| i as u8),
        }
    }

    #[test]
    fn reads_rotated_literal_manifest_without_executing_javascript() {
        let manifest =
            parse_manifest(include_str!("../../tests/fixtures/provider_manifest.js")).unwrap();
        assert_eq!(manifest.build, "999");
        assert_eq!(manifest.mask, std::array::from_fn(|i| i as u8));
        assert_eq!(manifest.prefix, "test:");
        assert_eq!(manifest.epoch_ms, 604_800_000);
        assert!(parse_manifest("process.exit(); fetch('https://example.com')").is_err());
        assert!(string_expression("eval('anything')", &[], &HashMap::new()).is_err());
    }

    #[test]
    fn token_matches_independent_aes_gcm_fixture() {
        let value = session()
            .extensions("query test", 1_794_630_000_000)
            .unwrap();
        assert_eq!(
            value["aaReq"].as_str().unwrap(),
            include_str!("../../tests/fixtures/provider_token.txt").trim()
        );
        assert_eq!(
            session()
                .extensions("query test", 1_794_630_000_001)
                .unwrap(),
            value
        );
    }

    #[test]
    fn authenticates_decrypted_sources_and_rejects_tampered_data() {
        let fixture = include_str!("../../tests/fixtures/provider_encrypted.json");
        let decoded: Value = serde_json::from_str(&session().decode(fixture).unwrap()).unwrap();
        assert_eq!(
            decoded["data"]["episode"]["sourceUrls"][0]["sourceName"],
            "Yt-mp4"
        );
        let mut value: Value = serde_json::from_str(fixture).unwrap();
        let mut bytes = STANDARD
            .decode(value["data"]["tobeparsed"].as_str().unwrap())
            .unwrap();
        *bytes.last_mut().unwrap() ^= 1;
        value["data"]["tobeparsed"] = STANDARD.encode(bytes).into();
        assert!(session().decode(&value.to_string()).is_err());
        for data in ["", "AQ==", "not base64!"] {
            assert!(
                session()
                    .decode(&json!({"data":{"tobeparsed":data}}).to_string())
                    .is_err()
            );
        }
    }

    #[test]
    fn evaluates_only_checked_integer_arithmetic() {
        assert_eq!(arithmetic("691810447+1*-87010447").unwrap(), 604_800_000);
        assert_eq!(
            arithmetic("5141*10217+-56236803+90111206").unwrap(),
            86_400_000
        );
        assert!(arithmetic("999999999999999999999999").is_err());
        assert!(arithmetic("call()").is_err());
    }
}
