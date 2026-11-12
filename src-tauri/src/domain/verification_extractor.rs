use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::domain::channel::ChannelType;
use crate::domain::validation::normalize_unicode_digits;

const MAXIMUM_ARTIFACTS_PER_NOTIFICATION: usize = 8;
const OTP_MINIMUM_DIGITS: usize = 4;
const OTP_MAXIMUM_DIGITS: usize = 8;
const OTP_CONTEXT_WINDOW_WORDS: usize = 8;

/// Category of actionable verification token extracted from a captured notification.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum VerificationArtifactKind {
    OtpCode,
    VerificationLink,
}

/// Extracted OTP code or verification link ready for one-click copy or test assertions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct VerificationArtifact {
    pub kind: VerificationArtifactKind,
    pub value: String,
    pub label: String,
}

/// Extracts deduplicated OTP codes and verification/magic links from a notification payload.
pub fn extract_verification_artifacts(
    channel_type: ChannelType,
    subject: &str,
    preview_text: &str,
    payload_json: &Value,
    raw_payload: &str,
) -> Vec<VerificationArtifact> {
    let mut candidate_texts = Vec::new();
    collect_priority_texts(
        channel_type,
        subject,
        preview_text,
        payload_json,
        raw_payload,
        &mut candidate_texts,
    );

    let mut artifacts = Vec::new();
    let mut seen_keys: HashSet<(VerificationArtifactKind, String)> = HashSet::new();

    for text in &candidate_texts {
        if artifacts.len() >= MAXIMUM_ARTIFACTS_PER_NOTIFICATION {
            break;
        }
        for code in extract_otp_codes_from_text(text, channel_type) {
            if artifacts.len() >= MAXIMUM_ARTIFACTS_PER_NOTIFICATION {
                break;
            }
            if seen_keys.insert((VerificationArtifactKind::OtpCode, code.clone())) {
                artifacts.push(VerificationArtifact {
                    kind: VerificationArtifactKind::OtpCode,
                    value: code,
                    label: "OTP".to_string(),
                });
            }
        }
    }

    for text in &candidate_texts {
        if artifacts.len() >= MAXIMUM_ARTIFACTS_PER_NOTIFICATION {
            break;
        }
        for link in extract_verification_links_from_text(text) {
            if artifacts.len() >= MAXIMUM_ARTIFACTS_PER_NOTIFICATION {
                break;
            }
            if seen_keys.insert((VerificationArtifactKind::VerificationLink, link.clone())) {
                artifacts.push(VerificationArtifact {
                    kind: VerificationArtifactKind::VerificationLink,
                    value: link,
                    label: "Link".to_string(),
                });
            }
        }
    }

    artifacts
}

fn collect_priority_texts(
    channel_type: ChannelType,
    subject: &str,
    preview_text: &str,
    payload_json: &Value,
    raw_payload: &str,
    output: &mut Vec<String>,
) {
    if !subject.trim().is_empty() {
        output.push(subject.to_string());
    }

    match channel_type {
        ChannelType::Email => {
            push_json_string(payload_json, "textBody", output);
            if let Some(html) = payload_json
                .get("sanitizedHtml")
                .and_then(Value::as_str)
                .filter(|value| !value.trim().is_empty())
            {
                output.push(html.to_string());
                output.push(strip_html_tags(html));
            }
        }
        ChannelType::Sms => {
            push_json_string(payload_json, "message", output);
        }
        ChannelType::Push => {
            push_json_string(payload_json, "title", output);
            push_json_string(payload_json, "body", output);
            if let Some(data) = payload_json.get("data") {
                collect_json_strings(data, output, 0);
            }
        }
        ChannelType::Webhook => {
            if let Some(json_body) = payload_json.get("jsonBody") {
                collect_json_strings(json_body, output, 0);
            }
            push_json_string(payload_json, "textBody", output);
        }
    }

    if output.is_empty() && !preview_text.trim().is_empty() {
        output.push(preview_text.to_string());
    }
    if output.is_empty() && !raw_payload.trim().is_empty() {
        output.push(raw_payload.to_string());
    }
}

fn push_json_string(object: &Value, field: &str, output: &mut Vec<String>) {
    if let Some(value) = object
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        output.push(value.to_string());
    }
}

fn collect_json_strings(value: &Value, output: &mut Vec<String>, depth: usize) {
    if depth > 6 {
        return;
    }
    match value {
        Value::String(text) => {
            let trimmed = text.trim();
            if !trimmed.is_empty() {
                output.push(trimmed.to_string());
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_json_strings(item, output, depth + 1);
            }
        }
        Value::Object(map) => {
            for (key, nested) in map {
                if key.eq_ignore_ascii_case("headers") {
                    continue;
                }
                collect_json_strings(nested, output, depth + 1);
            }
        }
        _ => {}
    }
}

fn extract_otp_codes_from_text(raw_text: &str, channel_type: ChannelType) -> Vec<String> {
    let cleaned_text = if raw_text.contains('<') && raw_text.contains('>') {
        strip_html_tags(raw_text)
    } else {
        raw_text.to_string()
    };
    let normalized = normalize_unicode_digits(&cleaned_text);
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut codes = Vec::new();

    for (index, token) in tokens.iter().enumerate() {
        if contains_url_prefix(token) || token.starts_with('+') || token.starts_with('#') {
            continue;
        }

        let Some(candidate_digits) = parse_otp_token(token) else {
            continue;
        };

        let is_likely_year = candidate_digits.len() == 4
            && (candidate_digits.starts_with("19") || candidate_digits.starts_with("20"));

        let window_start = index.saturating_sub(OTP_CONTEXT_WINDOW_WORDS);
        let window_end = (index + OTP_CONTEXT_WINDOW_WORDS + 1).min(tokens.len());
        let has_keyword_context = tokens[window_start..window_end]
            .iter()
            .any(|neighbor| is_otp_context_keyword(neighbor));

        if has_keyword_context || (channel_type == ChannelType::Sms && !is_likely_year) {
            codes.push(candidate_digits);
        }
    }

    codes
}

fn parse_otp_token(raw_token: &str) -> Option<String> {
    let trimmed = raw_token.trim_matches(|character: char| {
        matches!(
            character,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | ')'
                | '('
                | '['
                | ']'
                | '"'
                | '\''
                | '«'
                | '»'
                | '،'
                | '؛'
        )
    });

    if trimmed.is_empty() {
        return None;
    }

    if trimmed.bytes().all(|byte| byte.is_ascii_digit())
        && (OTP_MINIMUM_DIGITS..=OTP_MAXIMUM_DIGITS).contains(&trimmed.len())
    {
        return Some(trimmed.to_string());
    }

    if let Some((left, right)) = trimmed.split_once('-')
        && left.len() == 3
        && right.len() == 3
        && left.bytes().all(|byte| byte.is_ascii_digit())
        && right.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Some(format!("{left}{right}"));
    }

    None
}

fn is_otp_context_keyword(raw_word: &str) -> bool {
    let cleaned = raw_word
        .trim_matches(|character: char| !character.is_alphanumeric() && character != '-')
        .to_ascii_lowercase();

    if matches!(
        cleaned.as_str(),
        "code"
            | "otp"
            | "pin"
            | "passcode"
            | "verify"
            | "verification"
            | "confirm"
            | "confirmation"
            | "security"
            | "one-time"
            | "2fa"
            | "mfa"
            | "token"
            | "login"
            | "sign-in"
            | "signin"
            | "auth"
    ) {
        return true;
    }

    matches!(
        raw_word.trim_matches(|character: char| matches!(character, ':' | '：' | '.' | '،' | '!')),
        "کد" | "تایید" | "تأیید" | "رمز" | "ورود" | "اعتبارسنجی" | "احراز"
    )
}

fn extract_verification_links_from_text(raw_text: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut search_offset = 0;

    while let Some(relative_position) = find_next_http_scheme(&raw_text[search_offset..]) {
        let absolute_start = search_offset + relative_position;
        let remainder = &raw_text[absolute_start..];
        let raw_url = remainder
            .split(|character: char| {
                character.is_whitespace() || matches!(character, '"' | '\'' | '<' | '>' | '`')
            })
            .next()
            .unwrap_or_default();

        let trimmed_url = raw_url
            .trim_end_matches(|character: char| {
                matches!(character, '.' | ',' | ';' | ':' | '!' | '?' | ')' | ']')
            })
            .replace("&amp;", "&");

        if trimmed_url.len() > "https://x".len() && is_verification_url(&trimmed_url) {
            links.push(trimmed_url);
        }

        search_offset = absolute_start + raw_url.len().max(1);
    }

    links
}

fn find_next_http_scheme(slice: &str) -> Option<usize> {
    let https_position = slice.find("https://");
    let http_position = slice.find("http://");
    match (https_position, http_position) {
        (Some(first), Some(second)) => Some(first.min(second)),
        (Some(first), None) => Some(first),
        (None, Some(second)) => Some(second),
        (None, None) => None,
    }
}

fn contains_url_prefix(token: &str) -> bool {
    token.contains("http://") || token.contains("https://")
}

fn is_verification_url(url: &str) -> bool {
    let lowercase = url.to_ascii_lowercase();
    const INDICATORS: &[&str] = &[
        "verify",
        "confirm",
        "token",
        "magic",
        "auth",
        "reset",
        "activate",
        "activation",
        "callback",
        "code=",
        "otp=",
        "invite",
        "signin",
        "login",
    ];
    INDICATORS
        .iter()
        .any(|indicator| lowercase.contains(indicator))
}

fn strip_html_tags(html: &str) -> String {
    let mut plain = String::with_capacity(html.len());
    let mut inside_tag = false;
    for character in html.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => {
                inside_tag = false;
                plain.push(' ');
            }
            _ if !inside_tag => plain.push(character),
            _ => {}
        }
    }
    plain
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::{VerificationArtifactKind, extract_verification_artifacts};
    use crate::domain::channel::ChannelType;

    #[test]
    fn extracts_english_and_persian_otp_codes_and_magic_links() {
        let artifacts = extract_verification_artifacts(
            ChannelType::Email,
            "Your verification code",
            "",
            &json!({
                "textBody": "کد تایید شما: ۴۸۲۹۱۰\nOr click https://app.example.test/auth/verify?token=abc123&amp;next=/dashboard.",
                "sanitizedHtml": "<p style=\"color:#123456\">Copyright 2026</p>",
            }),
            "",
        );

        assert!(artifacts.iter().any(|artifact| {
            artifact.kind == VerificationArtifactKind::OtpCode && artifact.value == "482910"
        }));
        assert!(artifacts.iter().any(|artifact| {
            artifact.kind == VerificationArtifactKind::VerificationLink
                && artifact.value
                    == "https://app.example.test/auth/verify?token=abc123&next=/dashboard"
        }));
        assert!(!artifacts.iter().any(|artifact| artifact.value == "123456"));
        assert!(!artifacts.iter().any(|artifact| artifact.value == "2026"));
    }

    #[test]
    fn extracts_dashed_otp_and_sms_standalone_code() {
        let artifacts = extract_verification_artifacts(
            ChannelType::Sms,
            "SMS",
            "",
            &json!({
                "message": "Use 741-852 to sign in to NotifyTrap."
            }),
            "",
        );

        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].kind, VerificationArtifactKind::OtpCode);
        assert_eq!(artifacts[0].value, "741852");
    }

    #[test]
    fn ignores_non_verification_links_and_phone_numbers() {
        let artifacts = extract_verification_artifacts(
            ChannelType::Sms,
            "SMS",
            "",
            &json!({
                "message": "Support +989123456789 at https://example.test/docs/pricing"
            }),
            "",
        );

        assert!(artifacts.is_empty());
    }

    proptest! {
        #[test]
        fn otp_extraction_never_panics_and_deduplicates(
            digits in "[0-9]{4,8}",
            prefix in "[a-zA-Z0-9 ]{0,20}",
        ) {
            let body = format!("{prefix} verification code: {digits} repeat code: {digits}");
            let artifacts = extract_verification_artifacts(
                ChannelType::Email,
                "Verify",
                "",
                &json!({ "textBody": body }),
                "",
            );
            let otp_matches: Vec<_> = artifacts
                .iter()
                .filter(|artifact| artifact.kind == VerificationArtifactKind::OtpCode && artifact.value == digits)
                .collect();
            prop_assert_eq!(otp_matches.len(), 1);
        }
    }
}
