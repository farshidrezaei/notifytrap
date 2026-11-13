use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::sync::Arc;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

use notifytrap_lib::application::application_state::ApplicationServices;
use notifytrap_lib::application::capture_servers::CaptureServerCoordinator;
use notifytrap_lib::configuration::server_configuration::ServerConfiguration;
use notifytrap_lib::domain::channel::{
    ChannelChaosConfiguration, ChannelType, CreateChannelInput, UpdateChannelRulesInput,
    WebhookMockConfiguration,
};
use notifytrap_lib::domain::notification::{
    NotificationQuery, NotificationStatus, WebhookReplayInput,
};
use notifytrap_lib::domain::project::CreateProjectInput;
use notifytrap_lib::domain::workspace::CreateWorkspaceInput;
use notifytrap_lib::events::BroadcastEventPublisher;
use notifytrap_lib::infrastructure::database::connection_manager::DatabaseConnectionManager;
use notifytrap_lib::infrastructure::security::secret_key_provider::EphemeralSecretKeyProvider;

struct TestRuntimeHarness {
    services: ApplicationServices,
    event_publisher: Arc<BroadcastEventPublisher>,
    coordinator: CaptureServerCoordinator,
    http_address: SocketAddr,
    smtp_address: SocketAddr,
    workspace_id: String,
    project_id: String,
    environment_id: String,
}

async fn setup_capture_harness(
    maximum_http_payload_bytes: usize,
    maximum_smtp_message_bytes: usize,
) -> TestRuntimeHarness {
    let database_manager =
        DatabaseConnectionManager::open_in_memory().expect("in-memory sqlite initialized");
    let key_provider = Arc::new(EphemeralSecretKeyProvider::new());
    let services = ApplicationServices::new(database_manager, key_provider);
    let event_publisher = Arc::new(BroadcastEventPublisher::new(64));

    let workspace = services
        .workspaces
        .create_workspace(CreateWorkspaceInput {
            name: "Integration Workspace".to_string(),
            slug: Some("integration-workspace".to_string()),
            description: None,
        })
        .expect("workspace created");

    let project = services
        .projects
        .create_project(CreateProjectInput {
            workspace_id: workspace.id.clone(),
            name: "Capture Target".to_string(),
            slug: Some("capture-target".to_string()),
            description: None,
        })
        .expect("project created");

    let environments = services
        .environments
        .list_environments_by_project(&project.id)
        .expect("environments listed");
    let environment_id = environments[0].id.clone();

    let configuration = ServerConfiguration {
        bind_address: "127.0.0.1".to_string(),
        http_port: 0,
        smtp_port: 0,
        maximum_http_payload_bytes,
        maximum_smtp_message_bytes,
        shutdown_grace_period_seconds: 2,
    };

    let coordinator = CaptureServerCoordinator::new(&configuration);
    let status = coordinator
        .start_or_restart(&configuration, services.clone(), event_publisher.clone())
        .await
        .expect("local capture servers should bind cleanly on loopback");

    assert!(status.is_http_server_running);
    assert!(status.is_smtp_server_running);

    TestRuntimeHarness {
        services,
        event_publisher,
        coordinator,
        http_address: status
            .http_endpoint
            .parse()
            .expect("valid bound http address"),
        smtp_address: status
            .smtp_endpoint
            .parse()
            .expect("valid bound smtp address"),
        workspace_id: workspace.id,
        project_id: project.id,
        environment_id,
    }
}

async fn send_raw_http_request(
    address: SocketAddr,
    method: &str,
    path_and_query: &str,
    extra_headers: &[(&str, &str)],
    body: &str,
) -> (u16, String) {
    let mut stream = TcpStream::connect(address)
        .await
        .expect("connect to http capture server");

    let mut request = format!(
        "{method} {path_and_query} HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\nContent-Length: {}\r\n",
        body.len()
    );
    for (name, value) in extra_headers {
        request.push_str(&format!("{name}: {value}\r\n"));
    }
    request.push_str("\r\n");
    request.push_str(body);

    stream
        .write_all(request.as_bytes())
        .await
        .expect("write raw http request");
    stream.flush().await.expect("flush http stream");

    let mut raw_response = Vec::new();
    stream
        .read_to_end(&mut raw_response)
        .await
        .expect("read http response");

    let text = String::from_utf8_lossy(&raw_response).into_owned();
    let (head, response_body) = text
        .split_once("\r\n\r\n")
        .expect("valid HTTP response separator");
    let status_line = head.lines().next().expect("HTTP status line");
    let status_code: u16 = status_line
        .split_whitespace()
        .nth(1)
        .expect("HTTP status code")
        .parse()
        .expect("numeric status code");

    (status_code, response_body.to_string())
}

async fn read_smtp_reply(reader: &mut BufReader<tokio::net::tcp::OwnedReadHalf>) -> (u16, String) {
    let mut full_reply = String::new();
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line).await.expect("read smtp reply");
        assert!(bytes > 0, "unexpected SMTP EOF");
        full_reply.push_str(&line);
        let bytes_slice = line.as_bytes();
        if bytes_slice.len() >= 4 && bytes_slice[3] == b' ' {
            let code: u16 = line[..3].parse().expect("numeric smtp reply code");
            return (code, full_reply);
        }
    }
}

async fn write_smtp_command(writer: &mut tokio::net::tcp::OwnedWriteHalf, command: &str) {
    writer
        .write_all(format!("{command}\r\n").as_bytes())
        .await
        .expect("write smtp command");
    writer.flush().await.expect("flush smtp writer");
}

#[tokio::test]
async fn http_capture_endpoints_handle_sms_push_webhook_health_auth_and_limits() {
    let harness = setup_capture_harness(4096, 1024 * 1024).await;
    let mut event_receiver = harness.event_publisher.subscribe_notifications();

    // 1. Health check
    let (health_status, health_body) =
        send_raw_http_request(harness.http_address, "GET", "/api/v1/health", &[], "").await;
    assert_eq!(health_status, 200);
    let health_json: Value = serde_json::from_str(&health_body).expect("health json");
    assert_eq!(health_json["status"], "ok");

    // 2. Provision SMS, Push, Webhook, Email channels
    let sms_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "SMS Channel".to_string(),
            channel_type: ChannelType::Sms,
            credential_identifier: None,
            is_enabled: Some(true),
        })
        .expect("sms channel");

    let push_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "Push Channel".to_string(),
            channel_type: ChannelType::Push,
            credential_identifier: None,
            is_enabled: Some(true),
        })
        .expect("push channel");

    let webhook_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "Webhook Channel".to_string(),
            channel_type: ChannelType::Webhook,
            credential_identifier: None,
            is_enabled: Some(true),
        })
        .expect("webhook channel");

    // 3. Valid SMS capture
    let sms_body = json!({
        "to": "+989120000000",
        "message": "Verification code: 482-910",
        "from": "NotifyApp",
        "metadata": { "template": "otp" }
    })
    .to_string();

    let (sms_status, sms_reply) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/sms",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
        ],
        &sms_body,
    )
    .await;
    assert_eq!(sms_status, 202, "sms reply: {sms_reply}");
    let sms_event = event_receiver.recv().await.expect("sms event emitted");
    assert_eq!(sms_event.notification.channel_type, ChannelType::Sms);
    assert_eq!(sms_event.notification.recipient, "+989120000000");

    // 4. Reject mismatched channel type (Push key sent to SMS route) & invalid SMS body & oversized body
    let (wrong_type_status, _) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/sms",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &push_channel.credential.composite_key),
        ],
        &sms_body,
    )
    .await;
    assert_eq!(wrong_type_status, 401);

    let (bad_sms_status, _) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/sms",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
        ],
        r#"{"to":"","message":""}"#,
    )
    .await;
    assert_eq!(bad_sms_status, 400);

    let oversized_payload = format!(r#"{{"to":"+123","message":"{}"}}"#, "x".repeat(6000));
    let (oversized_status, _) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/sms",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
        ],
        &oversized_payload,
    )
    .await;
    assert_eq!(oversized_status, 413);

    // 5. Valid Push capture
    let push_body = json!({
        "device_token": "fcm-token-9988",
        "platform": "android",
        "title": "Deployment Ready",
        "body": "Staging build #42 completed",
        "data": { "screen": "deployments" }
    })
    .to_string();

    let (push_status, _) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/push",
        &[
            ("Content-Type", "application/json"),
            (
                "Authorization",
                &format!("Bearer {}", push_channel.credential.composite_key),
            ),
        ],
        &push_body,
    )
    .await;
    assert_eq!(push_status, 202);
    let push_event = event_receiver.recv().await.expect("push event emitted");
    assert_eq!(push_event.notification.channel_type, ChannelType::Push);

    // 6. Webhook capture via subpath + query fallback + sensitive header/query secret redaction
    let webhook_path = format!(
        "/api/v1/webhooks/{}/stripe/events?event=invoice.paid&channel_key={}&token=tok_secret&Api_Key=key_secret",
        webhook_channel.credential.credential_identifier, webhook_channel.credential.secret_token,
    );
    let (webhook_status, webhook_reply) = send_raw_http_request(
        harness.http_address,
        "PUT",
        &webhook_path,
        &[
            ("Content-Type", "application/json"),
            ("X-Custom-Trace", "trace-123"),
            ("X-Api-Key", "sensitive-api-key"),
            ("Cookie", "session=secret-cookie"),
        ],
        r#"{"invoiceId":"inv_42","paid":true}"#,
    )
    .await;
    assert_eq!(webhook_status, 202);
    let webhook_receipt: Value = serde_json::from_str(&webhook_reply).expect("receipt json");
    let webhook_notification_id = webhook_receipt["notificationId"]
        .as_str()
        .expect("notificationId");

    let webhook_detail = harness
        .services
        .notifications
        .get_notification_detail(webhook_notification_id)
        .expect("webhook stored");
    assert_eq!(webhook_detail.notification.payload_json["method"], "PUT");
    assert_eq!(
        webhook_detail.notification.payload_json["subpath"],
        "/stripe/events"
    );
    assert_eq!(
        webhook_detail.notification.payload_json["query"]["channel_key"],
        "[REDACTED]"
    );
    assert_eq!(
        webhook_detail.notification.payload_json["query"]["token"],
        "[REDACTED]"
    );
    assert_eq!(
        webhook_detail.notification.payload_json["query"]["Api_Key"],
        "[REDACTED]"
    );
    assert_eq!(
        webhook_detail.notification.payload_json["headers"]["x-api-key"],
        "[REDACTED]"
    );
    assert_eq!(
        webhook_detail.notification.payload_json["headers"]["cookie"],
        "[REDACTED]"
    );
    assert_eq!(
        webhook_detail.notification.payload_json["headers"]["x-custom-trace"],
        "trace-123"
    );

    harness
        .coordinator
        .shutdown_all(Some(&*harness.event_publisher))
        .await;
}

#[tokio::test]
async fn smtp_capture_server_authenticates_parses_mime_attachments_and_enforces_limits() {
    let harness = setup_capture_harness(1024 * 1024, 4096).await;
    let mut event_receiver = harness.event_publisher.subscribe_notifications();

    let email_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "SMTP Local".to_string(),
            channel_type: ChannelType::Email,
            credential_identifier: None,
            is_enabled: Some(true),
        })
        .expect("email channel");

    // 1. Unauthenticated MAIL FROM rejection (530) + invalid password (535)
    {
        let stream = TcpStream::connect(harness.smtp_address)
            .await
            .expect("connect smtp");
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        assert_eq!(read_smtp_reply(&mut reader).await.0, 220);

        write_smtp_command(&mut write_half, "EHLO localhost").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);

        write_smtp_command(&mut write_half, "MAIL FROM:<app@local.test>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 530);

        let bad_plain = STANDARD.encode(format!(
            "\0{}\0wrong-secret",
            email_channel.credential.credential_identifier
        ));
        write_smtp_command(&mut write_half, &format!("AUTH PLAIN {bad_plain}")).await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 535);

        write_smtp_command(&mut write_half, "QUIT").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 221);
    }

    // 2. Successful AUTH PLAIN + multipart MIME + HTML XSS sanitization + dot unstuffing + PDF attachment
    {
        let stream = TcpStream::connect(harness.smtp_address)
            .await
            .expect("connect smtp");
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        assert_eq!(read_smtp_reply(&mut reader).await.0, 220);
        write_smtp_command(&mut write_half, "EHLO notifytrap.client").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);

        let auth_plain = STANDARD.encode(format!(
            "\0{}\0{}",
            email_channel.credential.credential_identifier, email_channel.credential.secret_token
        ));
        write_smtp_command(&mut write_half, &format!("AUTH PLAIN {auth_plain}")).await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 235);

        write_smtp_command(&mut write_half, "MAIL FROM:<mailer@arsam.local>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);

        write_smtp_command(&mut write_half, "RCPT TO:<farshid@example.com>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);

        write_smtp_command(&mut write_half, "DATA").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 354);

        let pdf_base64 = STANDARD.encode(b"%PDF-1.4\nhello");
        let mime_payload = format!(
            "From: Arsam Mailer <mailer@arsam.local>\r\n\
             To: Farshid <farshid@example.com>\r\n\
             Cc: audit@arsam.local\r\n\
             Subject: Invoice Ready\r\n\
             MIME-Version: 1.0\r\n\
             Content-Type: multipart/mixed; boundary=\"MIXED_BOUND\"\r\n\r\n\
             --MIXED_BOUND\r\n\
             Content-Type: multipart/alternative; boundary=\"ALT_BOUND\"\r\n\r\n\
             --ALT_BOUND\r\n\
             Content-Type: text/plain; charset=utf-8\r\n\r\n\
             Hello plain text\r\n\
             ..dot-stuffed line\r\n\r\n\
             --ALT_BOUND\r\n\
             Content-Type: text/html; charset=utf-8\r\n\r\n\
             <div><p>Hello HTML</p><script>alert('xss')</script></div>\r\n\r\n\
             --ALT_BOUND--\r\n\
             --MIXED_BOUND\r\n\
             Content-Type: application/pdf; name=\"invoice.pdf\"\r\n\
             Content-Disposition: attachment; filename=\"invoice.pdf\"\r\n\
             Content-Transfer-Encoding: base64\r\n\r\n\
             {pdf_base64}\r\n\
             --MIXED_BOUND--\r\n\
             .\r\n"
        );

        write_half
            .write_all(mime_payload.as_bytes())
            .await
            .expect("send mime body");
        write_half.flush().await.expect("flush mime body");

        let (data_code, data_reply) = read_smtp_reply(&mut reader).await;
        assert_eq!(data_code, 250, "smtp data reply: {data_reply}");

        // 3. Reject oversized DATA message (limit 4096 bytes)
        write_smtp_command(&mut write_half, "MAIL FROM:<mailer@arsam.local>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "RCPT TO:<farshid@example.com>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "DATA").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 354);

        let oversized_email = format!("Subject: Too Big\r\n\r\n{}\r\n.\r\n", "A".repeat(5000));
        write_half
            .write_all(oversized_email.as_bytes())
            .await
            .expect("send oversized email");
        write_half.flush().await.expect("flush oversized email");
        assert_eq!(read_smtp_reply(&mut reader).await.0, 552);

        write_smtp_command(&mut write_half, "QUIT").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 221);
    }

    let smtp_event = event_receiver.recv().await.expect("smtp event emitted");
    assert_eq!(smtp_event.notification.channel_type, ChannelType::Email);
    assert_eq!(smtp_event.notification.subject, "Invoice Ready");
    assert_eq!(smtp_event.attachment_count, 1);

    let detail = harness
        .services
        .notifications
        .get_notification_detail(&smtp_event.notification.id)
        .expect("email stored");
    assert_eq!(detail.notification.status, NotificationStatus::Captured);
    assert!(
        detail.notification.payload_json["textBody"]
            .as_str()
            .unwrap_or_default()
            .contains(".dot-stuffed line")
    );
    let sanitized_html = detail.notification.payload_json["sanitizedHtml"]
        .as_str()
        .unwrap_or_default();
    assert!(sanitized_html.contains("<p>Hello HTML</p>"));
    assert!(!sanitized_html.contains("<script"));
    assert_eq!(detail.attachments.len(), 1);
    assert_eq!(detail.attachments[0].filename, "invoice.pdf");

    let blob = harness
        .services
        .notifications
        .get_attachment(&detail.attachments[0].id)
        .expect("pdf attachment readable");
    assert_eq!(blob.content_blob, b"%PDF-1.4\nhello");

    // 4. Verify AUTH LOGIN works as well
    {
        let stream = TcpStream::connect(harness.smtp_address)
            .await
            .expect("connect smtp");
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);

        assert_eq!(read_smtp_reply(&mut reader).await.0, 220);
        write_smtp_command(&mut write_half, "AUTH LOGIN").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 334);
        write_smtp_command(
            &mut write_half,
            &STANDARD.encode(&email_channel.credential.credential_identifier),
        )
        .await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 334);
        write_smtp_command(
            &mut write_half,
            &STANDARD.encode(&email_channel.credential.secret_token),
        )
        .await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 235);
        write_smtp_command(&mut write_half, "MAIL FROM:<login@arsam.local>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "RCPT TO:<user@example.com>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "DATA").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 354);
        write_half
            .write_all(b"Subject: Login Auth OK\r\n\r\nVerified AUTH LOGIN.\r\n.\r\n")
            .await
            .expect("send login email");
        write_half.flush().await.expect("flush login email");
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "QUIT").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 221);
    }

    let page = harness
        .services
        .notifications
        .query_notifications(NotificationQuery {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            channel_id: None,
            channel_type: Some(ChannelType::Email),
            status: None,
            search_query: None,
            limit: Some(10),
            offset: Some(0),
            ..NotificationQuery::default()
        })
        .expect("query captured emails");
    assert_eq!(page.total_count, 2);

    // 5. Reject abusive >8KB command line cleanly without buffering unbounded input
    {
        let abusive_stream = TcpStream::connect(harness.smtp_address)
            .await
            .expect("connect smtp abusive");
        let (read_half, mut write_half) = abusive_stream.into_split();
        let mut reader = BufReader::new(read_half);
        assert_eq!(read_smtp_reply(&mut reader).await.0, 220);

        let oversized_command = format!("EHLO {}\r\n", "x".repeat(9000));
        let _ = write_half.write_all(oversized_command.as_bytes()).await;
        let _ = write_half.flush().await;
        let mut probe_line = String::new();
        let read_bytes = reader.read_line(&mut probe_line).await.unwrap_or(0);
        assert_eq!(read_bytes, 0, "server must close oversized command line");
    }

    harness
        .coordinator
        .shutdown_all(Some(&*harness.event_publisher))
        .await;
}

#[tokio::test]
async fn http_query_wait_run_tag_and_verification_extraction_end_to_end() {
    let harness = setup_capture_harness(2 * 1024 * 1024, 2 * 1024 * 1024).await;

    let sms_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "Automation SMS".to_string(),
            channel_type: ChannelType::Sms,
            credential_identifier: None,
            is_enabled: Some(true),
        })
        .expect("create sms channel");

    let email_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "Automation Email".to_string(),
            channel_type: ChannelType::Email,
            credential_identifier: None,
            is_enabled: Some(true),
        })
        .expect("create email channel");

    // 1. Capture SMS with X-NotifyTrap-Run and Persian/English OTP + magic link
    let sms_body = json!({
        "to": "+989120001122",
        "from": "NotifyAuth",
        "message": "کد تایید شما: ۴۸۲۹۱۰ — https://app.example.local/auth/verify?token=run1"
    })
    .to_string();

    let (capture_status, capture_response_text) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/sms",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
            ("X-NotifyTrap-Run", "ci-run-42"),
        ],
        &sms_body,
    )
    .await;
    assert_eq!(capture_status, 202);
    let capture_json: Value = serde_json::from_str(&capture_response_text).expect("receipt json");
    let first_sms_id = capture_json["notificationId"]
        .as_str()
        .expect("notification id")
        .to_string();

    // 2. Query API scoped by run_id returns verificationArtifacts & testRunIdentifier
    let (query_status, query_body) = send_raw_http_request(
        harness.http_address,
        "GET",
        "/api/v1/notifications?run_id=ci-run-42",
        &[("X-Channel-Key", &sms_channel.credential.composite_key)],
        "",
    )
    .await;
    assert_eq!(query_status, 200);
    let query_json: Value = serde_json::from_str(&query_body).expect("query json");
    assert_eq!(query_json["totalCount"], 1);
    assert_eq!(query_json["items"][0]["testRunIdentifier"], "ci-run-42");
    let artifacts = query_json["items"][0]["verificationArtifacts"]
        .as_array()
        .expect("verificationArtifacts array");
    assert!(artifacts.iter().any(|item| item["value"] == "482910"));
    assert!(
        artifacts
            .iter()
            .any(|item| item["value"] == "https://app.example.local/auth/verify?token=run1")
    );

    // 3. Cross-channel detail lookup is strictly isolated (404)
    let (cross_status, _) = send_raw_http_request(
        harness.http_address,
        "GET",
        &format!("/api/v1/notifications/{first_sms_id}"),
        &[("X-Channel-Key", &email_channel.credential.composite_key)],
        "",
    )
    .await;
    assert_eq!(cross_status, 404);

    // 4. Wait endpoint wakes immediately on concurrent capture without polling sleep
    let wait_headers = [(
        "X-Channel-Key",
        sms_channel.credential.composite_key.as_str(),
    )];
    let wait_future = send_raw_http_request(
        harness.http_address,
        "GET",
        "/api/v1/notifications/wait?run_id=ci-run-99&timeout_ms=3000",
        &wait_headers,
        "",
    );
    let trigger_future = async {
        tokio::task::yield_now().await;
        let delayed_sms = json!({
            "to": "+989120003344",
            "message": "Your login code is 991122"
        })
        .to_string();
        send_raw_http_request(
            harness.http_address,
            "POST",
            "/api/v1/sms",
            &[
                ("Content-Type", "application/json"),
                ("X-Channel-Key", &sms_channel.credential.composite_key),
                ("X-NotifyTrap-Run", "ci-run-99"),
            ],
            &delayed_sms,
        )
        .await
    };
    let ((wait_status, wait_body), (trigger_status, _)) = tokio::join!(wait_future, trigger_future);
    assert_eq!(trigger_status, 202);
    assert_eq!(wait_status, 200);
    let wait_json: Value = serde_json::from_str(&wait_body).expect("wait detail json");
    assert_eq!(wait_json["notification"]["testRunIdentifier"], "ci-run-99");
    assert_eq!(
        wait_json["notification"]["verificationArtifacts"][0]["value"],
        "991122"
    );

    // 5. Wait endpoint returns 408 WAIT_TIMEOUT when no matching item arrives
    let (timeout_status, timeout_body) = send_raw_http_request(
        harness.http_address,
        "GET",
        "/api/v1/notifications/wait?run_id=never-arrives&timeout_ms=80",
        &[("X-Channel-Key", &sms_channel.credential.composite_key)],
        "",
    )
    .await;
    assert_eq!(timeout_status, 408);
    let timeout_json: Value = serde_json::from_str(&timeout_body).expect("timeout json");
    assert_eq!(timeout_json["code"], "WAIT_TIMEOUT");

    // 6. SMTP capture preserves X-NotifyTrap-Run header
    {
        let stream = TcpStream::connect(harness.smtp_address)
            .await
            .expect("connect smtp");
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        assert_eq!(read_smtp_reply(&mut reader).await.0, 220);
        write_smtp_command(&mut write_half, "EHLO localhost").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        let auth_plain = STANDARD.encode(format!(
            "\0{}\0{}",
            email_channel.channel.credential_identifier, email_channel.credential.secret_token
        ));
        write_smtp_command(&mut write_half, &format!("AUTH PLAIN {auth_plain}")).await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 235);
        write_smtp_command(&mut write_half, "MAIL FROM:<ci@notifytrap.local>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "RCPT TO:<qa@notifytrap.local>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "DATA").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 354);
        write_half
            .write_all(
                b"From: ci@notifytrap.local\r\nTo: qa@notifytrap.local\r\nSubject: Reset Password\r\nX-NotifyTrap-Run: smtp-suite-7\r\n\r\nUse code 654321.\r\n.\r\n",
            )
            .await
            .expect("write smtp data");
        write_half.flush().await.expect("flush smtp");
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "QUIT").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 221);
    }

    let (smtp_query_status, smtp_query_body) = send_raw_http_request(
        harness.http_address,
        "GET",
        "/api/v1/notifications?run_id=smtp-suite-7",
        &[("X-Channel-Key", &email_channel.credential.composite_key)],
        "",
    )
    .await;
    assert_eq!(smtp_query_status, 200);
    let smtp_query_json: Value = serde_json::from_str(&smtp_query_body).expect("smtp query json");
    assert_eq!(smtp_query_json["totalCount"], 1);
    assert_eq!(
        smtp_query_json["items"][0]["verificationArtifacts"][0]["value"],
        "654321"
    );

    harness
        .coordinator
        .shutdown_all(Some(&*harness.event_publisher))
        .await;
}

#[tokio::test]
async fn webhook_mock_loopback_replay_and_chaos_rules_end_to_end() {
    let harness = setup_capture_harness(1024 * 1024, 4096).await;

    let webhook_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "Stripe Mock Hook".to_string(),
            channel_type: ChannelType::Webhook,
            credential_identifier: Some("stripe-mock-hook".to_string()),
            is_enabled: Some(true),
        })
        .expect("create webhook channel");

    let email_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "Chaos Email".to_string(),
            channel_type: ChannelType::Email,
            credential_identifier: Some("chaos-email".to_string()),
            is_enabled: Some(true),
        })
        .expect("create email channel");

    // 1. Configure custom webhook mock response (201 + custom header + custom JSON body)
    let mut mock_headers = BTreeMap::new();
    mock_headers.insert("X-Mock-Reply".to_string(), "stripe-ok".to_string());
    let updated_webhook = harness
        .services
        .channels
        .update_channel_rules(UpdateChannelRulesInput {
            channel_id: webhook_channel.channel.id.clone(),
            webhook_mock: WebhookMockConfiguration {
                is_enabled: true,
                status_code: 201,
                headers: mock_headers,
                body: r#"{"acknowledged":true,"source":"local-mock"}"#.to_string(),
                delay_milliseconds: 0,
            },
            chaos_rule: ChannelChaosConfiguration::default(),
            forwarding_rule: notifytrap_lib::domain::forwarding::ChannelForwardingConfiguration::default(),
        })
        .expect("update webhook mock rules");
    assert!(updated_webhook.webhook_mock.is_enabled);
    assert_eq!(updated_webhook.webhook_mock.status_code, 201);

    let (mock_status, mock_body) = send_raw_http_request(
        harness.http_address,
        "POST",
        &format!(
            "/api/v1/webhooks/{}/checkout",
            webhook_channel.credential.credential_identifier
        ),
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &webhook_channel.credential.composite_key),
            ("X-Replay-Signature", "sig_local_123"),
        ],
        r#"{"event":"checkout.completed"}"#,
    )
    .await;
    assert_eq!(mock_status, 201);
    assert_eq!(mock_body, r#"{"acknowledged":true,"source":"local-mock"}"#);

    let stored_page = harness
        .services
        .notifications
        .query_notifications(NotificationQuery {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            channel_id: None,
            channel_type: Some(ChannelType::Webhook),
            status: None,
            test_run_identifier: None,
            recipient_contains: None,
            subject_contains: None,
            since: None,
            search_query: None,
            limit: Some(10),
            offset: Some(0),
        })
        .expect("query stored mocked webhook");
    assert_eq!(stored_page.total_count, 1);
    let captured_webhook_id = stored_page.items[0].id.clone();

    // 2. Strict loopback replay against an ephemeral 127.0.0.1 receiver
    let replay_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind local replay receiver");
    let replay_port = replay_listener.local_addr().expect("local addr").port();
    let receiver_task = tokio::spawn(async move {
        let (mut socket, _) = replay_listener.accept().await.expect("accept replay");
        let mut request_buf = [0_u8; 4096];
        let read_len = socket.read(&mut request_buf).await.expect("read replay");
        let request_text = String::from_utf8_lossy(&request_buf[..read_len]).to_string();
        let response = "HTTP/1.1 202 Accepted\r\nContent-Type: application/json\r\nContent-Length: 19\r\n\r\n{\"replayed\":\"done\"}";
        socket
            .write_all(response.as_bytes())
            .await
            .expect("write replay");
        socket.flush().await.expect("flush replay");
        request_text
    });

    let replay_result = harness
        .services
        .notifications
        .replay_webhook_notification(WebhookReplayInput {
            notification_id: captured_webhook_id.clone(),
            target_url: format!("http://127.0.0.1:{replay_port}/local-receiver"),
        })
        .await
        .expect("loopback replay succeeds");
    assert_eq!(replay_result.status_code, 202);
    assert_eq!(replay_result.method, "POST");
    assert_eq!(replay_result.response_body, r#"{"replayed":"done"}"#);

    let received_raw = receiver_task.await.expect("receiver task");
    assert!(received_raw.starts_with("POST /local-receiver HTTP/1.1\r\n"));
    assert!(
        received_raw
            .to_ascii_lowercase()
            .contains("x-replay-signature: sig_local_123")
    );
    assert!(received_raw.contains(r#"{"event":"checkout.completed"}"#));

    // Reject non-loopback replay targets
    let external_replay_error = harness
        .services
        .notifications
        .replay_webhook_notification(WebhookReplayInput {
            notification_id: captured_webhook_id,
            target_url: "http://example.com/webhook".to_string(),
        })
        .await
        .expect_err("external host replay must be rejected");
    assert!(external_replay_error.to_string().contains("127.0.0.1"));

    // 3. Chaos failure injection on HTTP webhook (429) and SMTP email (451)
    harness
        .services
        .channels
        .update_channel_rules(UpdateChannelRulesInput {
            channel_id: webhook_channel.channel.id.clone(),
            webhook_mock: WebhookMockConfiguration::default(),
            chaos_rule: ChannelChaosConfiguration {
                is_enabled: true,
                latency_milliseconds: 0,
                failure_rate_percent: 100,
                http_error_status_code: 429,
                smtp_error_code: 451,
                error_message: "Rate limit simulated".to_string(),
            },
            forwarding_rule: notifytrap_lib::domain::forwarding::ChannelForwardingConfiguration::default(),
        })
        .expect("enable webhook chaos");

    let (chaos_http_status, chaos_http_body) = send_raw_http_request(
        harness.http_address,
        "POST",
        &format!(
            "/api/v1/webhooks/{}/checkout",
            webhook_channel.credential.credential_identifier
        ),
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &webhook_channel.credential.composite_key),
        ],
        r#"{"attempt":2}"#,
    )
    .await;
    assert_eq!(chaos_http_status, 429);
    assert!(chaos_http_body.contains("CHAOS_SIMULATED_FAILURE"));

    harness
        .services
        .channels
        .update_channel_rules(UpdateChannelRulesInput {
            channel_id: email_channel.channel.id.clone(),
            webhook_mock: WebhookMockConfiguration::default(),
            chaos_rule: ChannelChaosConfiguration {
                is_enabled: true,
                latency_milliseconds: 0,
                failure_rate_percent: 100,
                http_error_status_code: 503,
                smtp_error_code: 451,
                error_message: "Transient mailbox unavailable".to_string(),
            },
            forwarding_rule: notifytrap_lib::domain::forwarding::ChannelForwardingConfiguration::default(),
        })
        .expect("enable smtp chaos");

    {
        let stream = TcpStream::connect(harness.smtp_address)
            .await
            .expect("connect smtp");
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        assert_eq!(read_smtp_reply(&mut reader).await.0, 220);
        write_smtp_command(&mut write_half, "EHLO localhost").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        let auth_plain = STANDARD.encode(format!(
            "\0{}\0{}",
            email_channel.channel.credential_identifier, email_channel.credential.secret_token
        ));
        write_smtp_command(&mut write_half, &format!("AUTH PLAIN {auth_plain}")).await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 235);
        write_smtp_command(&mut write_half, "MAIL FROM:<ci@notifytrap.local>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "RCPT TO:<qa@notifytrap.local>").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 250);
        write_smtp_command(&mut write_half, "DATA").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 354);
        write_half
            .write_all(b"From: ci@notifytrap.local\r\nTo: qa@notifytrap.local\r\nSubject: Chaos\r\n\r\nHello\r\n.\r\n")
            .await
            .expect("write smtp data");
        write_half.flush().await.expect("flush smtp");
        assert_eq!(read_smtp_reply(&mut reader).await.0, 451);
        write_smtp_command(&mut write_half, "QUIT").await;
        assert_eq!(read_smtp_reply(&mut reader).await.0, 221);
    }

    harness
        .coordinator
        .shutdown_all(Some(&*harness.event_publisher))
        .await;
}

#[tokio::test]
async fn structured_search_and_local_mcp_json_rpc_end_to_end() {
    let harness = setup_capture_harness(1024 * 1024, 4096).await;

    let sms_channel = harness
        .services
        .channels
        .create_channel(CreateChannelInput {
            workspace_id: harness.workspace_id.clone(),
            project_id: harness.project_id.clone(),
            environment_id: harness.environment_id.clone(),
            name: "MCP SMS".to_string(),
            channel_type: ChannelType::Sms,
            credential_identifier: Some("mcp-sms".to_string()),
            is_enabled: Some(true),
        })
        .expect("create sms channel");

    let (status_1, _) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/sms",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
            ("X-NotifyTrap-Run", "mcp-run-1"),
        ],
        r#"{"to":"+989120001111","from":"Auth","message":"Your verification code is 774123"}"#,
    )
    .await;
    assert_eq!(status_1, 202);

    let (status_2, _) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/sms",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
            ("X-NotifyTrap-Run", "mcp-run-2"),
        ],
        r#"{"to":"+989120002222","from":"Promo","message":"Weekly newsletter digest"}"#,
    )
    .await;
    assert_eq!(status_2, 202);

    // Structured search: channel:sms otp:true /774[0-9]{3}/
    let (search_status, search_body) = send_raw_http_request(
        harness.http_address,
        "GET",
        "/api/v1/notifications?search=channel%3Asms%20otp%3Atrue%20%2F774%5B0-9%5D%7B3%7D%2F",
        &[("X-Channel-Key", &sms_channel.credential.composite_key)],
        "",
    )
    .await;
    assert_eq!(search_status, 200);
    let search_json: Value = serde_json::from_str(&search_body).expect("search json");
    assert_eq!(search_json["totalCount"], 1);
    assert_eq!(search_json["items"][0]["testRunIdentifier"], "mcp-run-1");

    // MCP initialize
    let (mcp_init_status, mcp_init_body) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/mcp",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
        ],
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
    )
    .await;
    assert_eq!(mcp_init_status, 200);
    let mcp_init_json: Value = serde_json::from_str(&mcp_init_body).expect("mcp init json");
    assert_eq!(
        mcp_init_json["result"]["serverInfo"]["name"],
        "notifytrap-local-mcp"
    );

    // MCP tools/list
    let (mcp_list_status, mcp_list_body) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/mcp",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
        ],
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
    )
    .await;
    assert_eq!(mcp_list_status, 200);
    let mcp_list_json: Value = serde_json::from_str(&mcp_list_body).expect("mcp list json");
    assert_eq!(
        mcp_list_json["result"]["tools"].as_array().map(Vec::len),
        Some(4)
    );

    // MCP tools/call -> extract_otp_codes
    let (mcp_call_status, mcp_call_body) = send_raw_http_request(
        harness.http_address,
        "POST",
        "/api/v1/mcp",
        &[
            ("Content-Type", "application/json"),
            ("X-Channel-Key", &sms_channel.credential.composite_key),
        ],
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"extract_otp_codes","arguments":{"run_id":"mcp-run-1"}}}"#,
    )
    .await;
    assert_eq!(mcp_call_status, 200);
    let mcp_call_json: Value = serde_json::from_str(&mcp_call_body).expect("mcp call json");
    let text_payload = mcp_call_json["result"]["content"][0]["text"]
        .as_str()
        .expect("mcp tool text");
    assert!(text_payload.contains("774123"));

    harness
        .coordinator
        .shutdown_all(Some(&*harness.event_publisher))
        .await;
}
