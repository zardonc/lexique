#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_validation_rejects_blank_oversized_and_control_characters() {
        assert!(!valid_text(" \n\t"));
        assert!(!valid_text(&"x".repeat(5001)));
        assert!(!valid_text("hello\0world"));
        assert!(valid_text("é\r\n\tbonjour"));
    }

    #[test]
    fn quota_error_and_oversized_response_are_reported() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
        };
        for (status, body, expected) in [
            ("456 Quota Exceeded", String::new(), "quota_or_rate_limited"),
            ("200 OK", "x".repeat(256 * 1024 + 1), "invalid_response"),
            (
                "200 OK",
                r#"{"translations":[{"detected_source_language":"FR","text":""}]}"#.into(),
                "invalid_response",
            ),
        ] {
            let server = TcpListener::bind("127.0.0.1:0").unwrap();
            let endpoint = format!("http://{}/translate", server.local_addr().unwrap());
            let thread = std::thread::spawn(move || {
                let (mut socket, _) = server.accept().unwrap();
                let mut request = [0; 4096];
                let _ = socket.read(&mut request).unwrap();
                let _ = write!(
                    socket,
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
            });
            let error = tauri::async_runtime::block_on(translate(
                &endpoint, "secret", "bonjour", "fr", "en",
            ))
            .unwrap_err();
            assert_eq!(error, expected);
            thread.join().unwrap();
        }
    }

    #[test]
    fn french_fallback_preserves_source_and_maps_authorization() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
        };
        let server = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/translate", server.local_addr().unwrap());
        let thread = std::thread::spawn(move || {
            for (index, expected_target) in ["ZH-HANS", "FR"].iter().enumerate() {
                let (mut socket, _) = server.accept().unwrap();
                let mut request = Vec::new();
                let mut buffer = [0; 2048];
                loop {
                    let count = socket.read(&mut buffer).unwrap();
                    assert!(count > 0);
                    request.extend_from_slice(&buffer[..count]);
                    let Some(header_end) =
                        request.windows(4).position(|bytes| bytes == b"\r\n\r\n")
                    else {
                        continue;
                    };
                    let headers = String::from_utf8_lossy(&request[..header_end]);
                    let length: usize = headers
                        .lines()
                        .find_map(|line| {
                            line.to_ascii_lowercase()
                                .strip_prefix("content-length: ")
                                .map(|value| value.parse().unwrap())
                        })
                        .unwrap();
                    if request.len() < header_end + 4 + length {
                        continue;
                    }
                    assert!(
                        headers
                            .to_ascii_lowercase()
                            .contains("authorization: deepl-auth-key secret")
                    );
                    let body: serde_json::Value =
                        serde_json::from_slice(&request[header_end + 4..header_end + 4 + length])
                            .unwrap();
                    assert_eq!(body["text"][0], "one\ntwo");
                    assert_eq!(body["target_lang"], *expected_target);
                    break;
                }
                let body = if index == 0 {
                    r#"{"translations":[{"detected_source_language":"ZH","text":"first"}]}"#
                } else {
                    r#"{"translations":[{"detected_source_language":"ZH","text":"second"}]}"#
                };
                write!(socket, "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}", body.len(), body).unwrap();
            }
        });
        let result = tauri::async_runtime::block_on(translate(
            &endpoint, "secret", "one\ntwo", "auto", "zh-CN",
        ))
        .unwrap();
        assert_eq!(
            (
                result.text.as_str(),
                result.source.as_str(),
                result.target.as_str()
            ),
            ("second", "zh-CN", "fr")
        );
        thread.join().unwrap();
    }
}
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Serialize)]
pub(crate) struct TranslationResult {
    pub text: String,
    pub source: String,
    pub target: String,
}

#[derive(Deserialize)]
struct DeeplResponse {
    translations: Vec<DeeplTranslation>,
}

#[derive(Deserialize)]
struct DeeplTranslation {
    detected_source_language: String,
    text: String,
}

fn language_code(language: &str) -> Result<&'static str, &'static str> {
    match language {
        "fr" => Ok("FR"),
        "zh-CN" => Ok("ZH"),
        "en" => Ok("EN"),
        _ => Err("invalid_request"),
    }
}

fn detected_language(language: &str) -> Result<&'static str, &'static str> {
    match language {
        "FR" => Ok("fr"),
        "ZH" => Ok("zh-CN"),
        "EN" => Ok("en"),
        _ => Err("invalid_response"),
    }
}

pub(crate) fn valid_text(text: &str) -> bool {
    !text.trim().is_empty()
        && text.chars().count() <= 5000
        && !text
            .chars()
            .any(|ch| ch.is_control() && !matches!(ch, '\n' | '\r' | '\t'))
}

pub(crate) async fn translate_deepl(
    tier: &lexique_storage::DeeplTier,
    key: &str,
    text: &str,
    source: &str,
    target: &str,
) -> Result<TranslationResult, &'static str> {
    let endpoint = match tier {
        lexique_storage::DeeplTier::Free => "https://api-free.deepl.com/v2/translate",
        lexique_storage::DeeplTier::Pro => "https://api.deepl.com/v2/translate",
    };
    translate(endpoint, key, text, source, target).await
}

async fn translate(
    endpoint: &str,
    key: &str,
    text: &str,
    source: &str,
    target: &str,
) -> Result<TranslationResult, &'static str> {
    if !valid_text(text) || key.is_empty() || key.chars().any(char::is_control) {
        return Err("invalid_request");
    }
    let target_code = language_code(target)?;
    if source != "auto" {
        language_code(source)?;
    }
    let client = reqwest::Client::builder()
        .connect_timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| "network_or_timeout")?;
    tokio::time::timeout(Duration::from_secs(30), async {
        let first = request(&client, endpoint, key, text, source, target_code).await?;
        let detected = detected_language(&first.detected_source_language)?;
        if source == "auto" && target != "fr" && detected == target {
            let second = request(&client, endpoint, key, text, source, "FR").await?;
            Ok(TranslationResult {
                text: second.text,
                source: detected.into(),
                target: "fr".into(),
            })
        } else {
            Ok(TranslationResult {
                text: first.text,
                source: if source == "auto" { detected } else { source }.into(),
                target: target.into(),
            })
        }
    })
    .await
    .map_err(|_| "network_or_timeout")?
}

async fn request(
    client: &reqwest::Client,
    endpoint: &str,
    key: &str,
    text: &str,
    source: &str,
    target: &str,
) -> Result<DeeplTranslation, &'static str> {
    let target = match target {
        "ZH" => "ZH-HANS",
        "EN" => "EN-US",
        other => other,
    };
    let mut body = serde_json::json!({"text": [text], "target_lang": target});
    if source != "auto" {
        body["source_lang"] = language_code(source)?.into();
    }
    let mut response = client
        .post(endpoint)
        .header(
            reqwest::header::AUTHORIZATION,
            format!("DeepL-Auth-Key {key}"),
        )
        .json(&body)
        .send()
        .await
        .map_err(|_| "network_or_timeout")?;
    match response.status().as_u16() {
        200 => {}
        401 | 403 => return Err("auth_failed"),
        429 | 456 => return Err("quota_or_rate_limited"),
        400..=499 => return Err("invalid_request"),
        _ => return Err("network_or_timeout"),
    }
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await.map_err(|_| "network_or_timeout")? {
        if bytes.len() + chunk.len() > 256 * 1024 {
            return Err("invalid_response");
        }
        bytes.extend_from_slice(&chunk);
    }
    let mut decoded: DeeplResponse =
        serde_json::from_slice(&bytes).map_err(|_| "invalid_response")?;
    if decoded.translations.len() != 1 || decoded.translations[0].text.trim().is_empty() {
        return Err("invalid_response");
    }
    Ok(decoded.translations.remove(0))
}
