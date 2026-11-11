use std::net::Ipv4Addr;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::domain::errors::ApplicationError;

pub const MINIMUM_NAME_LENGTH: usize = 2;
pub const MAXIMUM_NAME_LENGTH: usize = 80;
pub const MINIMUM_SLUG_LENGTH: usize = 2;
pub const MAXIMUM_SLUG_LENGTH: usize = 64;
pub const MAXIMUM_DESCRIPTION_LENGTH: usize = 280;
pub const MAXIMUM_SEARCH_QUERY_LENGTH: usize = 64;
pub const MINIMUM_TCP_PORT: u16 = 1024;
pub const MAXIMUM_TCP_PORT: u16 = 65535;
pub const MAXIMUM_PAGE_SIZE: u32 = 200;
pub const DEFAULT_PAGE_SIZE: u32 = 50;
pub const MINIMUM_PAYLOAD_BYTES: usize = 1024 * 1024;
pub const MAXIMUM_PAYLOAD_BYTES: usize = 100 * 1024 * 1024;
pub const MINIMUM_RETENTION_DAYS: u64 = 1;
pub const MAXIMUM_RETENTION_DAYS: u64 = 3650;
pub const MINIMUM_SHUTDOWN_GRACE_SECONDS: u64 = 1;
pub const MAXIMUM_SHUTDOWN_GRACE_SECONDS: u64 = 60;

/// Returns the current UTC timestamp formatted as RFC 3339.
pub fn current_utc_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// Validates and trims a required human-readable entity name without control characters.
pub fn validate_name(raw_name: &str) -> Result<String, ApplicationError> {
    let trimmed = raw_name.trim();
    if trimmed.chars().any(char::is_control) {
        return Err(ApplicationError::Validation(
            "Name must not contain control or newline characters".to_string(),
        ));
    }
    let char_count = trimmed.chars().count();
    if !(MINIMUM_NAME_LENGTH..=MAXIMUM_NAME_LENGTH).contains(&char_count) {
        return Err(ApplicationError::Validation(format!(
            "Name length must be between {MINIMUM_NAME_LENGTH} and {MAXIMUM_NAME_LENGTH} characters"
        )));
    }
    Ok(trimmed.to_string())
}

/// Validates and trims an optional entity description without unsafe control bytes.
pub fn validate_description(raw_description: &str) -> Result<String, ApplicationError> {
    let normalized = raw_description.replace("\r\n", "\n");
    let trimmed = normalized.trim();
    if trimmed
        .chars()
        .any(|character| character.is_control() && character != '\n')
    {
        return Err(ApplicationError::Validation(
            "Description must not contain control characters".to_string(),
        ));
    }
    if trimmed.chars().count() > MAXIMUM_DESCRIPTION_LENGTH {
        return Err(ApplicationError::Validation(format!(
            "Description must be at most {MAXIMUM_DESCRIPTION_LENGTH} characters"
        )));
    }
    Ok(trimmed.to_string())
}

/// Converts Persian, Arabic-Indic, and Hindi decimal digits into standard ASCII English digits (`0`-`9`).
pub fn normalize_unicode_digits(raw_value: &str) -> String {
    raw_value
        .chars()
        .map(|character| match character {
            '\u{06F0}'..='\u{06F9}' => {
                char::from_u32(('0' as u32) + (character as u32 - 0x06F0)).unwrap_or(character)
            }
            '\u{0660}'..='\u{0669}' => {
                char::from_u32(('0' as u32) + (character as u32 - 0x0660)).unwrap_or(character)
            }
            '\u{0966}'..='\u{096F}' => {
                char::from_u32(('0' as u32) + (character as u32 - 0x0966)).unwrap_or(character)
            }
            _ => character,
        })
        .collect()
}

/// Strictly validates a lowercase ASCII hyphenated slug identifier.
pub fn validate_slug(raw_slug: &str) -> Result<String, ApplicationError> {
    let trimmed = raw_slug.trim();
    if trimmed.is_empty() {
        return Err(ApplicationError::Validation(format!(
            "Slug length must be between {MINIMUM_SLUG_LENGTH} and {MAXIMUM_SLUG_LENGTH} characters"
        )));
    }
    if !trimmed.is_ascii() {
        return Err(ApplicationError::Validation(
            "Slug must use English ASCII characters only".to_string(),
        ));
    }

    let length = trimmed.len();
    if !(MINIMUM_SLUG_LENGTH..=MAXIMUM_SLUG_LENGTH).contains(&length) {
        return Err(ApplicationError::Validation(format!(
            "Slug length must be between {MINIMUM_SLUG_LENGTH} and {MAXIMUM_SLUG_LENGTH} characters"
        )));
    }

    if !is_valid_slug_syntax(trimmed) {
        return Err(ApplicationError::Validation(
            "Slug must use lowercase English letters, digits, and single hyphens without leading or trailing hyphens".to_string(),
        ));
    }

    Ok(trimmed.to_string())
}

/// Derives a clean lowercase slug when callers omit an explicit slug.
pub fn derive_slug_or_validate(
    explicit_slug: Option<&str>,
    fallback_name: &str,
) -> Result<String, ApplicationError> {
    match explicit_slug {
        Some(raw_slug) if !raw_slug.trim().is_empty() => validate_slug(raw_slug),
        _ => {
            let normalized_digits = normalize_unicode_digits(fallback_name.trim());
            let mut candidate = String::new();
            let mut last_was_hyphen = true;
            for character in normalized_digits.chars() {
                if character.is_ascii_alphanumeric() {
                    candidate.push(character.to_ascii_lowercase());
                    last_was_hyphen = false;
                } else if !last_was_hyphen {
                    candidate.push('-');
                    last_was_hyphen = true;
                }
            }
            while candidate.ends_with('-') {
                candidate.pop();
            }
            if candidate.len() > MAXIMUM_SLUG_LENGTH {
                candidate.truncate(MAXIMUM_SLUG_LENGTH);
                while candidate.ends_with('-') {
                    candidate.pop();
                }
            }
            validate_slug(&candidate)
        }
    }
}

/// Validates a UUID string identifier.
pub fn validate_uuid(raw_identifier: &str, field_name: &str) -> Result<String, ApplicationError> {
    let trimmed = raw_identifier.trim();
    let parsed = Uuid::parse_str(trimmed)
        .map_err(|_| ApplicationError::Validation(format!("{field_name} must be a valid UUID")))?;
    Ok(parsed.to_string())
}

/// Validates a loopback or local IPv4 bind address, normalizing multi-script decimal digits to English ASCII.
pub fn validate_bind_address(raw_address: &str) -> Result<String, ApplicationError> {
    let normalized = normalize_unicode_digits(raw_address.trim());
    let parsed: Ipv4Addr = normalized.parse().map_err(|_| {
        ApplicationError::Validation("Bind address must be a valid IPv4 address".to_string())
    })?;
    Ok(parsed.to_string())
}

/// Validates an unprivileged TCP port number.
pub fn validate_tcp_port(port: u16, label: &str) -> Result<u16, ApplicationError> {
    if !(MINIMUM_TCP_PORT..=MAXIMUM_TCP_PORT).contains(&port) {
        return Err(ApplicationError::Validation(format!(
            "{label} port must be between {MINIMUM_TCP_PORT} and {MAXIMUM_TCP_PORT}"
        )));
    }
    Ok(port)
}

/// Validates optional inbox search text length and blocks control bytes.
pub fn validate_search_query(raw_query: Option<&str>) -> Result<Option<String>, ApplicationError> {
    let Some(query) = raw_query else {
        return Ok(None);
    };
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    if trimmed.chars().any(char::is_control) {
        return Err(ApplicationError::Validation(
            "Search query must not contain control characters".to_string(),
        ));
    }
    if trimmed.chars().count() > MAXIMUM_SEARCH_QUERY_LENGTH {
        return Err(ApplicationError::Validation(format!(
            "Search query must be at most {MAXIMUM_SEARCH_QUERY_LENGTH} characters"
        )));
    }
    Ok(Some(trimmed.to_string()))
}

/// Validates inbox pagination limit and offset bounds.
pub fn validate_pagination(
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<(u32, u32), ApplicationError> {
    let resolved_limit = limit.unwrap_or(DEFAULT_PAGE_SIZE);
    if !(1..=MAXIMUM_PAGE_SIZE).contains(&resolved_limit) {
        return Err(ApplicationError::Validation(format!(
            "Pagination limit must be between 1 and {MAXIMUM_PAGE_SIZE}"
        )));
    }
    Ok((resolved_limit, offset.unwrap_or(0)))
}

/// Validates a bounded retention period in days.
pub fn validate_retention_days(days: u64) -> Result<u64, ApplicationError> {
    if !(MINIMUM_RETENTION_DAYS..=MAXIMUM_RETENTION_DAYS).contains(&days) {
        return Err(ApplicationError::Validation(format!(
            "Retention days must be between {MINIMUM_RETENTION_DAYS} and {MAXIMUM_RETENTION_DAYS}"
        )));
    }
    Ok(days)
}

/// Validates server graceful shutdown timeout bounds.
pub fn validate_grace_period_seconds(seconds: u64) -> Result<u64, ApplicationError> {
    if !(MINIMUM_SHUTDOWN_GRACE_SECONDS..=MAXIMUM_SHUTDOWN_GRACE_SECONDS).contains(&seconds) {
        return Err(ApplicationError::Validation(format!(
            "Shutdown grace period must be between {MINIMUM_SHUTDOWN_GRACE_SECONDS} and {MAXIMUM_SHUTDOWN_GRACE_SECONDS} seconds"
        )));
    }
    Ok(seconds)
}

/// Validates an RFC 3339 timestamp string and normalizes it.
pub fn validate_rfc3339_timestamp(
    raw_timestamp: &str,
    field_name: &str,
) -> Result<String, ApplicationError> {
    let trimmed = raw_timestamp.trim();
    if trimmed.is_empty() {
        return Err(ApplicationError::Validation(format!(
            "{field_name} must not be empty"
        )));
    }
    DateTime::parse_from_rfc3339(trimmed).map_err(|_| {
        ApplicationError::Validation(format!("{field_name} must be a valid RFC-3339 timestamp"))
    })?;
    Ok(trimmed.to_string())
}

/// Validates a trimmed single-line bounded string without ASCII control characters.
pub fn validate_bounded_text(
    raw_value: &str,
    field_name: &str,
    min_length: usize,
    max_length: usize,
    allow_newlines: bool,
) -> Result<String, ApplicationError> {
    let trimmed = raw_value.trim();
    let has_invalid_control = trimmed.chars().any(|character| {
        if allow_newlines && (character == '\n' || character == '\r' || character == '\t') {
            return false;
        }
        character.is_control()
    });
    if has_invalid_control {
        return Err(ApplicationError::Validation(format!(
            "{field_name} must not contain control characters"
        )));
    }
    let length = trimmed.chars().count();
    if !(min_length..=max_length).contains(&length) {
        return Err(ApplicationError::Validation(format!(
            "{field_name} length must be between {min_length} and {max_length} characters"
        )));
    }
    Ok(trimmed.to_string())
}

fn is_valid_slug_syntax(slug: &str) -> bool {
    if slug.starts_with('-') || slug.ends_with('-') || slug.contains("--") {
        return false;
    }
    slug.bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}
