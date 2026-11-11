use mail_parser::{AddressList, MessageParser};
use serde_json::{Value, json};

use crate::domain::notification::CaptureAttachmentInput;
use crate::infrastructure::security::html_sanitizer::sanitize_email_html;

const PREVIEW_MAX_CHARACTERS: usize = 160;

/// Normalized parsed email representation extracted from an RFC 5322 MIME payload.
#[derive(Debug, Clone)]
pub struct ParsedSmtpEmail {
    pub sender: String,
    pub recipient: String,
    pub subject: String,
    pub preview_text: String,
    pub payload_json: Value,
    pub attachments: Vec<CaptureAttachmentInput>,
}

/// Parses raw RFC 5322 email bytes into canonical JSON metadata, sanitized HTML, and attachments.
pub fn parse_smtp_message(
    raw_bytes: &[u8],
    envelope_sender: &str,
    envelope_recipients: &[String],
) -> ParsedSmtpEmail {
    let parser = MessageParser::new();
    let Some(message) = parser.parse(raw_bytes) else {
        let fallback_text = String::from_utf8_lossy(raw_bytes).to_string();
        let primary_recipient = envelope_recipients.first().cloned().unwrap_or_default();
        return ParsedSmtpEmail {
            sender: envelope_sender.to_string(),
            recipient: primary_recipient,
            subject: "(Unparsable MIME Message)".to_string(),
            preview_text: truncate_preview(&fallback_text, PREVIEW_MAX_CHARACTERS),
            payload_json: json!({
                "from": [envelope_sender],
                "to": envelope_recipients,
                "cc": [],
                "bcc": [],
                "subject": "",
                "textBody": fallback_text,
                "rawHtml": "",
                "sanitizedHtml": "",
                "headers": [],
            }),
            attachments: Vec::new(),
        };
    };

    let from_addresses = format_address_list(message.from());
    let to_addresses = format_address_list(message.to());
    let cc_addresses = format_address_list(message.cc());
    let bcc_addresses = format_address_list(message.bcc());

    let effective_from = if !from_addresses.is_empty() {
        from_addresses.clone()
    } else if !envelope_sender.is_empty() {
        vec![envelope_sender.to_string()]
    } else {
        Vec::new()
    };

    let effective_to = if !to_addresses.is_empty() {
        to_addresses.clone()
    } else {
        envelope_recipients.to_vec()
    };

    let subject = message.subject().unwrap_or_default().trim().to_string();
    let text_body = message
        .body_text(0)
        .map(|text| text.into_owned())
        .unwrap_or_default();
    let raw_html = message
        .body_html(0)
        .map(|html| html.into_owned())
        .unwrap_or_default();
    let sanitized_html = if raw_html.trim().is_empty() {
        String::new()
    } else {
        sanitize_email_html(&raw_html)
    };

    let headers: Vec<Value> = message
        .headers()
        .iter()
        .map(|header| {
            json!({
                "name": header.name().as_str(),
                "value": String::from_utf8_lossy(header.raw_value()).trim(),
            })
        })
        .collect();

    let mut attachments = Vec::new();
    let mut attachment_descriptors = Vec::new();

    for (index, part) in message.attachments().enumerate() {
        let filename = part
            .attachment_name()
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_string)
            .unwrap_or_else(|| format!("attachment-{}", index + 1));

        let content_type = part
            .content_type()
            .map(|ct| match ct.subtype() {
                Some(subtype) => format!("{}/{}", ct.ctype(), subtype),
                None => ct.ctype().to_string(),
            })
            .unwrap_or_else(|| "application/octet-stream".to_string());

        let content_id = part.content_id().map(|cid| cid.trim().to_string());
        let is_inline = content_id.is_some();
        let content_blob = part.decoded().into_owned();
        let size_bytes = content_blob.len();

        attachment_descriptors.push(json!({
            "filename": filename,
            "contentType": content_type,
            "contentId": content_id,
            "isInline": is_inline,
            "sizeBytes": size_bytes,
        }));

        attachments.push(CaptureAttachmentInput {
            filename,
            content_type,
            content_id,
            is_inline,
            content_blob,
        });
    }

    let preview_source = if !text_body.trim().is_empty() {
        text_body.clone()
    } else if !subject.is_empty() {
        subject.clone()
    } else {
        sanitized_html.clone()
    };

    let sender = effective_from
        .first()
        .cloned()
        .unwrap_or_else(|| envelope_sender.to_string());
    let recipient = effective_to.join(", ");

    let payload_json = json!({
        "envelopeFrom": envelope_sender,
        "envelopeTo": envelope_recipients,
        "from": effective_from,
        "to": effective_to,
        "cc": cc_addresses,
        "bcc": bcc_addresses,
        "subject": subject,
        "textBody": text_body,
        "rawHtml": raw_html,
        "sanitizedHtml": sanitized_html,
        "headers": headers,
        "attachments": attachment_descriptors,
    });

    ParsedSmtpEmail {
        sender,
        recipient,
        subject: if subject.is_empty() {
            "(No Subject)".to_string()
        } else {
            subject
        },
        preview_text: truncate_preview(&preview_source, PREVIEW_MAX_CHARACTERS),
        payload_json,
        attachments,
    }
}

fn format_address_list(list: Option<AddressList<'_>>) -> Vec<String> {
    let Some(address_list) = list else {
        return Vec::new();
    };

    address_list
        .mailboxes()
        .filter_map(|mailbox| match (mailbox.name(), mailbox.address()) {
            (Some(name), Some(email)) if !name.trim().is_empty() => {
                Some(format!("{} <{}>", name.trim(), email.trim()))
            }
            (_, Some(email)) => Some(email.trim().to_string()),
            _ => None,
        })
        .collect()
}

fn truncate_preview(text: &str, maximum_characters: usize) -> String {
    let compact: String = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut result = String::new();
    for (index, character) in compact.chars().enumerate() {
        if index >= maximum_characters {
            result.push('…');
            break;
        }
        result.push(character);
    }
    result
}
