use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;
use tokio::sync::{Semaphore, watch};
use tokio::task::{JoinHandle, JoinSet};

use crate::application::application_state::ApplicationServices;
use crate::configuration::server_configuration::ServerConfiguration;
use crate::domain::errors::ApplicationError;
use crate::events::EventPublisher;
use crate::infrastructure::smtp::session::handle_smtp_connection;

const MAXIMUM_CONCURRENT_SMTP_CONNECTIONS: usize = 64;
const SMTP_SHUTDOWN_DRAIN_TIMEOUT: Duration = Duration::from_secs(3);

/// Handle to a running Tokio local SMTP capture server supporting graceful shutdown.
pub struct RunningSmtpServer {
    bound_address: SocketAddr,
    shutdown_sender: watch::Sender<bool>,
    task_handle: JoinHandle<()>,
}

impl RunningSmtpServer {
    pub async fn start(
        configuration: &ServerConfiguration,
        services: ApplicationServices,
        event_publisher: Arc<dyn EventPublisher>,
    ) -> Result<Self, ApplicationError> {
        let bind_ip: IpAddr = configuration
            .bind_address
            .parse()
            .map_err(|_| ApplicationError::Validation("Invalid SMTP bind address".to_string()))?;
        let target_address = SocketAddr::new(bind_ip, configuration.smtp_port);

        let listener = TcpListener::bind(target_address).await.map_err(|error| {
            ApplicationError::Conflict(format!(
                "Unable to bind SMTP capture server on {target_address}: {error}"
            ))
        })?;
        let bound_address = listener.local_addr().map_err(|error| {
            ApplicationError::Internal(format!("Unable to resolve bound SMTP address: {error}"))
        })?;

        let maximum_message_bytes = configuration.maximum_smtp_message_bytes;
        let connection_limiter = Arc::new(Semaphore::new(MAXIMUM_CONCURRENT_SMTP_CONNECTIONS));
        let (shutdown_sender, mut shutdown_receiver) = watch::channel(false);

        let task_handle = tokio::spawn(async move {
            let mut active_connections: JoinSet<()> = JoinSet::new();

            loop {
                tokio::select! {
                    biased;
                    _ = shutdown_receiver.changed() => {
                        if *shutdown_receiver.borrow() {
                            break;
                        }
                    }
                    Some(_) = active_connections.join_next(), if !active_connections.is_empty() => {}
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((mut stream, _peer_address)) => {
                                let Ok(permit) = connection_limiter.clone().try_acquire_owned() else {
                                    let _ = stream
                                        .write_all(b"421 4.3.2 Too many concurrent local SMTP connections\r\n")
                                        .await;
                                    continue;
                                };
                                let session_services = services.clone();
                                let session_publisher = event_publisher.clone();
                                let session_shutdown_receiver = shutdown_receiver.clone();
                                active_connections.spawn(async move {
                                    let _connection_permit = permit;
                                    handle_smtp_connection(
                                        stream,
                                        session_services,
                                        session_publisher,
                                        maximum_message_bytes,
                                        session_shutdown_receiver,
                                    ).await;
                                });
                            }
                            Err(error) => {
                                tracing::warn!("SMTP listener accept error: {error}");
                                break;
                            }
                        }
                    }
                }
            }

            if !active_connections.is_empty() {
                let drain_result = tokio::time::timeout(SMTP_SHUTDOWN_DRAIN_TIMEOUT, async {
                    while active_connections.join_next().await.is_some() {}
                })
                .await;

                if drain_result.is_err() {
                    active_connections.abort_all();
                    while active_connections.join_next().await.is_some() {}
                }
            }
        });

        Ok(Self {
            bound_address,
            shutdown_sender,
            task_handle,
        })
    }

    pub fn bound_address(&self) -> SocketAddr {
        self.bound_address
    }

    pub async fn shutdown(self) {
        let _ = self.shutdown_sender.send(true);
        let _ = self.task_handle.await;
    }
}
