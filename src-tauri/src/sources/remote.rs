//! Remote fetcher — HTTP GET with redirect tracking and timeouts.

use crate::error::{AppError, AppResult};
use crate::model::RemoteFetch;
use std::time::Duration;

const MAX_BODY_BYTES: usize = 8 * 1024 * 1024; // 8 MiB cap — plenty for any README.
const TIMEOUT: Duration = Duration::from_secs(20);

pub async fn fetch(client: &reqwest::Client, url: &str) -> AppResult<RemoteFetch> {
    let res = client
        .get(url)
        .timeout(TIMEOUT)
        .send()
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;

    let status = res.status();
    if !status.is_success() {
        return Err(AppError::Http(format!(
            "{} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        )));
    }

    let final_url = res.url().to_string();
    let content_type = res
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let bytes = res
        .bytes()
        .await
        .map_err(|e| AppError::Http(e.to_string()))?;
    if bytes.len() > MAX_BODY_BYTES {
        return Err(AppError::Http(format!(
            "response too large: {} bytes",
            bytes.len()
        )));
    }

    let text = String::from_utf8_lossy(&bytes).into_owned();
    Ok(RemoteFetch {
        text,
        final_url,
        content_type,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn client() -> reqwest::Client {
        reqwest::Client::builder()
            .user_agent("vidi-test")
            .build()
            .unwrap()
    }

    #[tokio::test]
    async fn fetch_happy_path() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/a.md"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string("# Hi")
                    .insert_header("content-type", "text/markdown"),
            )
            .mount(&server)
            .await;

        let url = format!("{}/a.md", server.uri());
        let r = fetch(&client(), &url).await.unwrap();
        assert_eq!(r.text, "# Hi");
        assert!(r
            .content_type
            .as_deref()
            .is_some_and(|c| c.contains("markdown") || c.contains("plain")));
        assert!(r.final_url.ends_with("/a.md"));
    }

    #[tokio::test]
    async fn fetch_404_is_error() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/missing"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&server)
            .await;

        let url = format!("{}/missing", server.uri());
        let err = fetch(&client(), &url).await.unwrap_err();
        match err {
            AppError::Http(m) => assert!(m.starts_with("404")),
            _ => panic!("expected http error"),
        }
    }

    #[tokio::test]
    async fn fetch_redirect_updates_final_url() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/a"))
            .respond_with(ResponseTemplate::new(302).insert_header("location", "/b"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/b"))
            .respond_with(ResponseTemplate::new(200).set_body_string("redirected"))
            .mount(&server)
            .await;

        let url = format!("{}/a", server.uri());
        let r = fetch(&client(), &url).await.unwrap();
        assert_eq!(r.text, "redirected");
        assert!(r.final_url.ends_with("/b"));
    }
}
