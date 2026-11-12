use std::env;
use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use serde_json::{Value, json};

use notifytrap_lib::infrastructure::http::loopback_replay::{
    execute_loopback_http_replay, validate_loopback_target_url,
};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:4080";
const DEFAULT_WAIT_TIMEOUT_MILLISECONDS: u64 = 8_000;

struct CliOptions {
    base_url: String,
    channel_key: String,
    run_id: Option<String>,
    search: Option<String>,
    timeout_ms: u64,
    format: String,
    positional: Vec<String>,
}

#[tokio::main]
async fn main() -> ExitCode {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments.is_empty() || matches!(arguments[0].as_str(), "-h" | "--help" | "help") {
        print_usage();
        return ExitCode::SUCCESS;
    }
    if matches!(arguments[0].as_str(), "-V" | "--version" | "version") {
        println!("notifytrap-cli {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    let subcommand = arguments[0].as_str();
    let options = match parse_options(&arguments[1..]) {
        Ok(parsed) => parsed,
        Err(message) => {
            eprintln!("Error: {message}");
            return ExitCode::from(2);
        }
    };

    if options.channel_key.is_empty() {
        eprintln!("Error: --channel-key (or NOTIFYTRAP_CHANNEL_KEY) is required");
        return ExitCode::from(2);
    }

    let outcome = match subcommand {
        "tail" => run_tail(&options).await,
        "wait-otp" => run_wait_otp(&options).await,
        "export" => run_export(&options).await,
        "mcp" => run_mcp_stdio_bridge(&options).await,
        unknown => Err(format!(
            "Unknown subcommand '{unknown}'. Run 'notifytrap-cli --help' for usage."
        )),
    };

    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("Error: {message}");
            ExitCode::FAILURE
        }
    }
}

fn parse_options(arguments: &[String]) -> Result<CliOptions, String> {
    let mut base_url =
        env::var("NOTIFYTRAP_BASE_URL").unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());
    let mut channel_key = env::var("NOTIFYTRAP_CHANNEL_KEY").unwrap_or_default();
    let mut run_id = None;
    let mut search = None;
    let mut timeout_ms = DEFAULT_WAIT_TIMEOUT_MILLISECONDS;
    let mut format = "json".to_string();
    let mut positional = Vec::new();

    let mut index = 0;
    while index < arguments.len() {
        let flag = arguments[index].as_str();
        match flag {
            "--base-url" => {
                index += 1;
                base_url = arguments
                    .get(index)
                    .cloned()
                    .ok_or_else(|| "--base-url requires a value".to_string())?;
            }
            "--channel-key" => {
                index += 1;
                channel_key = arguments
                    .get(index)
                    .cloned()
                    .ok_or_else(|| "--channel-key requires a value".to_string())?;
            }
            "--run-id" => {
                index += 1;
                run_id = Some(
                    arguments
                        .get(index)
                        .cloned()
                        .ok_or_else(|| "--run-id requires a value".to_string())?,
                );
            }
            "--search" => {
                index += 1;
                search = Some(
                    arguments
                        .get(index)
                        .cloned()
                        .ok_or_else(|| "--search requires a value".to_string())?,
                );
            }
            "--timeout-ms" => {
                index += 1;
                let raw = arguments
                    .get(index)
                    .ok_or_else(|| "--timeout-ms requires a number".to_string())?;
                timeout_ms = raw
                    .parse::<u64>()
                    .map_err(|_| format!("Invalid --timeout-ms value: {raw}"))?
                    .clamp(50, 9_000);
            }
            "--format" => {
                index += 1;
                format = arguments
                    .get(index)
                    .cloned()
                    .ok_or_else(|| "--format requires json or summary".to_string())?;
            }
            other if other.starts_with('-') => {
                return Err(format!("Unknown option: {other}"));
            }
            value => positional.push(value.to_string()),
        }
        index += 1;
    }

    Ok(CliOptions {
        base_url: base_url.trim_end_matches('/').to_string(),
        channel_key,
        run_id,
        search,
        timeout_ms,
        format,
        positional,
    })
}

async fn run_tail(options: &CliOptions) -> Result<(), String> {
    let mut query_parts = vec!["limit=25".to_string()];
    if let Some(run_id) = &options.run_id {
        query_parts.push(format!("run_id={}", url_encode(run_id)));
    }
    if let Some(search) = &options.search {
        query_parts.push(format!("search={}", url_encode(search)));
    }
    let url = format!(
        "{}/api/v1/notifications?{}",
        options.base_url,
        query_parts.join("&")
    );
    let body = send_loopback_request("GET", &url, &options.channel_key, None).await?;
    let parsed: Value = serde_json::from_str(&body)
        .map_err(|error| format!("Invalid JSON response from NotifyTrap: {error}"))?;
    let items = parsed
        .get("items")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for item in items {
        let id = item.get("id").and_then(Value::as_str).unwrap_or("");
        let channel_type = item
            .get("channelType")
            .and_then(Value::as_str)
            .unwrap_or("");
        let subject = item.get("subject").and_then(Value::as_str).unwrap_or("");
        let recipient = item.get("recipient").and_then(Value::as_str).unwrap_or("");
        let received_at = item.get("receivedAt").and_then(Value::as_str).unwrap_or("");
        println!("[{received_at}] ({channel_type}) {id} -> {recipient} | {subject}");
    }
    Ok(())
}

async fn run_wait_otp(options: &CliOptions) -> Result<(), String> {
    let mut query_parts = vec![format!("timeout_ms={}", options.timeout_ms)];
    if let Some(run_id) = &options.run_id {
        query_parts.push(format!("run_id={}", url_encode(run_id)));
    }
    let search_value = options
        .search
        .clone()
        .unwrap_or_else(|| "otp:true".to_string());
    query_parts.push(format!("search={}", url_encode(&search_value)));

    let url = format!(
        "{}/api/v1/notifications/wait?{}",
        options.base_url,
        query_parts.join("&")
    );
    let body = send_loopback_request("GET", &url, &options.channel_key, None).await?;
    let detail: Value = serde_json::from_str(&body)
        .map_err(|error| format!("Invalid JSON response from NotifyTrap: {error}"))?;

    let artifacts = detail
        .pointer("/notification/verificationArtifacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();

    for artifact in artifacts {
        if artifact.get("kind").and_then(Value::as_str) == Some("otpCode")
            && let Some(value) = artifact.get("value").and_then(Value::as_str)
        {
            println!("{value}");
            return Ok(());
        }
    }

    Err("Notification arrived, but no OTP code was extracted".to_string())
}

async fn run_export(options: &CliOptions) -> Result<(), String> {
    let notification_id = options
        .positional
        .first()
        .ok_or_else(|| "export requires <notification_id>".to_string())?;
    let url = format!(
        "{}/api/v1/notifications/{}",
        options.base_url,
        url_encode(notification_id)
    );
    let body = send_loopback_request("GET", &url, &options.channel_key, None).await?;
    if options.format == "summary" {
        let detail: Value = serde_json::from_str(&body)
            .map_err(|error| format!("Invalid JSON response: {error}"))?;
        let summary = detail.get("notification").unwrap_or(&detail);
        let formatted = serde_json::to_string_pretty(summary)
            .map_err(|error| format!("Serialization failure: {error}"))?;
        println!("{formatted}");
    } else {
        println!("{body}");
    }
    Ok(())
}

async fn run_mcp_stdio_bridge(options: &CliOptions) -> Result<(), String> {
    let mcp_url = format!("{}/api/v1/mcp", options.base_url);
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line_result in stdin.lock().lines() {
        let line = line_result.map_err(|error| format!("Failed reading stdin: {error}"))?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response_body = match send_loopback_request(
            "POST",
            &mcp_url,
            &options.channel_key,
            Some(trimmed),
        )
        .await
        {
            Ok(body) => body,
            Err(error_message) => json!({
                "jsonrpc": "2.0",
                "id": Value::Null,
                "error": { "code": -32000, "message": error_message }
            })
            .to_string(),
        };
        writeln!(stdout, "{response_body}")
            .and_then(|()| stdout.flush())
            .map_err(|error| format!("Failed writing stdout: {error}"))?;
    }
    Ok(())
}

async fn send_loopback_request(
    method: &str,
    url: &str,
    channel_key: &str,
    body: Option<&str>,
) -> Result<String, String> {
    let target = validate_loopback_target_url(url).map_err(|error| error.to_string())?;
    let mut headers = vec![("X-Channel-Key".to_string(), channel_key.to_string())];
    if body.is_some() {
        headers.push(("Content-Type".to_string(), "application/json".to_string()));
    }

    let result = execute_loopback_http_replay(&target, method, &headers, body.unwrap_or(""))
        .await
        .map_err(|error| error.to_string())?;

    if result.status_code >= 400 {
        return Err(format!(
            "HTTP {}: {}",
            result.status_code, result.response_body
        ));
    }
    Ok(result.response_body)
}

fn url_encode(raw: &str) -> String {
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{byte:02X}")),
        }
    }
    encoded
}

fn print_usage() {
    println!(
        "NotifyTrap CLI Companion ({})
Usage:
  notifytrap-cli <subcommand> [options]

Subcommands:
  tail       List recent notifications for a channel
  wait-otp   Wait for a matching notification and print the extracted OTP code
  export     Export a notification detail JSON by ID
  mcp        Run stdio MCP JSON-RPC 2.0 bridge against local NotifyTrap

Options:
  --base-url <url>        Local NotifyTrap HTTP base URL (default: http://127.0.0.1:4080)
  --channel-key <key>     Channel key (<identifier>.<secret>) or NOTIFYTRAP_CHANNEL_KEY
  --run-id <id>           Filter by X-NotifyTrap-Run identifier
  --search <query>        Structured search query (e.g. 'channel:email otp:true')
  --timeout-ms <ms>       Wait timeout in milliseconds (50..9000, default: 8000)
  --format <json|summary> Output format for export (default: json)",
        env!("CARGO_PKG_VERSION")
    );
}
