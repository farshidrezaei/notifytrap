use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::time::{Duration, Instant};

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::domain::errors::ApplicationError;
use crate::domain::notification::{ReplayHeaderEntry, WebhookReplayResult};

const REPLAY_CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const REPLAY_IO_TIMEOUT: Duration = Duration::from_secs(10);
const MAXIMUM_REPLAY_RESPONSE_BYTES: usize = 1_048_576;

/// Validated target restricted strictly to local loopback addresses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedLoopbackTarget {
    pub normalized_url: String,
    pub socket_address: SocketAddr,
    pub host_header: String,
    pub request_path: String,
}

/// Validates that a webhook replay URL targets local loopback (`127.0.0.1`, `localhost`, `[::1]`) over `http://`.
pub fn validate_loopback_target_url(
    raw_url: &str,
) -> Result<ParsedLoopbackTarget, ApplicationError> {
    let trimmed = raw_url.trim();
    if trimmed.is_empty() {
        return Err(ApplicationError::Validation(
            "Replay target URL must not be empty".to_string(),
        ));
    }
    if trimmed.contains(['\r', '\n', '\0']) {
        return Err(ApplicationError::Validation(
            "Replay target URL contains invalid control characters".to_string(),
        ));
    }

    let without_scheme = trimmed.strip_prefix("http://").ok_or_else(|| {
        ApplicationError::Validation(
            "Replay target URL must use http:// and target a local loopback address".to_string(),
        )
    })?;

    if without_scheme.contains('#') {
        return Err(ApplicationError::Validation(
            "Replay target URL must not contain a URL fragment".to_string(),
        ));
    }

    let (authority, request_path) = match without_scheme.find(['/', '?']) {
        Some(separator_index) => {
            let (auth_part, rest_part) = without_scheme.split_at(separator_index);
            let normalized_path = if rest_part.starts_with('?') {
                format!("/{rest_part}")
            } else {
                rest_part.to_string()
            };
            (auth_part, normalized_path)
        }
        None => (without_scheme, "/".to_string()),
    };

    if authority.is_empty() || authority.contains('@') {
        return Err(ApplicationError::Validation(
            "Replay target URL authority is invalid".to_string(),
        ));
    }

    let (host, port) = parse_loopback_authority(authority)?;
    let ip_address = match host.as_str() {
        "127.0.0.1" | "localhost" => IpAddr::V4(Ipv4Addr::LOCALHOST),
        "::1" => IpAddr::V6(Ipv6Addr::LOCALHOST),
        _ => {
            return Err(ApplicationError::Validation(
                "Replay target host must be 127.0.0.1, localhost, or [::1]".to_string(),
            ));
        }
    };

    let host_header = if host == "::1" {
        if port == 80 {
            "[::1]".to_string()
        } else {
            format!("[::1]:{port}")
        }
    } else if port == 80 {
        host.clone()
    } else {
        format!("{host}:{port}")
    };

    Ok(ParsedLoopbackTarget {
        normalized_url: format!("http://{host_header}{request_path}"),
        socket_address: SocketAddr::new(ip_address, port),
        host_header,
        request_path,
    })
}

fn parse_loopback_authority(authority: &str) -> Result<(String, u16), ApplicationError> {
    if let Some(after_open) = authority.strip_prefix('[') {
        let close_index = after_open.find(']').ok_or_else(|| {
            ApplicationError::Validation("Malformed IPv6 loopback host".to_string())
        })?;
        let ipv6_host = after_open[..close_index].to_ascii_lowercase();
        let remainder = &after_open[close_index + 1..];
        let port = if remainder.is_empty() {
            80
        } else if let Some(raw_port) = remainder.strip_prefix(':') {
            parse_port_number(raw_port)?
        } else {
            return Err(ApplicationError::Validation(
                "Invalid port suffix in IPv6 loopback authority".to_string(),
            ));
        };
        return Ok((ipv6_host, port));
    }

    if let Some((raw_host, raw_port)) = authority.rsplit_once(':') {
        if raw_host.contains(':') {
            return Err(ApplicationError::Validation(
                "IPv6 hosts must be wrapped in square brackets".to_string(),
            ));
        }
        return Ok((raw_host.to_ascii_lowercase(), parse_port_number(raw_port)?));
    }

    Ok((authority.to_ascii_lowercase(), 80))
}

fn parse_port_number(raw_port: &str) -> Result<u16, ApplicationError> {
    let port: u16 = raw_port.parse().map_err(|_| {
        ApplicationError::Validation("Replay target port must be between 1 and 65535".to_string())
    })?;
    if port == 0 {
        return Err(ApplicationError::Validation(
            "Replay target port must be between 1 and 65535".to_string(),
        ));
    }
    Ok(port)
}

/// Sends a captured webhook payload over TCP strictly to a validated loopback socket address.
pub async fn execute_loopback_http_replay(
    target: &ParsedLoopbackTarget,
    method: &str,
    captured_headers: &[(String, String)],
    body_text: &str,
) -> Result<WebhookReplayResult, ApplicationError> {
    let normalized_method = validate_http_method(method)?;
    let started_at = Instant::now();

    let mut stream = tokio::time::timeout(
        REPLAY_CONNECT_TIMEOUT,
        TcpStream::connect(target.socket_address),
    )
    .await
    .map_err(|_| {
        ApplicationError::Internal(format!(
            "Timed out connecting to loopback target {}",
            target.socket_address
        ))
    })?
    .map_err(|error| {
        ApplicationError::Internal(format!(
            "Failed to connect to loopback target {}: {error}",
            target.socket_address
        ))
    })?;

    let mut request_head = format!(
        "{normalized_method} {} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\n",
        target.request_path, target.host_header
    );

    let mut has_content_type = false;
    for (header_name, header_value) in captured_headers {
        let lower_name = header_name.trim().to_ascii_lowercase();
        if lower_name.is_empty()
            || matches!(
                lower_name.as_str(),
                "host" | "content-length" | "connection" | "transfer-encoding"
            )
            || header_value == "[REDACTED]"
            || header_name.contains(['\r', '\n', ':'])
            || header_value.contains(['\r', '\n'])
        {
            continue;
        }
        if lower_name == "content-type" {
            has_content_type = true;
        }
        request_head.push_str(&format!(
            "{}: {}\r\n",
            header_name.trim(),
            header_value.trim()
        ));
    }

    let body_bytes = body_text.as_bytes();
    if !body_bytes.is_empty() && !has_content_type {
        request_head.push_str("Content-Type: application/json\r\n");
    }
    request_head.push_str(&format!("Content-Length: {}\r\n\r\n", body_bytes.len()));

    tokio::time::timeout(REPLAY_IO_TIMEOUT, async {
        stream.write_all(request_head.as_bytes()).await?;
        if !body_bytes.is_empty() {
            stream.write_all(body_bytes).await?;
        }
        stream.flush().await?;
        Ok::<(), std::io::Error>(())
    })
    .await
    .map_err(|_| ApplicationError::Internal("Timed out writing HTTP replay request".to_string()))?
    .map_err(|error| {
        ApplicationError::Internal(format!("Failed writing HTTP replay request: {error}"))
    })?;

    let raw_response = tokio::time::timeout(REPLAY_IO_TIMEOUT, read_bounded_response(&mut stream))
        .await
        .map_err(|_| {
            ApplicationError::Internal("Timed out waiting for loopback HTTP response".to_string())
        })?
        .map_err(|error| {
            ApplicationError::Internal(format!("Failed reading loopback HTTP response: {error}"))
        })?;

    let duration_milliseconds = u64::try_from(started_at.elapsed().as_millis()).unwrap_or(u64::MAX);
    parse_http_response(
        &target.normalized_url,
        &normalized_method,
        duration_milliseconds,
        &raw_response,
    )
}

fn validate_http_method(raw_method: &str) -> Result<String, ApplicationError> {
    let trimmed = raw_method.trim().to_ascii_uppercase();
    if trimmed.is_empty()
        || !trimmed
            .chars()
            .all(|character| character.is_ascii_alphabetic())
    {
        return Err(ApplicationError::Validation(
            "Invalid HTTP method for webhook replay".to_string(),
        ));
    }
    Ok(trimmed)
}

async fn read_bounded_response(stream: &mut TcpStream) -> Result<Vec<u8>, std::io::Error> {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 8192];
    loop {
        let read_bytes = stream.read(&mut chunk).await?;
        if read_bytes == 0 {
            break;
        }
        let remaining_capacity = MAXIMUM_REPLAY_RESPONSE_BYTES.saturating_sub(buffer.len());
        if remaining_capacity == 0 {
            break;
        }
        let take_bytes = read_bytes.min(remaining_capacity);
        buffer.extend_from_slice(&chunk[..take_bytes]);
        if buffer.len() >= MAXIMUM_REPLAY_RESPONSE_BYTES {
            break;
        }
    }
    Ok(buffer)
}

fn parse_http_response(
    target_url: &str,
    method: &str,
    duration_milliseconds: u64,
    raw_bytes: &[u8],
) -> Result<WebhookReplayResult, ApplicationError> {
    let header_end = raw_bytes
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .ok_or_else(|| {
            ApplicationError::Internal(
                "Loopback target returned a malformed HTTP response".to_string(),
            )
        })?;

    let head_text = String::from_utf8_lossy(&raw_bytes[..header_end]);
    let mut lines = head_text.split("\r\n");
    let status_line = lines.next().ok_or_else(|| {
        ApplicationError::Internal("Loopback target returned an empty HTTP status line".to_string())
    })?;

    let status_code = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|token| token.parse::<u16>().ok())
        .ok_or_else(|| {
            ApplicationError::Internal(format!(
                "Could not parse HTTP status code from '{status_line}'"
            ))
        })?;

    let mut response_headers = Vec::new();
    let mut is_chunked = false;
    for header_line in lines {
        if let Some((name, value)) = header_line.split_once(':') {
            let trimmed_name = name.trim().to_string();
            let trimmed_value = value.trim().to_string();
            if trimmed_name.eq_ignore_ascii_case("transfer-encoding")
                && trimmed_value.to_ascii_lowercase().contains("chunked")
            {
                is_chunked = true;
            }
            response_headers.push(ReplayHeaderEntry {
                name: trimmed_name,
                value: trimmed_value,
            });
        }
    }

    let raw_body = &raw_bytes[header_end + 4..];
    let decoded_body = if is_chunked {
        decode_chunked_body(raw_body)
    } else {
        raw_body.to_vec()
    };

    Ok(WebhookReplayResult {
        target_url: target_url.to_string(),
        method: method.to_string(),
        status_code,
        duration_milliseconds,
        response_headers,
        response_body: String::from_utf8_lossy(&decoded_body).to_string(),
    })
}

fn decode_chunked_body(raw_chunked: &[u8]) -> Vec<u8> {
    let mut output = Vec::new();
    let mut cursor = 0_usize;
    while cursor < raw_chunked.len() {
        let Some(relative_line_end) = raw_chunked[cursor..]
            .windows(2)
            .position(|window| window == b"\r\n")
        else {
            break;
        };
        let size_line = String::from_utf8_lossy(&raw_chunked[cursor..cursor + relative_line_end]);
        let hex_size = size_line.split(';').next().unwrap_or("").trim();
        let Ok(chunk_size) = usize::from_str_radix(hex_size, 16) else {
            break;
        };
        cursor += relative_line_end + 2;
        if chunk_size == 0 || cursor + chunk_size > raw_chunked.len() {
            break;
        }
        output.extend_from_slice(&raw_chunked[cursor..cursor + chunk_size]);
        cursor += chunk_size;
        if raw_chunked.get(cursor..cursor + 2) == Some(b"\r\n") {
            cursor += 2;
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::validate_loopback_target_url;

    #[test]
    fn allows_strict_loopback_targets_and_rejects_external_hosts() {
        let valid_ipv4 = validate_loopback_target_url("http://127.0.0.1:3001/hooks/stripe?ok=1")
            .expect("127.0.0.1 loopback should succeed");
        assert_eq!(
            valid_ipv4.normalized_url,
            "http://127.0.0.1:3001/hooks/stripe?ok=1"
        );
        assert_eq!(valid_ipv4.request_path, "/hooks/stripe?ok=1");

        let valid_localhost = validate_loopback_target_url("http://localhost:8080")
            .expect("localhost should succeed");
        assert_eq!(valid_localhost.normalized_url, "http://localhost:8080/");

        let valid_ipv6 =
            validate_loopback_target_url("http://[::1]:9000/callback").expect("[::1] should work");
        assert_eq!(valid_ipv6.normalized_url, "http://[::1]:9000/callback");

        for rejected in [
            "",
            "https://127.0.0.1:3000/hook",
            "http://example.com/hook",
            "http://192.168.1.10:3000/hook",
            "http://0.0.0.0:3000/hook",
            "http://user@127.0.0.1:3000/hook",
            "http://127.0.0.1:0/hook",
            "http://127.0.0.1:3000/hook#frag",
        ] {
            assert!(
                validate_loopback_target_url(rejected).is_err(),
                "expected '{rejected}' to be rejected"
            );
        }
    }
}
