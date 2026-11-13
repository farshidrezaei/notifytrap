use tauri::State;

use crate::application::application_state::ApplicationState;
use crate::domain::errors::CommandErrorPayload;
use crate::domain::tunnel::{TunnelStartConfiguration, TunnelStatusReport};

/// Retrieves the current connection and public ingress state of the local tunnel.
#[tauri::command]
pub async fn get_tunnel_status(
    state: State<'_, ApplicationState>,
) -> Result<TunnelStatusReport, CommandErrorPayload> {
    Ok(state.tunnel_manager.current_status())
}

/// Initiates an ingress tunnel child process (e.g. cloudflared, ngrok, bore).
#[tauri::command]
pub async fn start_tunnel(
    configuration: TunnelStartConfiguration,
    state: State<'_, ApplicationState>,
) -> Result<TunnelStatusReport, CommandErrorPayload> {
    let http_port = state.current_server_configuration().http_port;
    state
        .tunnel_manager
        .start_tunnel(http_port, configuration)
        .await
        .map_err(|error| error.to_command_payload())
}

/// Stops any actively running ingress tunnel child process.
#[tauri::command]
pub async fn stop_tunnel(
    state: State<'_, ApplicationState>,
) -> Result<TunnelStatusReport, CommandErrorPayload> {
    state
        .tunnel_manager
        .stop_tunnel()
        .await
        .map_err(|error| error.to_command_payload())
}
