use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::domain::channel::ChannelSummary;
use crate::infrastructure::http::query_routes::{
    HttpNotificationQueryParameters, authenticate_any_channel, build_channel_query,
    wait_for_matching_notification,
};
use crate::infrastructure::http::routes::{HttpCaptureError, HttpCaptureState};

const MCP_PROTOCOL_VERSION: &str = "2024-11-05";

#[derive(Debug, Deserialize)]
pub struct McpJsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: String,
    pub params: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct McpJsonRpcResponse {
    pub jsonrpc: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpJsonRpcError>,
}

#[derive(Debug, Serialize)]
pub struct McpJsonRpcError {
    pub code: i32,
    pub message: String,
}

/// Handles local Model Context Protocol (MCP) JSON-RPC 2.0 requests scoped to the authenticated channel.
pub(super) async fn handle_mcp_json_rpc(
    State(state): State<HttpCaptureState>,
    headers: HeaderMap,
    Json(request): Json<McpJsonRpcRequest>,
) -> Result<(StatusCode, Json<McpJsonRpcResponse>), HttpCaptureError> {
    let channel = authenticate_any_channel(&headers, &state)?;
    let response_id = request.id.clone();

    let outcome = match request.method.as_str() {
        "initialize" => Ok(json!({
            "protocolVersion": MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {}
            },
            "serverInfo": {
                "name": "notifytrap-local-mcp",
                "version": env!("CARGO_PKG_VERSION")
            }
        })),
        "notifications/initialized" => Ok(json!({})),
        "tools/list" => Ok(build_tools_list_result()),
        "tools/call" => execute_tool_call(&state, &channel, request.params).await,
        unknown_method => Err((-32601, format!("Unsupported MCP method: {unknown_method}"))),
    };

    let response = match outcome {
        Ok(result) => McpJsonRpcResponse {
            jsonrpc: "2.0",
            id: response_id,
            result: Some(result),
            error: None,
        },
        Err((code, message)) => McpJsonRpcResponse {
            jsonrpc: "2.0",
            id: response_id,
            result: None,
            error: Some(McpJsonRpcError { code, message }),
        },
    };

    Ok((StatusCode::OK, Json(response)))
}

fn build_tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "list_notifications",
                "description": "List captured notifications for the authenticated NotifyTrap channel. Supports structured search syntax (e.g. 'channel:email otp:true \"reset password\"').",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "search": { "type": "string" },
                        "status": { "type": "string", "enum": ["captured", "read", "archived"] },
                        "run_id": { "type": "string" },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
                    }
                }
            },
            {
                "name": "wait_for_notification",
                "description": "Wait without busy polling for a matching notification to arrive on the authenticated channel.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "run_id": { "type": "string" },
                        "recipient": { "type": "string" },
                        "subject": { "type": "string" },
                        "search": { "type": "string" },
                        "timeout_ms": { "type": "integer", "minimum": 50, "maximum": 30000 }
                    }
                }
            },
            {
                "name": "get_notification_detail",
                "description": "Fetch full captured payload, headers, and verification artifacts for a specific notification ID.",
                "inputSchema": {
                    "type": "object",
                    "required": ["notification_id"],
                    "properties": {
                        "notification_id": { "type": "string" }
                    }
                }
            },
            {
                "name": "extract_otp_codes",
                "description": "List extracted OTP verification codes and magic links from recent matching notifications.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "run_id": { "type": "string" },
                        "search": { "type": "string" },
                        "limit": { "type": "integer", "minimum": 1, "maximum": 50 }
                    }
                }
            }
        ]
    })
}

async fn execute_tool_call(
    state: &HttpCaptureState,
    channel: &ChannelSummary,
    params: Option<Value>,
) -> Result<Value, (i32, String)> {
    let params_object = params.as_ref().and_then(Value::as_object).ok_or((
        -32602,
        "MCP tools/call requires a params object".to_string(),
    ))?;

    let tool_name = params_object
        .get("name")
        .and_then(Value::as_str)
        .ok_or((-32602, "MCP tools/call requires tool 'name'".to_string()))?;

    let arguments = params_object
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match tool_name {
        "list_notifications" => {
            let query_params: HttpNotificationQueryParameters =
                serde_json::from_value(arguments).unwrap_or_default();
            let query = build_channel_query(channel, &query_params).map_err(|error| {
                (
                    -32602,
                    format!("Invalid query parameters: {}", error.message()),
                )
            })?;
            let page = state
                .services
                .notifications
                .query_notifications(query)
                .map_err(|error| (-32000, error.to_string()))?;
            Ok(wrap_mcp_text_result(&page))
        }
        "wait_for_notification" => {
            let query_params: HttpNotificationQueryParameters =
                serde_json::from_value(arguments).unwrap_or_default();
            match wait_for_matching_notification(state, channel, &query_params).await {
                Ok(detail) => Ok(wrap_mcp_text_result(&detail)),
                Err(error) => Ok(wrap_mcp_error_text(error.message())),
            }
        }
        "get_notification_detail" => {
            let notification_id = arguments
                .get("notification_id")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or((
                    -32602,
                    "get_notification_detail requires 'notification_id'".to_string(),
                ))?;
            let detail = state
                .services
                .notifications
                .get_notification_detail(notification_id)
                .map_err(|error| (-32000, error.to_string()))?;
            if detail.notification.channel_id != channel.id {
                return Ok(wrap_mcp_error_text(
                    "Notification not found for this channel",
                ));
            }
            Ok(wrap_mcp_text_result(&detail))
        }
        "extract_otp_codes" => {
            let mut query_params: HttpNotificationQueryParameters =
                serde_json::from_value(arguments).unwrap_or_default();
            if query_params.limit.is_none() {
                query_params.limit = Some(10);
            }
            let query = build_channel_query(channel, &query_params).map_err(|error| {
                (
                    -32602,
                    format!("Invalid query parameters: {}", error.message()),
                )
            })?;
            let page = state
                .services
                .notifications
                .query_notifications(query)
                .map_err(|error| (-32000, error.to_string()))?;

            let artifacts: Vec<Value> = page
                .items
                .into_iter()
                .filter(|item| !item.verification_artifacts.is_empty())
                .map(|item| {
                    json!({
                        "notificationId": item.id,
                        "receivedAt": item.received_at,
                        "subject": item.subject,
                        "recipient": item.recipient,
                        "artifacts": item.verification_artifacts,
                    })
                })
                .collect();
            Ok(wrap_mcp_text_result(&json!({ "matches": artifacts })))
        }
        unknown_tool => Err((-32602, format!("Unknown MCP tool: {unknown_tool}"))),
    }
}

fn wrap_mcp_text_result<T: Serialize>(value: &T) -> Value {
    let serialized = serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".to_string());
    json!({
        "content": [
            {
                "type": "text",
                "text": serialized
            }
        ],
        "isError": false
    })
}

fn wrap_mcp_error_text(message: &str) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": message
            }
        ],
        "isError": true
    })
}
