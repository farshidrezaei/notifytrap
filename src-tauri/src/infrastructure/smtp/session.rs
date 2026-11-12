use std::sync::Arc;
use std::time::Duration;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use crate::application::application_state::ApplicationServices;
use crate::domain::channel::{ChannelSummary, ChannelType, should_trigger_chaos_failure};
use crate::domain::notification::CaptureNotificationInput;
use crate::events::{EventPublisher, NotificationCapturedEvent};
use crate::infrastructure::smtp::mime_parser::parse_smtp_message;

const MAXIMUM_COMMAND_LINE_BYTES: usize = 8192;
const MAXIMUM_DATA_LINE_FLOOR_BYTES: usize = 65_536;
const SMTP_SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Default)]
struct SmtpTransaction {
    envelope_sender: Option<String>,
    envelope_recipients: Vec<String>,
}

/// Runs a single RFC 5321 ESMTP connection with AUTH PLAIN / AUTH LOGIN support.
pub async fn handle_smtp_connection(
    stream: TcpStream,
    services: ApplicationServices,
    event_publisher: Arc<dyn EventPublisher>,
    maximum_message_bytes: usize,
) {
    let (read_half, mut write_half) = stream.into_split();
    let mut reader = BufReader::new(read_half);

    if write_line(
        &mut write_half,
        "220 notifytrap.local ESMTP NotifyTrap ready",
    )
    .await
    .is_err()
    {
        return;
    }

    let mut authenticated_channel: Option<ChannelSummary> = None;
    let mut transaction = SmtpTransaction::default();

    loop {
        let Some(line) = read_crlf_line(&mut reader, MAXIMUM_COMMAND_LINE_BYTES).await else {
            return;
        };

        let trimmed = line.trim_end_matches(['\r', '\n']);
        if trimmed.is_empty() {
            if write_line(&mut write_half, "500 5.5.2 Unrecognized empty command")
                .await
                .is_err()
            {
                return;
            }
            continue;
        }

        let (verb, argument) = split_command(trimmed);
        let uppercase_verb = verb.to_ascii_uppercase();

        match uppercase_verb.as_str() {
            "EHLO" => {
                let response = format!(
                    "250-notifytrap.local\r\n250-SIZE {maximum_message_bytes}\r\n250 AUTH PLAIN LOGIN\r\n"
                );
                if write_half.write_all(response.as_bytes()).await.is_err()
                    || write_half.flush().await.is_err()
                {
                    return;
                }
            }
            "HELO" => {
                if write_line(&mut write_half, "250 notifytrap.local")
                    .await
                    .is_err()
                {
                    return;
                }
            }
            "AUTH" => {
                authenticated_channel =
                    match handle_auth_command(argument, &mut reader, &mut write_half, &services)
                        .await
                    {
                        Ok(channel) => {
                            if write_line(&mut write_half, "235 2.7.0 Authentication successful")
                                .await
                                .is_err()
                            {
                                return;
                            }
                            Some(channel)
                        }
                        Err(AuthOutcome::InvalidCredentials) => {
                            if write_line(
                                &mut write_half,
                                "535 5.7.8 Authentication credentials invalid",
                            )
                            .await
                            .is_err()
                            {
                                return;
                            }
                            None
                        }
                        Err(AuthOutcome::UnsupportedMechanism) => {
                            if write_line(
                                &mut write_half,
                                "504 5.5.4 Unrecognized authentication type",
                            )
                            .await
                            .is_err()
                            {
                                return;
                            }
                            authenticated_channel
                        }
                        Err(AuthOutcome::ConnectionClosed) => return,
                    };
            }
            "MAIL" => {
                if authenticated_channel.is_none() {
                    if write_line(&mut write_half, "530 5.7.0 Authentication required")
                        .await
                        .is_err()
                    {
                        return;
                    }
                    continue;
                }

                if let Some(sender_address) = extract_envelope_address(argument, "FROM:") {
                    transaction = SmtpTransaction {
                        envelope_sender: Some(sender_address),
                        envelope_recipients: Vec::new(),
                    };
                    if write_line(&mut write_half, "250 2.1.0 Ok").await.is_err() {
                        return;
                    }
                } else if write_line(&mut write_half, "501 5.5.4 Syntax error in MAIL parameters")
                    .await
                    .is_err()
                {
                    return;
                }
            }
            "RCPT" => {
                if authenticated_channel.is_none() {
                    if write_line(&mut write_half, "530 5.7.0 Authentication required")
                        .await
                        .is_err()
                    {
                        return;
                    }
                    continue;
                }
                if transaction.envelope_sender.is_none() {
                    if write_line(&mut write_half, "503 5.5.1 Need MAIL command first")
                        .await
                        .is_err()
                    {
                        return;
                    }
                    continue;
                }

                if let Some(recipient_address) = extract_envelope_address(argument, "TO:") {
                    transaction.envelope_recipients.push(recipient_address);
                    if write_line(&mut write_half, "250 2.1.5 Ok").await.is_err() {
                        return;
                    }
                } else if write_line(&mut write_half, "501 5.5.4 Syntax error in RCPT parameters")
                    .await
                    .is_err()
                {
                    return;
                }
            }
            "DATA" => {
                let Some(channel) = authenticated_channel.as_ref() else {
                    if write_line(&mut write_half, "530 5.7.0 Authentication required")
                        .await
                        .is_err()
                    {
                        return;
                    }
                    continue;
                };

                if transaction.envelope_sender.is_none()
                    || transaction.envelope_recipients.is_empty()
                {
                    if write_line(
                        &mut write_half,
                        "503 5.5.1 Need MAIL and RCPT commands first",
                    )
                    .await
                    .is_err()
                    {
                        return;
                    }
                    continue;
                }

                if write_line(&mut write_half, "354 End data with <CR><LF>.<CR><LF>")
                    .await
                    .is_err()
                {
                    return;
                }

                match read_smtp_data_payload(&mut reader, maximum_message_bytes).await {
                    DataReadResult::Complete(raw_bytes) => {
                        let envelope_sender =
                            transaction.envelope_sender.take().unwrap_or_default();
                        let envelope_recipients =
                            std::mem::take(&mut transaction.envelope_recipients);

                        if channel.chaos_rule.is_enabled {
                            if channel.chaos_rule.latency_milliseconds > 0 {
                                tokio::time::sleep(Duration::from_millis(
                                    channel.chaos_rule.latency_milliseconds,
                                ))
                                .await;
                            }
                            if should_trigger_chaos_failure(channel.chaos_rule.failure_rate_percent)
                            {
                                let smtp_reply = format!(
                                    "{} {}",
                                    channel.chaos_rule.smtp_error_code,
                                    channel.chaos_rule.error_message
                                );
                                if write_line(&mut write_half, &smtp_reply).await.is_err() {
                                    return;
                                }
                                continue;
                            }
                        }

                        let parsed =
                            parse_smtp_message(&raw_bytes, &envelope_sender, &envelope_recipients);
                        let raw_payload = String::from_utf8_lossy(&raw_bytes).to_string();

                        match services.notifications.capture_notification(
                            CaptureNotificationInput {
                                workspace_id: channel.workspace_id.clone(),
                                project_id: channel.project_id.clone(),
                                environment_id: channel.environment_id.clone(),
                                channel_id: channel.id.clone(),
                                channel_type: ChannelType::Email,
                                recipient: parsed.recipient,
                                sender: parsed.sender,
                                subject: parsed.subject,
                                preview_text: parsed.preview_text,
                                payload_json: parsed.payload_json,
                                raw_payload,
                                test_run_identifier: parsed.test_run_identifier,
                                received_at: None,
                                attachments: parsed.attachments,
                            },
                        ) {
                            Ok(detail) => {
                                event_publisher.emit_notification_captured(
                                    &NotificationCapturedEvent {
                                        notification: detail.notification.to_summary(),
                                        attachment_count: detail.attachments.len(),
                                    },
                                );
                                if write_line(
                                    &mut write_half,
                                    &format!("250 2.0.0 Ok: queued as {}", detail.notification.id),
                                )
                                .await
                                .is_err()
                                {
                                    return;
                                }
                            }
                            Err(_) => {
                                if write_line(&mut write_half, "451 4.3.0 Local storage failure")
                                    .await
                                    .is_err()
                                {
                                    return;
                                }
                            }
                        }
                    }
                    DataReadResult::ExceededSizeLimit => {
                        transaction = SmtpTransaction::default();
                        if write_line(
                            &mut write_half,
                            "552 5.3.4 Message size exceeds fixed maximum message size",
                        )
                        .await
                        .is_err()
                        {
                            return;
                        }
                    }
                    DataReadResult::ConnectionClosed => return,
                }
            }
            "RSET" => {
                transaction = SmtpTransaction::default();
                if write_line(&mut write_half, "250 2.0.0 Ok").await.is_err() {
                    return;
                }
            }
            "NOOP" => {
                if write_line(&mut write_half, "250 2.0.0 Ok").await.is_err() {
                    return;
                }
            }
            "QUIT" => {
                let _ = write_line(&mut write_half, "221 2.0.0 Bye").await;
                return;
            }
            _ => {
                if write_line(&mut write_half, "502 5.5.1 Command not implemented")
                    .await
                    .is_err()
                {
                    return;
                }
            }
        }
    }
}

enum AuthOutcome {
    InvalidCredentials,
    UnsupportedMechanism,
    ConnectionClosed,
}

enum DataReadResult {
    Complete(Vec<u8>),
    ExceededSizeLimit,
    ConnectionClosed,
}

async fn handle_auth_command(
    argument: &str,
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    write_half: &mut tokio::net::tcp::OwnedWriteHalf,
    services: &ApplicationServices,
) -> Result<ChannelSummary, AuthOutcome> {
    let (mechanism, initial_response) = split_command(argument);
    match mechanism.to_ascii_uppercase().as_str() {
        "PLAIN" => {
            let encoded_payload = if !initial_response.is_empty() {
                initial_response.to_string()
            } else {
                write_line(write_half, "334 ")
                    .await
                    .map_err(|_| AuthOutcome::ConnectionClosed)?;
                let line = read_crlf_line(reader, MAXIMUM_COMMAND_LINE_BYTES)
                    .await
                    .ok_or(AuthOutcome::ConnectionClosed)?;
                line.trim().to_string()
            };

            let (username, password) =
                decode_auth_plain(&encoded_payload).ok_or(AuthOutcome::InvalidCredentials)?;
            authenticate_email_channel(services, &username, &password)
        }
        "LOGIN" => {
            let encoded_username = if !initial_response.is_empty() {
                initial_response.to_string()
            } else {
                write_line(write_half, "334 VXNlcm5hbWU6")
                    .await
                    .map_err(|_| AuthOutcome::ConnectionClosed)?;
                let line = read_crlf_line(reader, MAXIMUM_COMMAND_LINE_BYTES)
                    .await
                    .ok_or(AuthOutcome::ConnectionClosed)?;
                line.trim().to_string()
            };

            let username =
                decode_base64_utf8(&encoded_username).ok_or(AuthOutcome::InvalidCredentials)?;

            write_line(write_half, "334 UGFzc3dvcmQ6")
                .await
                .map_err(|_| AuthOutcome::ConnectionClosed)?;
            let password_line = read_crlf_line(reader, MAXIMUM_COMMAND_LINE_BYTES)
                .await
                .ok_or(AuthOutcome::ConnectionClosed)?;
            let password =
                decode_base64_utf8(password_line.trim()).ok_or(AuthOutcome::InvalidCredentials)?;

            authenticate_email_channel(services, &username, &password)
        }
        _ => Err(AuthOutcome::UnsupportedMechanism),
    }
}

fn authenticate_email_channel(
    services: &ApplicationServices,
    username: &str,
    password: &str,
) -> Result<ChannelSummary, AuthOutcome> {
    let channel = services
        .channels
        .authenticate_identifier_and_secret(username, password)
        .map_err(|_| AuthOutcome::InvalidCredentials)?;

    if channel.channel_type != ChannelType::Email {
        return Err(AuthOutcome::InvalidCredentials);
    }

    Ok(channel)
}

fn decode_auth_plain(encoded: &str) -> Option<(String, String)> {
    let raw_bytes = STANDARD.decode(encoded.trim()).ok()?;
    let parts: Vec<&[u8]> = raw_bytes.split(|byte| *byte == 0).collect();
    if parts.len() != 3 {
        return None;
    }
    let username = String::from_utf8(parts[1].to_vec()).ok()?;
    let password = String::from_utf8(parts[2].to_vec()).ok()?;
    if username.is_empty() || password.is_empty() {
        return None;
    }
    Some((username, password))
}

fn decode_base64_utf8(encoded: &str) -> Option<String> {
    let bytes = STANDARD.decode(encoded.trim()).ok()?;
    String::from_utf8(bytes).ok()
}

async fn read_smtp_data_payload(
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    maximum_message_bytes: usize,
) -> DataReadResult {
    let mut accumulated = Vec::new();
    let mut is_oversized = false;
    let maximum_line_bytes = maximum_message_bytes
        .max(MAXIMUM_DATA_LINE_FLOOR_BYTES)
        .saturating_add(16);

    loop {
        let mut line_buffer = Vec::new();
        let mut limited_reader = reader.take((maximum_line_bytes + 1) as u64);
        let read_outcome = tokio::time::timeout(
            SMTP_SESSION_IDLE_TIMEOUT,
            limited_reader.read_until(b'\n', &mut line_buffer),
        )
        .await;

        match read_outcome {
            Ok(Ok(0)) | Ok(Err(_)) | Err(_) => return DataReadResult::ConnectionClosed,
            Ok(Ok(_)) => {
                if line_buffer.len() > maximum_line_bytes {
                    return DataReadResult::ConnectionClosed;
                }
                let without_crlf = strip_line_ending(&line_buffer);
                if without_crlf == b"." {
                    return if is_oversized {
                        DataReadResult::ExceededSizeLimit
                    } else {
                        DataReadResult::Complete(accumulated)
                    };
                }

                if is_oversized {
                    continue;
                }

                // RFC 5321 Section 4.5.2 dot-unstuffing.
                let unstuffed = without_crlf.strip_prefix(b".").unwrap_or(without_crlf);
                let added_length = unstuffed.len() + 2;
                if accumulated.len().saturating_add(added_length) > maximum_message_bytes {
                    is_oversized = true;
                    accumulated.clear();
                    continue;
                }

                accumulated.extend_from_slice(unstuffed);
                accumulated.extend_from_slice(b"\r\n");
            }
        }
    }
}

fn strip_line_ending(bytes: &[u8]) -> &[u8] {
    if let Some(without_lf) = bytes.strip_suffix(b"\n") {
        without_lf.strip_suffix(b"\r").unwrap_or(without_lf)
    } else {
        bytes
    }
}

async fn read_crlf_line(
    reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>,
    maximum_bytes: usize,
) -> Option<String> {
    let mut buffer = Vec::new();
    let mut limited_reader = reader.take((maximum_bytes + 1) as u64);
    let bytes_read = tokio::time::timeout(
        SMTP_SESSION_IDLE_TIMEOUT,
        limited_reader.read_until(b'\n', &mut buffer),
    )
    .await
    .ok()?
    .ok()?;
    if bytes_read == 0 || buffer.len() > maximum_bytes {
        return None;
    }
    Some(String::from_utf8_lossy(&buffer).into_owned())
}

async fn write_line(
    write_half: &mut tokio::net::tcp::OwnedWriteHalf,
    line: &str,
) -> std::io::Result<()> {
    write_half.write_all(line.as_bytes()).await?;
    write_half.write_all(b"\r\n").await?;
    write_half.flush().await
}

fn split_command(input: &str) -> (&str, &str) {
    let trimmed = input.trim();
    match trimmed.split_once(char::is_whitespace) {
        Some((command, rest)) => (command, rest.trim_start()),
        None => (trimmed, ""),
    }
}

fn extract_envelope_address(argument: &str, expected_prefix: &str) -> Option<String> {
    let trimmed = argument.trim();
    let prefix_len = expected_prefix.len();
    if trimmed.len() < prefix_len || !trimmed[..prefix_len].eq_ignore_ascii_case(expected_prefix) {
        return None;
    }

    let remainder = trimmed[prefix_len..].trim();
    let angle_content = if let Some(start) = remainder.find('<') {
        let end = remainder.find('>')?;
        if end < start {
            return None;
        }
        &remainder[start + 1..end]
    } else {
        remainder.split_whitespace().next()?
    };

    Some(angle_content.trim().to_string())
}
