use std::process::Stdio;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use regex::Regex;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{Mutex as TokioMutex, oneshot};

use crate::domain::errors::ApplicationError;
use crate::domain::tunnel::{
    TunnelProvider, TunnelStartConfiguration, TunnelStatus, TunnelStatusReport,
};
use crate::domain::validation::current_utc_timestamp;
use crate::events::EventPublisher;

struct ActiveProcess {
    child: Child,
    shutdown_sender: Option<oneshot::Sender<()>>,
}

/// Orchestrates background execution of ingress tunnel child processes (cloudflared, ngrok, bore, custom).
#[derive(Clone)]
pub struct TunnelManager {
    report_state: Arc<RwLock<TunnelStatusReport>>,
    process_handle: Arc<TokioMutex<Option<ActiveProcess>>>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl TunnelManager {
    pub fn new(event_publisher: Arc<dyn EventPublisher>) -> Self {
        Self {
            report_state: Arc::new(RwLock::new(TunnelStatusReport::default())),
            process_handle: Arc::new(TokioMutex::new(None)),
            event_publisher,
        }
    }

    pub fn current_status(&self) -> TunnelStatusReport {
        self.report_state
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    pub async fn stop_tunnel(&self) -> Result<TunnelStatusReport, ApplicationError> {
        let mut handle_guard = self.process_handle.lock().await;
        if let Some(mut active) = handle_guard.take() {
            if let Some(sender) = active.shutdown_sender.take() {
                let _ = sender.send(());
            }
            let _ = active.child.kill().await;
        }

        let updated_report = TunnelStatusReport {
            is_active: false,
            status: TunnelStatus::Disconnected,
            provider: self.current_status().provider,
            public_url: None,
            local_target: self.current_status().local_target,
            error_message: None,
            started_at: None,
        };

        if let Ok(mut guard) = self.report_state.write() {
            *guard = updated_report.clone();
        }
        self.event_publisher
            .emit_tunnel_status_changed(&updated_report);
        Ok(updated_report)
    }

    pub async fn start_tunnel(
        &self,
        local_http_port: u16,
        configuration: TunnelStartConfiguration,
    ) -> Result<TunnelStatusReport, ApplicationError> {
        // Stop any running tunnel instance first
        self.stop_tunnel().await?;

        let local_target = format!("http://127.0.0.1:{local_http_port}");
        let initial_report = TunnelStatusReport {
            is_active: true,
            status: TunnelStatus::Starting,
            provider: configuration.provider,
            public_url: None,
            local_target: local_target.clone(),
            error_message: None,
            started_at: Some(current_utc_timestamp()),
        };

        if let Ok(mut guard) = self.report_state.write() {
            *guard = initial_report.clone();
        }
        self.event_publisher
            .emit_tunnel_status_changed(&initial_report);

        let (program, arguments) = match configuration.provider {
            TunnelProvider::Cloudflared => {
                let bin = configuration
                    .custom_binary_path
                    .filter(|p| !p.trim().is_empty())
                    .unwrap_or_else(|| "cloudflared".to_string());
                let args = vec![
                    "tunnel".to_string(),
                    "--url".to_string(),
                    local_target.clone(),
                    "--no-autoupdate".to_string(),
                ];
                (bin, args)
            }
            TunnelProvider::Ngrok => {
                let bin = configuration
                    .custom_binary_path
                    .filter(|p| !p.trim().is_empty())
                    .unwrap_or_else(|| "ngrok".to_string());
                let mut args = vec!["http".to_string(), local_http_port.to_string()];
                if let Some(token) = configuration
                    .authentication_token
                    .filter(|t| !t.trim().is_empty())
                {
                    args.push("--authtoken".to_string());
                    args.push(token);
                }
                (bin, args)
            }
            TunnelProvider::Bore => {
                let bin = configuration
                    .custom_binary_path
                    .filter(|p| !p.trim().is_empty())
                    .unwrap_or_else(|| "bore".to_string());
                let args = vec![
                    "local".to_string(),
                    local_http_port.to_string(),
                    "--to".to_string(),
                    "bore.pub".to_string(),
                ];
                (bin, args)
            }
            TunnelProvider::Custom => {
                let bin = configuration.custom_binary_path.ok_or_else(|| {
                    ApplicationError::Validation(
                        "Custom binary path is required for custom provider".to_string(),
                    )
                })?;
                let custom_args = configuration
                    .custom_arguments
                    .unwrap_or_default()
                    .split_whitespace()
                    .map(|item| item.replace("{port}", &local_http_port.to_string()))
                    .collect::<Vec<_>>();
                (bin, custom_args)
            }
        };

        let mut command = Command::new(&program);
        command
            .args(&arguments)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(spawn_error) => {
                let failed_report = TunnelStatusReport {
                    is_active: false,
                    status: TunnelStatus::Failed,
                    provider: configuration.provider,
                    public_url: None,
                    local_target,
                    error_message: Some(format!("Failed to execute '{program}': {spawn_error}")),
                    started_at: None,
                };
                if let Ok(mut guard) = self.report_state.write() {
                    *guard = failed_report.clone();
                }
                self.event_publisher
                    .emit_tunnel_status_changed(&failed_report);
                return Err(ApplicationError::Internal(format!(
                    "Failed to spawn tunnel process '{program}': {spawn_error}"
                )));
            }
        };

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

        {
            let mut handle_guard = self.process_handle.lock().await;
            *handle_guard = Some(ActiveProcess {
                child,
                shutdown_sender: Some(shutdown_tx),
            });
        }

        let report_state_clone = Arc::clone(&self.report_state);
        let event_publisher_clone = Arc::clone(&self.event_publisher);
        let provider = configuration.provider;
        let target_clone = local_target.clone();

        tokio::spawn(async move {
            let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(128);

            if let Some(out) = stdout {
                let tx_out = tx.clone();
                tokio::spawn(async move {
                    let mut reader = BufReader::new(out).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let _ = tx_out.send(line).await;
                    }
                });
            }

            if let Some(err) = stderr {
                let tx_err = tx.clone();
                tokio::spawn(async move {
                    let mut reader = BufReader::new(err).lines();
                    while let Ok(Some(line)) = reader.next_line().await {
                        let _ = tx_err.send(line).await;
                    }
                });
            }
            drop(tx);

            let url_regex = Regex::new(r"https://[a-zA-Z0-9_\-\.]+\.(trycloudflare\.com|ngrok-free\.app|ngrok\.io|bore\.pub)").unwrap();

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        break;
                    }
                    maybe_line = rx.recv() => {
                        match maybe_line {
                            Some(line) => {
                                if let Some(matched) = url_regex.find(&line) {
                                    let detected_url = matched.as_str().to_string();
                                    let connected_report = TunnelStatusReport {
                                        is_active: true,
                                        status: TunnelStatus::Connected,
                                        provider,
                                        public_url: Some(detected_url),
                                        local_target: target_clone.clone(),
                                        error_message: None,
                                        started_at: Some(current_utc_timestamp()),
                                    };
                                    if let Ok(mut guard) = report_state_clone.write() {
                                        *guard = connected_report.clone();
                                    }
                                    event_publisher_clone.emit_tunnel_status_changed(&connected_report);
                                }
                            }
                            None => {
                                break;
                            }
                        }
                    }
                }
            }
        });

        // Wait brief period for immediate startup outcome
        tokio::time::sleep(Duration::from_millis(500)).await;
        Ok(self.current_status())
    }
}
