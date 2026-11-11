use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use crate::application::application_state::ApplicationServices;
use crate::configuration::server_configuration::ServerConfiguration;
use crate::domain::errors::ApplicationError;
use crate::events::EventPublisher;
use crate::infrastructure::http::routes::{HttpCaptureState, build_http_capture_router};

/// Handle to a running Axum local HTTP capture server supporting graceful shutdown.
pub struct RunningHttpServer {
    bound_address: SocketAddr,
    shutdown_sender: Option<oneshot::Sender<()>>,
    task_handle: JoinHandle<()>,
}

impl RunningHttpServer {
    pub async fn start(
        configuration: &ServerConfiguration,
        services: ApplicationServices,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Result<Self, ApplicationError> {
        let bind_ip: IpAddr = configuration
            .bind_address
            .parse()
            .map_err(|_| ApplicationError::Validation("Invalid HTTP bind address".to_string()))?;
        let target_address = SocketAddr::new(bind_ip, configuration.http_port);

        let listener = TcpListener::bind(target_address).await.map_err(|error| {
            ApplicationError::Conflict(format!(
                "Unable to bind HTTP capture server on {target_address}: {error}"
            ))
        })?;
        let bound_address = listener.local_addr().map_err(|error| {
            ApplicationError::Internal(format!("Unable to resolve bound HTTP address: {error}"))
        })?;

        let router = build_http_capture_router(
            HttpCaptureState {
                services,
                event_publisher,
                bound_endpoint: bound_address.to_string(),
            },
            configuration.maximum_http_payload_bytes,
        );

        let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();
        let task_handle = tokio::spawn(async move {
            let server = axum::serve(listener, router).with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            });
            if let Err(error) = server.await {
                tracing::error!("Local HTTP capture server terminated with error: {error}");
            }
        });

        Ok(Self {
            bound_address,
            shutdown_sender: Some(shutdown_sender),
            task_handle,
        })
    }

    pub fn bound_address(&self) -> SocketAddr {
        self.bound_address
    }

    pub async fn shutdown(mut self) {
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }
        let _ = self.task_handle.await;
    }
}
