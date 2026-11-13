use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use notifytrap_lib::application::application_state::ApplicationServices;
use notifytrap_lib::configuration::server_configuration::ServerConfiguration;
use notifytrap_lib::events::BroadcastEventPublisher;
use notifytrap_lib::infrastructure::database::connection_manager::DatabaseConnectionManager;
use notifytrap_lib::infrastructure::http::routes::HttpCaptureState;
use notifytrap_lib::infrastructure::http::server::RunningHttpServer;
use notifytrap_lib::infrastructure::http::web_server::RunningWebCompanionServer;
use notifytrap_lib::infrastructure::security::secret_key_provider::EphemeralSecretKeyProvider;
use notifytrap_lib::infrastructure::smtp::server::RunningSmtpServer;

#[tokio::test]
async fn headless_server_and_web_companion_end_to_end_lifecycle() {
    let database_manager = DatabaseConnectionManager::open_in_memory()
        .expect("in-memory database should initialize");
    let key_provider = Arc::new(EphemeralSecretKeyProvider::new());
    let services = ApplicationServices::new(database_manager, key_provider);
    let event_publisher = Arc::new(BroadcastEventPublisher::default());

    let server_config = ServerConfiguration {
        bind_address: "127.0.0.1".to_string(),
        http_port: 0, // Auto-bind port
        smtp_port: 0,
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
    .expect("start headless http server");

    let smtp_server = RunningSmtpServer::start(
        &server_config,
        services.clone(),
        Arc::clone(&event_publisher) as Arc<dyn notifytrap_lib::events::EventPublisher>,
    )
    .await
    .expect("start headless smtp server");

    let capture_state = HttpCaptureState {
        services: services.clone(),
        event_publisher: Arc::clone(&event_publisher) as Arc<dyn notifytrap_lib::events::EventPublisher>,
        bound_endpoint: http_server.bound_address().to_string(),
    };

    let mut assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    assets_dir.pop();
    assets_dir.push(".output");
    assets_dir.push("public");

    let web_companion = RunningWebCompanionServer::start(
        "127.0.0.1",
        0,
        Some(assets_dir),
        capture_state,
        10 * 1024 * 1024,
    )
    .await
    .expect("start headless web companion");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .expect("client");

    // 1. Verify HTTP Capture Health Endpoint
    let health_url = format!("http://{}/api/v1/health", http_server.bound_address());
    let health_response = client.get(&health_url).send().await.expect("health request");
    assert!(health_response.status().is_success());

    // 2. Verify Web Companion Index Serving
    let web_url = format!("http://{}/index.html", web_companion.bound_address());
    let web_response = client.get(&web_url).send().await.expect("web companion request");
    assert!(web_response.status().is_success());

    // 3. Graceful Shutdown Lifecycle
    http_server.shutdown().await;
    smtp_server.shutdown().await;
    web_companion.stop().await;
}
