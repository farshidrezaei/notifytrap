use regex::{Regex, RegexBuilder};

use crate::domain::channel::ChannelType;
use crate::domain::notification::{NotificationStatus, NotificationSummary};

const MAXIMUM_REGEX_PATTERN_BYTES: usize = 200;
const MAXIMUM_COMPILED_REGEX_BYTES: usize = 65_536;

/// Represents a parsed structured search query combining SQL-pushable filters
/// with optional in-memory regex and OTP artifact predicates.
#[derive(Debug, Clone, Default)]
pub struct StructuredSearchQuery {
    pub include_channels: Vec<ChannelType>,
    pub exclude_channels: Vec<ChannelType>,
    pub include_statuses: Vec<NotificationStatus>,
    pub exclude_statuses: Vec<NotificationStatus>,
    pub include_run_identifiers: Vec<String>,
    pub exclude_run_identifiers: Vec<String>,
    pub include_recipients: Vec<String>,
    pub exclude_recipients: Vec<String>,
    pub include_senders: Vec<String>,
    pub exclude_senders: Vec<String>,
    pub include_subjects: Vec<String>,
    pub exclude_subjects: Vec<String>,
    pub include_terms: Vec<String>,
    pub exclude_terms: Vec<String>,
    pub has_otp: Option<bool>,
    pub regex_patterns: Vec<Regex>,
}

impl StructuredSearchQuery {
    /// Returns true when the query requires post-SQL inspection of notification summaries.
    pub fn requires_summary_post_filter(&self) -> bool {
        self.has_otp.is_some() || !self.regex_patterns.is_empty()
    }

    /// Evaluates post-SQL predicates (`otp:` and `/regex/`) against a notification summary.
    pub fn matches_summary(&self, summary: &NotificationSummary) -> bool {
        if let Some(expected_otp) = self.has_otp {
            let has_artifacts = !summary.verification_artifacts.is_empty();
            if has_artifacts != expected_otp {
                return false;
            }
        }

        if !self.regex_patterns.is_empty() {
            let searchable_text = format!(
                "{}\n{}\n{}\n{}\n{}",
                summary.recipient,
                summary.sender,
                summary.subject,
                summary.preview_text,
                summary.test_run_identifier
            );
            for pattern in &self.regex_patterns {
                if !pattern.is_match(&searchable_text) {
                    return false;
                }
            }
        }

        true
    }
}

/// Parses a user-entered search string into structured qualifiers, quoted phrases,
/// negations (`-...`), and `/regex/` patterns without ever failing on partial syntax.
pub fn parse_structured_search_query(raw_input: &str) -> StructuredSearchQuery {
    let mut parsed = StructuredSearchQuery::default();
    let tokens = tokenize_search_input(raw_input);

    for token in tokens {
        if token.is_empty() {
            continue;
        }

        let (is_negated, body) = if let Some(stripped) = token.strip_prefix('-') {
            if stripped.is_empty() {
                (false, token.as_str())
            } else {
                (true, stripped)
            }
        } else {
            (false, token.as_str())
        };

        if !is_negated
            && let Some(regex_body) = unwrap_regex_literal(body)
            && let Some(compiled) = compile_bounded_regex(regex_body)
        {
            parsed.regex_patterns.push(compiled);
            continue;
        }

        if let Some((qualifier, raw_value)) = body.split_once(':') {
            let value = strip_matching_quotes(raw_value.trim());
            if !value.is_empty() && apply_qualifier_token(&mut parsed, qualifier, value, is_negated)
            {
                continue;
            }
        }

        let plain_value = strip_matching_quotes(body);
        if !plain_value.is_empty() {
            if is_negated {
                parsed.exclude_terms.push(plain_value.to_string());
            } else {
                parsed.include_terms.push(plain_value.to_string());
            }
        }
    }

    parsed
}

fn apply_qualifier_token(
    parsed: &mut StructuredSearchQuery,
    qualifier: &str,
    value: &str,
    is_negated: bool,
) -> bool {
    match qualifier.to_ascii_lowercase().as_str() {
        "channel" | "type" => {
            if let Ok(channel_type) = ChannelType::parse(&value.to_ascii_lowercase()) {
                if is_negated {
                    parsed.exclude_channels.push(channel_type);
                } else {
                    parsed.include_channels.push(channel_type);
                }
                true
            } else {
                false
            }
        }
        "status" => {
            if let Ok(status) = NotificationStatus::parse(&value.to_ascii_lowercase()) {
                if is_negated {
                    parsed.exclude_statuses.push(status);
                } else {
                    parsed.include_statuses.push(status);
                }
                true
            } else {
                false
            }
        }
        "run" | "tag" => {
            if is_negated {
                parsed.exclude_run_identifiers.push(value.to_string());
            } else {
                parsed.include_run_identifiers.push(value.to_string());
            }
            true
        }
        "to" | "recipient" => {
            if is_negated {
                parsed.exclude_recipients.push(value.to_string());
            } else {
                parsed.include_recipients.push(value.to_string());
            }
            true
        }
        "from" | "sender" => {
            if is_negated {
                parsed.exclude_senders.push(value.to_string());
            } else {
                parsed.include_senders.push(value.to_string());
            }
            true
        }
        "subject" => {
            if is_negated {
                parsed.exclude_subjects.push(value.to_string());
            } else {
                parsed.include_subjects.push(value.to_string());
            }
            true
        }
        "otp" => match value.to_ascii_lowercase().as_str() {
            "true" | "1" | "yes" => {
                parsed.has_otp = Some(!is_negated);
                true
            }
            "false" | "0" | "no" => {
                parsed.has_otp = Some(is_negated);
                true
            }
            _ => false,
        },
        _ => false,
    }
}

fn tokenize_search_input(raw_input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut active_quote: Option<char> = None;
    let mut inside_regex = false;

    for character in raw_input.chars() {
        if let Some(quote_char) = active_quote {
            if character == quote_char {
                active_quote = None;
            } else {
                current.push(character);
            }
            continue;
        }

        if inside_regex {
            current.push(character);
            if character == '/' {
                inside_regex = false;
            }
            continue;
        }

        if character.is_whitespace() {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            continue;
        }

        if character == '"' || character == '\'' {
            active_quote = Some(character);
            continue;
        }

        if character == '/' && current.is_empty() {
            inside_regex = true;
            current.push(character);
            continue;
        }

        current.push(character);
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn unwrap_regex_literal(token: &str) -> Option<&str> {
    if token.len() >= 3 && token.starts_with('/') && token.ends_with('/') {
        Some(&token[1..token.len() - 1])
    } else {
        None
    }
}

fn compile_bounded_regex(pattern: &str) -> Option<Regex> {
    if pattern.is_empty() || pattern.len() > MAXIMUM_REGEX_PATTERN_BYTES {
        return None;
    }
    RegexBuilder::new(pattern)
        .case_insensitive(true)
        .size_limit(MAXIMUM_COMPILED_REGEX_BYTES)
        .build()
        .ok()
}

fn strip_matching_quotes(value: &str) -> &str {
    if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        &value[1..value.len() - 1]
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn parses_qualifiers_negations_quotes_and_regex_table_driven() {
        let query = parse_structured_search_query(
            "channel:email -status:archived run:e2e-42 to:alice@example.com otp:true \"reset password\" -spam /order-[0-9]+/",
        );

        assert_eq!(query.include_channels, vec![ChannelType::Email]);
        assert_eq!(query.exclude_statuses, vec![NotificationStatus::Archived]);
        assert_eq!(query.include_run_identifiers, vec!["e2e-42"]);
        assert_eq!(query.include_recipients, vec!["alice@example.com"]);
        assert_eq!(query.has_otp, Some(true));
        assert_eq!(query.include_terms, vec!["reset password"]);
        assert_eq!(query.exclude_terms, vec!["spam"]);
        assert_eq!(query.regex_patterns.len(), 1);
        assert!(query.requires_summary_post_filter());
    }

    #[test]
    fn treats_invalid_regex_and_unknown_qualifiers_as_safe_literals() {
        let query = parse_structured_search_query("/[unclosed/ unknown:foo otp:false");
        assert!(query.regex_patterns.is_empty());
        assert_eq!(query.include_terms, vec!["/[unclosed/", "unknown:foo"]);
        assert_eq!(query.has_otp, Some(false));
    }

    proptest! {
        #[test]
        fn structured_search_parser_never_panics_on_arbitrary_input(input in ".*") {
            let parsed = parse_structured_search_query(&input);
            let _ = parsed.requires_summary_post_filter();
        }
    }
}
