use std::sync::{Arc, RwLock};

use tokio::sync::Mutex;

use crate::application::application_state::ApplicationServices;
use crate::configuration::server_configuration::ServerConfiguration;
use crate::domain::errors::ApplicationError;
use crate::events::{EventPublisher, ServerStatusChangedEvent};
use crate::infrastructure::http::server::RunningHttpServer;
use crate::infrastructure::smtp::server::RunningSmtpServer;

struct RuntimeHandles {
    http_server: Option<RunningHttpServer>,
    smtp_server: Option<RunningSmtpServer>,
}

/// Manages lifecycle, status reporting, and graceful shutdown of local HTTP and SMTP servers.
#[derive(Clone)]
pub struct CaptureServerCoordinator {
    handles: Arc<Mutex<RuntimeHandles>>,
    status_snapshot: Arc<RwLock<ServerStatusChangedEvent>>,
}

impl CaptureServerCoordinator {
    pub fn new(initial_configuration: &ServerConfiguration) -> Self {
        let initial_status = ServerStatusChangedEvent {
            is_http_server_running: false,
            is_smtp_server_running: false,
            http_endpoint: format!(
                "{}:{}",
                initial_configuration.bind_address, initial_configuration.http_port
            ),
            smtp_endpoint: format!(
                "{}:{}",
                initial_configuration.bind_address, initial_configuration.smtp_port
            ),
        };

        Self {
            handles: Arc::new(Mutex::new(RuntimeHandles {
                http_server: None,
                smtp_server: None,
            })),
            status_snapshot: Arc::new(RwLock::new(initial_status)),
        }
    }

    pub fn current_status(&self) -> ServerStatusChangedEvent {
        self.status_snapshot
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_else(|_| ServerStatusChangedEvent {
                is_http_server_running: false,
                is_smtp_server_running: false,
                http_endpoint: String::new(),
                smtp_endpoint: String::new(),
            })
    }

    pub async fn start_or_restart(
        &self,
        configuration: &ServerConfiguration,
        services: ApplicationServices,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Result<ServerStatusChangedEvent, ApplicationError> {
        let mut guard = self.handles.lock().await;

        if let Some(previous_http) = guard.http_server.take() {
            previous_http.shutdown().await;
        }
        if let Some(previous_smtp) = guard.smtp_server.take() {
            previous_smtp.shutdown().await;
        }

        let http_server =
            RunningHttpServer::start(configuration, services.clone(), event_publisher.clone())
                .await?;

        let smtp_server = match RunningSmtpServer::start(
            configuration,
            services,
            event_publisher.clone(),
        )
        .await
        {
            Ok(server) => server,
            Err(error) => {
                http_server.shutdown().await;
                self.update_snapshot_and_emit(
                    ServerStatusChangedEvent {
                        is_http_server_running: false,
                        is_smtp_server_running: false,
                        http_endpoint: format!(
                            "{}:{}",
                            configuration.bind_address, configuration.http_port
                        ),
                        smtp_endpoint: format!(
                            "{}:{}",
                            configuration.bind_address, configuration.smtp_port
                        ),
                    },
                    &*event_publisher,
                );
                return Err(error);
            }
        };

        let status = ServerStatusChangedEvent {
            is_http_server_running: true,
            is_smtp_server_running: true,
            http_endpoint: http_server.bound_address().to_string(),
            smtp_endpoint: smtp_server.bound_address().to_string(),
        };

        guard.http_server = Some(http_server);
        guard.smtp_server = Some(smtp_server);

        self.update_snapshot_and_emit(status.clone(), &*event_publisher);
        Ok(status)
    }

    pub async fn shutdown_all(&self, event_publisher: Option<&dyn EventPublisher>) {
        let mut guard = self.handles.lock().await;

        if let Some(http_server) = guard.http_server.take() {
            http_server.shutdown().await;
        }
        if let Some(smtp_server) = guard.smtp_server.take() {
            smtp_server.shutdown().await;
        }

        let mut next_status = self.current_status();
        next_status.is_http_server_running = false;
        next_status.is_smtp_server_running = false;

        if let Ok(mut snapshot) = self.status_snapshot.write() {
            *snapshot = next_status.clone();
        }
        if let Some(publisher) = event_publisher {
            publisher.emit_server_status_changed(&next_status);
        }
    }

    fn update_snapshot_and_emit(
        &self,
        status: ServerStatusChangedEvent,
        event_publisher: &dyn EventPublisher,
    ) {
        if let Ok(mut snapshot) = self.status_snapshot.write() {
            *snapshot = status.clone();
        }
        event_publisher.emit_server_status_changed(&status);
    }
}
