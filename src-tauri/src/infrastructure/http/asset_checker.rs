use std::sync::Arc;
use std::time::{Duration, Instant};

use regex::Regex;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

use crate::domain::asset_audit::{AssetCheckResult, AssetKind};
use crate::domain::errors::ApplicationError;

const MAXIMUM_CONCURRENT_CHECKS: usize = 5;
const ASSET_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const ASSET_USER_AGENT: &str = "NotifyTrap-AssetAuditor/1.5 (+https://github.com/farshidrezaei/notifytrap)";

/// Extracts candidate HTTP and HTTPS URLs from raw HTML for anchor links and images.
pub fn extract_candidate_urls(raw_html: &str) -> Vec<(String, AssetKind)> {
    let mut candidates = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // 1. Extract anchor hrefs
    let anchor_regex = Regex::new(r#"(?i)<a\b[^>]*\bhref\s*=\s*['"]([^'"]+)['"]"#)
        .expect("static regex pattern is valid");
    for captures in anchor_regex.captures_iter(raw_html) {
        if let Some(matched) = captures.get(1) {
            let url = matched.as_str().trim();
            if is_check_eligible_http_url(url) && seen.insert((url.to_string(), AssetKind::Link)) {
                candidates.push((url.to_string(), AssetKind::Link));
            }
        }
    }

    // 2. Extract image srcs
    let image_regex = Regex::new(r#"(?i)<img\b[^>]*\bsrc\s*=\s*['"]([^'"]+)['"]"#)
        .expect("static regex pattern is valid");
    for captures in image_regex.captures_iter(raw_html) {
        if let Some(matched) = captures.get(1) {
            let url = matched.as_str().trim();
            if is_check_eligible_http_url(url) && seen.insert((url.to_string(), AssetKind::Image)) {
                candidates.push((url.to_string(), AssetKind::Image));
            }
        }
    }

    candidates
}

fn is_check_eligible_http_url(url: &str) -> bool {
    let lower = url.to_lowercase();
    (lower.starts_with("http://") || lower.starts_with("https://"))
        && !lower.starts_with("cid:")
        && !lower.starts_with("javascript:")
        && !lower.starts_with("mailto:")
        && !lower.starts_with("tel:")
}

/// Concurrently validates status codes and availability of links and remote images inside an email.
pub async fn check_candidate_assets(
    candidates: Vec<(String, AssetKind)>,
) -> Result<Vec<AssetCheckResult>, ApplicationError> {
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let client = reqwest::Client::builder()
        .user_agent(ASSET_USER_AGENT)
        .timeout(ASSET_REQUEST_TIMEOUT)
        .redirect(reqwest::redirect::Policy::limited(3))
        .build()
        .map_err(|error| ApplicationError::Internal(format!("Failed to build HTTP auditor client: {error}")))?;

    let semaphore = Arc::new(Semaphore::new(MAXIMUM_CONCURRENT_CHECKS));
    let mut join_set = JoinSet::new();

    for (url, kind) in candidates {
        let client_clone = client.clone();
        let permit = semaphore.clone();

        join_set.spawn(async move {
            let _acquired = permit.acquire_owned().await;
            check_single_url(&client_clone, url, kind).await
        });
    }

    let mut results = Vec::new();
    while let Some(task_result) = join_set.join_next().await {
        match task_result {
            Ok(check_result) => results.push(check_result),
            Err(error) => {
                tracing::warn!("Asset check worker failed to join: {error}");
            }
        }
    }

    Ok(results)
}

async fn check_single_url(client: &reqwest::Client, url: String, kind: AssetKind) -> AssetCheckResult {
    let started_at = Instant::now();

    // 1. Try HEAD request first for fast headers-only inspection
    let head_attempt = client.head(&url).send().await;

    match head_attempt {
        Ok(response) => {
            let status = response.status().as_u16();
            let latency = started_at.elapsed().as_millis() as u64;

            // Some web servers return 405 Method Not Allowed or 403 on HEAD: fallback to lightweight GET
            if status == 405 || status == 403 {
                let get_attempt = client
                    .get(&url)
                    .header("Range", "bytes=0-1024")
                    .send()
                    .await;
                match get_attempt {
                    Ok(get_response) => {
                        let get_status = get_response.status().as_u16();
                        let total_latency = started_at.elapsed().as_millis() as u64;
                        let is_broken = get_status >= 400;
                        AssetCheckResult {
                            url,
                            kind,
                            status_code: Some(get_status),
                            is_broken,
                            latency_milliseconds: total_latency,
                            error_message: if is_broken {
                                Some(format!("HTTP status {get_status}"))
                            } else {
                                None
                            },
                        }
                    }
                    Err(error) => {
                        let total_latency = started_at.elapsed().as_millis() as u64;
                        AssetCheckResult {
                            url,
                            kind,
                            status_code: None,
                            is_broken: true,
                            latency_milliseconds: total_latency,
                            error_message: Some(error.to_string()),
                        }
                    }
                }
            } else {
                let is_broken = status >= 400;
                AssetCheckResult {
                    url,
                    kind,
                    status_code: Some(status),
                    is_broken,
                    latency_milliseconds: latency,
                    error_message: if is_broken {
                        Some(format!("HTTP status {status}"))
                    } else {
                        None
                    },
                }
            }
        }
        Err(error) => {
            let latency = started_at.elapsed().as_millis() as u64;
            AssetCheckResult {
                url,
                kind,
                status_code: None,
                is_broken: true,
                latency_milliseconds: latency,
                error_message: Some(error.to_string()),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_http_and_https_links_and_images_while_ignoring_cid_and_anchors() {
        let sample_html = r##"
            <div>
                <a href="https://example.com/login">Login</a>
                <a href="http://local.test/terms">Terms</a>
                <a href="#section-2">Anchor</a>
                <a href="mailto:support@example.com">Email</a>
                <a href="javascript:void(0)">JS</a>
                <img src="https://cdn.example.com/logo.png" alt="Logo" />
                <img src="cid:inline-attachment-uuid" alt="Embedded" />
            </div>
        "##;

        let candidates = extract_candidate_urls(sample_html);
        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0], ("https://example.com/login".to_string(), AssetKind::Link));
        assert_eq!(candidates[1], ("http://local.test/terms".to_string(), AssetKind::Link));
        assert_eq!(candidates[2], ("https://cdn.example.com/logo.png".to_string(), AssetKind::Image));
    }
}
