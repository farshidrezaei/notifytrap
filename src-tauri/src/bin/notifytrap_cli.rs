use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;

use serde_json::{Value, json};

use notifytrap_lib::application::application_state::ApplicationServices;
use notifytrap_lib::configuration::server_configuration::ServerConfiguration;
use notifytrap_lib::events::BroadcastEventPublisher;
use notifytrap_lib::infrastructure::database::connection_manager::DatabaseConnectionManager;
use notifytrap_lib::infrastructure::http::loopback_replay::{
    execute_loopback_http_replay, validate_loopback_target_url,
};
use notifytrap_lib::infrastructure::http::routes::HttpCaptureState;
use notifytrap_lib::infrastructure::http::server::RunningHttpServer;
use notifytrap_lib::infrastructure::http::web_server::RunningWebCompanionServer;
use notifytrap_lib::infrastructure::security::secret_key_provider::EphemeralSecretKeyProvider;
use notifytrap_lib::infrastructure::smtp::server::RunningSmtpServer;

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
    // Headless serve options
    bind_address: String,
    http_port: u16,
    smtp_port: u16,
    web_port: Option<u16>,
    database_path: Option<String>,
    web_assets_path: Option<String>,
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

    if subcommand == "serve" {
        return match run_headless_serve(&options).await {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("Error: {message}");
                ExitCode::FAILURE
            }
        };
    }

    if subcommand == "update" {
        return match run_self_update().await {
            Ok(()) => ExitCode::SUCCESS,
            Err(message) => {
                eprintln!("Error: {message}");
                ExitCode::FAILURE
            }
        };
    }

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

    let mut bind_address =
        env::var("NOTIFYTRAP_BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1".to_string());
    let mut http_port: u16 = env::var("NOTIFYTRAP_HTTP_PORT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(4080);
    let mut smtp_port: u16 = env::var("NOTIFYTRAP_SMTP_PORT")
        .ok()
        .and_then(|val| val.parse().ok())
        .unwrap_or(1025);
    let mut web_port: Option<u16> = env::var("NOTIFYTRAP_WEB_PORT")
        .ok()
        .and_then(|val| val.parse().ok());
    let mut database_path = env::var("NOTIFYTRAP_DATABASE_PATH").ok();
    let mut web_assets_path = env::var("NOTIFYTRAP_WEB_ASSETS_PATH").ok();

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
            "--bind-address" => {
                index += 1;
                bind_address = arguments
                    .get(index)
                    .cloned()
                    .ok_or_else(|| "--bind-address requires an IPv4 address".to_string())?;
            }
            "--http-port" => {
                index += 1;
                let raw = arguments
                    .get(index)
                    .ok_or_else(|| "--http-port requires a port number".to_string())?;
                http_port = raw
                    .parse::<u16>()
                    .map_err(|_| format!("Invalid --http-port value: {raw}"))?;
            }
            "--smtp-port" => {
                index += 1;
                let raw = arguments
                    .get(index)
                    .ok_or_else(|| "--smtp-port requires a port number".to_string())?;
                smtp_port = raw
                    .parse::<u16>()
                    .map_err(|_| format!("Invalid --smtp-port value: {raw}"))?;
            }
            "--web-port" => {
                index += 1;
                let raw = arguments
                    .get(index)
                    .ok_or_else(|| "--web-port requires a port number".to_string())?;
                web_port = Some(
                    raw.parse::<u16>()
                        .map_err(|_| format!("Invalid --web-port value: {raw}"))?,
                );
            }
            "--database-path" => {
                index += 1;
                database_path = Some(
                    arguments
                        .get(index)
                        .cloned()
                        .ok_or_else(|| "--database-path requires a path".to_string())?,
                );
            }
            "--web-assets-path" => {
                index += 1;
                web_assets_path = Some(
                    arguments
                        .get(index)
                        .cloned()
                        .ok_or_else(|| "--web-assets-path requires a directory path".to_string())?,
                );
            }
            other if other.starts_with('-') => {
                return Err(format!("Unknown option: {other}"));
            }
            value => positional.push(value.to_string()),
        }
        index += 1;
    }

    Ok(CliOptions {
        base_url,
        channel_key,
        run_id,
        search,
        timeout_ms,
        format,
        positional,
        bind_address,
        http_port,
        smtp_port,
        web_port,
        database_path,
        web_assets_path,
    })
}

async fn run_headless_serve(options: &CliOptions) -> Result<(), String> {
    println!(
        "=== NotifyTrap Headless Server v{} ===",
        env!("CARGO_PKG_VERSION")
    );

    let database_manager = if let Some(path_str) = &options.database_path {
        let path = Path::new(path_str);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        println!("Storage: File SQLite [{path_str}]");
        DatabaseConnectionManager::open_file(path).map_err(|err| err.to_string())?
    } else {
        println!("Storage: In-Memory SQLite (Isolated session)");
        DatabaseConnectionManager::open_in_memory().map_err(|err| err.to_string())?
    };

    let secret_key_provider = Arc::new(EphemeralSecretKeyProvider::new());
    let services = ApplicationServices::new(database_manager.clone(), secret_key_provider);
    let event_publisher = Arc::new(BroadcastEventPublisher::default());

    let server_config = ServerConfiguration {
        bind_address: options.bind_address.clone(),
        http_port: options.http_port,
        smtp_port: options.smtp_port,
        maximum_http_payload_bytes: 10 * 1024 * 1024,
        maximum_smtp_message_bytes: 25 * 1024 * 1024,
        shutdown_grace_period_seconds: 5,
    };

    let http_server = RunningHttpServer::start(
        &server_config,
        services.clone(),
        Arc::clone(&event_publisher) as Arc<dyn notifytrap_lib::events::EventPublisher>,
    )
    .await
    .map_err(|err| format!("Failed to start HTTP capture server: {err}"))?;
    println!("HTTP Capture API: http://{}", http_server.bound_address());

    let smtp_server = RunningSmtpServer::start(
        &server_config,
        services.clone(),
        Arc::clone(&event_publisher) as Arc<dyn notifytrap_lib::events::EventPublisher>,
    )
    .await
    .map_err(|err| format!("Failed to start SMTP capture server: {err}"))?;
    println!("SMTP Capture Server: smtp://{}", smtp_server.bound_address());

    let _web_companion = if let Some(web_port) = options.web_port {
        let assets_dir = options.web_assets_path.as_ref().map(PathBuf::from).or_else(|| {
            let candidate = PathBuf::from(".output/public");
            if candidate.is_dir() {
                Some(candidate)
            } else {
                None
            }
        });

        let capture_state = HttpCaptureState {
            services: services.clone(),
            event_publisher: Arc::clone(&event_publisher) as Arc<dyn notifytrap_lib::events::EventPublisher>,
            bound_endpoint: format!("{}:{}", options.bind_address, web_port),
        };

        let companion = RunningWebCompanionServer::start(
            &options.bind_address,
            web_port,
            assets_dir.clone(),
            capture_state,
            10 * 1024 * 1024,
        )
        .await
        .map_err(|err| format!("Failed to start Web Companion server: {err}"))?;

        if let Some(dir) = assets_dir {
            println!(
                "Web Companion UI: http://{} (Serving from: {})",
                companion.bound_address(),
                dir.display()
            );
        } else {
            println!(
                "Web Companion API: http://{} (Headless mode, no static UI folder found)",
                companion.bound_address()
            );
        }
        Some(companion)
    } else {
        None
    };

    println!("NotifyTrap is running in headless mode. Press Ctrl+C to terminate.");
    tokio::signal::ctrl_c()
        .await
        .map_err(|err| format!("Failed to listen for termination signal: {err}"))?;

    println!("\nShutting down headless capture servers...");
    http_server.shutdown().await;
    smtp_server.shutdown().await;
    if let Some(companion) = _web_companion {
        companion.stop().await;
    }
    println!("Servers terminated cleanly.");
    Ok(())
}

async fn run_tail(options: &CliOptions) -> Result<(), String> {
    let mut url = format!("{}/api/v1/notifications?limit=10", options.base_url);
    if let Some(run_id) = &options.run_id {
        url.push_str(&format!("&run_id={}", url_encode(run_id)));
    }
    if let Some(search) = &options.search {
        url.push_str(&format!("&search={}", url_encode(search)));
    }

    let body = send_loopback_request("GET", &url, &options.channel_key, None).await?;
    let parsed: Value =
        serde_json::from_str(&body).map_err(|error| format!("Invalid JSON response: {error}"))?;

    if let Some(items) = parsed.get("items").and_then(|val| val.as_array()) {
        if items.is_empty() {
            println!("No notifications captured for this channel scope.");
            return Ok(());
        }
        for item in items {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("-");
            let channel = item
                .get("channelType")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let sender = item.get("sender").and_then(|v| v.as_str()).unwrap_or("-");
            let subject = item.get("subject").and_then(|v| v.as_str()).unwrap_or("-");
            let preview = item
                .get("previewText")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let received = item
                .get("receivedAt")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            println!("[{received}] {channel:<7} | {sender} -> {subject}");
            if !preview.trim().is_empty() {
                println!("  preview: {preview}");
            }
            println!("  id: {id}");
        }
    } else {
        println!("{body}");
    }
    Ok(())
}

async fn run_wait_otp(options: &CliOptions) -> Result<(), String> {
    let mut url = format!(
        "{}/api/v1/notifications/wait?timeout_ms={}",
        options.base_url, options.timeout_ms
    );
    if let Some(run_id) = &options.run_id {
        url.push_str(&format!("&run_id={}", url_encode(run_id)));
    }
    if let Some(search) = &options.search {
        url.push_str(&format!("&search={}", url_encode(search)));
    }

    let body = send_loopback_request("GET", &url, &options.channel_key, None).await?;
    let parsed: Value =
        serde_json::from_str(&body).map_err(|error| format!("Invalid JSON response: {error}"))?;

    let artifacts = parsed
        .get("notification")
        .and_then(|notif| notif.get("verificationArtifacts"))
        .and_then(|val| val.as_array());

    if let Some(list) = artifacts {
        for artifact in list {
            let kind = artifact.get("kind").and_then(|v| v.as_str()).unwrap_or("");
            if kind == "otp" {
                if let Some(code) = artifact.get("value").and_then(|v| v.as_str()) {
                    println!("{code}");
                    return Ok(());
                }
            }
        }
        for artifact in list {
            if let Some(link) = artifact.get("value").and_then(|v| v.as_str()) {
                println!("{link}");
                return Ok(());
            }
        }
    }

    if let Some(preview) = parsed
        .get("notification")
        .and_then(|n| n.get("previewText"))
        .and_then(|v| v.as_str())
    {
        println!("{preview}");
        return Ok(());
    }

    Err("No OTP or verification artifact extracted from captured notification".to_string())
}

async fn run_export(options: &CliOptions) -> Result<(), String> {
    let notification_id = options.positional.first().ok_or_else(|| {
        "export requires <notification_id>. Run 'notifytrap-cli tail' to list recent IDs."
            .to_string()
    })?;

    let url = format!(
        "{}/api/v1/notifications/{notification_id}",
        options.base_url
    );
    let body = send_loopback_request("GET", &url, &options.channel_key, None).await?;

    if options.format.eq_ignore_ascii_case("summary") {
        let parsed: Value = serde_json::from_str(&body)
            .map_err(|error| format!("Invalid JSON response: {error}"))?;
        let subject = parsed
            .get("notification")
            .and_then(|n| n.get("subject"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let recipient = parsed
            .get("notification")
            .and_then(|n| n.get("recipient"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        let channel = parsed
            .get("notification")
            .and_then(|n| n.get("channelType"))
            .and_then(|v| v.as_str())
            .unwrap_or("-");
        println!("Notification: {notification_id}");
        println!("Channel:      {channel}");
        println!("Recipient:    {recipient}");
        println!("Subject:      {subject}");
    } else {
        println!("{body}");
    }
    Ok(())
}

async fn run_mcp_stdio_bridge(options: &CliOptions) -> Result<(), String> {
    let url = format!("{}/api/v1/mcp", options.base_url);
    let target = validate_loopback_target_url(&url).map_err(|error| error.to_string())?;

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let raw_line = match line {
            Ok(line) => line,
            Err(error) => return Err(format!("STDIN read failure: {error}")),
        };
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let headers = vec![
            ("X-Channel-Key".to_string(), options.channel_key.clone()),
            ("Content-Type".to_string(), "application/json".to_string()),
        ];

        let result = match execute_loopback_http_replay(&target, "POST", &headers, trimmed).await {
            Ok(res) => res,
            Err(err) => {
                let error_rpc = json!({
                    "jsonrpc": "2.0",
                    "id": Value::Null,
                    "error": {
                        "code": -32000,
                        "message": format!("Local NotifyTrap bridge transport error: {err}")
                    }
                });
                writeln!(stdout, "{}", serde_json::to_string(&error_rpc).unwrap())
                    .map_err(|e| e.to_string())?;
                stdout.flush().map_err(|e| e.to_string())?;
                continue;
            }
        };

        writeln!(stdout, "{}", result.response_body.trim()).map_err(|e| e.to_string())?;
        stdout.flush().map_err(|e| e.to_string())?;
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

async fn run_self_update() -> Result<(), String> {
    println!("[NotifyTrap CLI] Checking for updates...");
    let current_version = env!("CARGO_PKG_VERSION");
    let client = reqwest::Client::builder()
        .user_agent(format!("notifytrap-cli/{current_version}"))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|err| err.to_string())?;

    let release_url = "https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest";
    let response = client
        .get(release_url)
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|err| format!("Failed to reach update server: {err}"))?;

    if !response.status().is_success() {
        return Err(format!("Release server returned HTTP {}", response.status()));
    }

    let response_text = response
        .text()
        .await
        .map_err(|err| format!("Failed to read response body: {err}"))?;

    let release_json: Value = serde_json::from_str(&response_text)
        .map_err(|err| format!("Failed to parse release metadata: {err}"))?;

    let tag_name = release_json
        .get("tag_name")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let latest_version = tag_name.trim_start_matches('v');

    if latest_version == current_version {
        println!("[NotifyTrap CLI] You are already on the latest version (v{current_version}).");
        return Ok(());
    }

    println!("[NotifyTrap CLI] New version available: v{latest_version} (current: v{current_version})");

    let os = std::env::consts::OS;
    let expected_asset_suffix = match os {
        "linux" => "notifytrap-cli-linux-x86_64.tar.gz",
        "macos" => "notifytrap-cli-macos-universal.tar.gz",
        "windows" => "notifytrap-cli-windows-x86_64.zip",
        other => return Err(format!("Self-update is not supported on OS: {other}")),
    };

    let assets = release_json
        .get("assets")
        .and_then(|v| v.as_array())
        .ok_or_else(|| "No release assets found".to_string())?;

    let matched_asset = assets.iter().find(|a| {
        a.get("name")
            .and_then(|n| n.as_str())
            .map(|n| n == expected_asset_suffix)
            .unwrap_or(false)
    });

    let asset: &Value = match matched_asset {
        Some(a) => a,
        None => {
            println!("[NotifyTrap CLI] Run the official installer to update:");
            println!("  curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash");
            return Ok(());
        }
    };

    let download_url = asset
        .get("browser_download_url")
        .and_then(|u| u.as_str())
        .ok_or_else(|| "Missing download URL".to_string())?;

    println!("[NotifyTrap CLI] Downloading {expected_asset_suffix} from {download_url}...");
    let download_resp = client
        .get(download_url)
        .send()
        .await
        .map_err(|err| format!("Download failed: {err}"))?;

    let _bytes = download_resp
        .bytes()
        .await
        .map_err(|err| format!("Failed to read archive: {err}"))?;

    println!("[NotifyTrap CLI] Successfully downloaded update archive for v{latest_version}.");
    println!("[NotifyTrap CLI] To complete the update, rerun the installer or extract into your PATH.");
    println!("  curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash");
    Ok(())
}

fn print_usage() {
    println!(
        "NotifyTrap CLI Companion ({})
Usage:
  notifytrap-cli <subcommand> [options]

Subcommands:
  serve      Run headless capture servers (HTTP + SMTP) and optional Web Companion
  tail       List recent notifications for a channel
  wait-otp   Wait for a matching notification and print the extracted OTP code
  export     Export a notification detail JSON by ID
  mcp        Run stdio MCP JSON-RPC 2.0 bridge against local NotifyTrap
  update     Check for and perform self-update to latest GitHub release

Headless Serve Options:
  --bind-address <ip>     Bind IP address (default: 127.0.0.1 or NOTIFYTRAP_BIND_ADDRESS)
  --http-port <port>      HTTP capture API port (default: 4080 or NOTIFYTRAP_HTTP_PORT)
  --smtp-port <port>      SMTP capture server port (default: 1025 or NOTIFYTRAP_SMTP_PORT)
  --web-port <port>       Web Companion UI port (e.g. 4000 or NOTIFYTRAP_WEB_PORT)
  --database-path <path>  SQLite database file path (in-memory if omitted)
  --web-assets-path <dir> Static Nuxt SPA build directory (default: .output/public)

Client Options:
  --base-url <url>        Local NotifyTrap HTTP base URL (default: http://127.0.0.1:4080)
  --channel-key <key>     Channel key (<identifier>.<secret>) or NOTIFYTRAP_CHANNEL_KEY
  --run-id <id>           Filter by X-NotifyTrap-Run identifier
  --search <query>        Structured search query (e.g. 'channel:email otp:true')
  --timeout-ms <ms>       Wait timeout in milliseconds (50..9000, default: 8000)
  --format <json|summary> Output format for export (default: json)",
        env!("CARGO_PKG_VERSION")
    );
}
