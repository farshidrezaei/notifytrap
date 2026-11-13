use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;

use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

use crate::domain::errors::ApplicationError;
use crate::infrastructure::http::routes::{HttpCaptureState, build_http_capture_router};

/// Handle to a running Axum Web Companion server serving SPA assets and API endpoints.
pub struct RunningWebCompanionServer {
    bound_address: SocketAddr,
    shutdown_sender: Option<oneshot::Sender<()>>,
    task_handle: tokio::task::JoinHandle<()>,
}

impl RunningWebCompanionServer {
    pub async fn start(
        bind_address: &str,
        web_port: u16,
        static_assets_path: Option<PathBuf>,
        capture_state: HttpCaptureState,
        maximum_payload_bytes: usize,
    ) -> Result<Self, ApplicationError> {
        let bind_ip: IpAddr = bind_address.parse().map_err(|_| {
            ApplicationError::Validation("Invalid web server bind address".to_string())
        })?;
        let target_address = SocketAddr::new(bind_ip, web_port);

        let listener = TcpListener::bind(target_address).await.map_err(|error| {
            ApplicationError::Conflict(format!(
                "Unable to bind Web Companion server on {target_address}: {error}"
            ))
        })?;
        let bound_address = listener.local_addr().map_err(|error| {
            ApplicationError::Internal(format!("Unable to resolve bound web address: {error}"))
        })?;

        let api_router = build_http_capture_router(capture_state, maximum_payload_bytes);

        let mut app_router = api_router.layer(CorsLayer::permissive());

        if let Some(assets_dir) = static_assets_path
            && assets_dir.is_dir()
        {
            let serve_dir = ServeDir::new(&assets_dir).fallback(ServeDir::new(&assets_dir));
            app_router = app_router.fallback_service(serve_dir);
        }

        let (shutdown_sender, shutdown_receiver) = oneshot::channel::<()>();

        let task_handle = tokio::spawn(async move {
            let server = axum::serve(listener, app_router).with_graceful_shutdown(async move {
                let _ = shutdown_receiver.await;
            });

            if let Err(error) = server.await {
                tracing::error!("Web Companion server terminated with error: {error}");
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

    pub async fn stop(mut self) {
        if let Some(sender) = self.shutdown_sender.take() {
            let _ = sender.send(());
        }
        let _ = self.task_handle.await;
    }
}
