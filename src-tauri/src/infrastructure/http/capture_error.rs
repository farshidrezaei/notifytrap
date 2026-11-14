use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

use crate::domain::errors::ApplicationError;

#[derive(Debug, Serialize)]
pub struct HttpErrorBody {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureReceiptResponse {
    pub notification_id: String,
    pub channel_id: String,
    pub status: String,
    pub received_at: String,
}

pub struct HttpCaptureError {
    status_code: StatusCode,
    code: &'static str,
    message: String,
}

impl HttpCaptureError {
    pub fn from_application_error(error: ApplicationError) -> Self {
        match error {
            ApplicationError::Unauthorized => Self {
                status_code: StatusCode::UNAUTHORIZED,
                code: "UNAUTHORIZED",
                message: "Channel authentication failed".to_string(),
            },
            ApplicationError::Validation(message) => Self {
                status_code: StatusCode::BAD_REQUEST,
                code: "VALIDATION_ERROR",
                message,
            },
            ApplicationError::NotFound(message) => Self {
                status_code: StatusCode::NOT_FOUND,
                code: "NOT_FOUND",
                message,
            },
            ApplicationError::Conflict(message) => Self {
                status_code: StatusCode::CONFLICT,
                code: "CONFLICT",
                message,
            },
            other => {
                tracing::error!("HTTP capture internal failure: {other}");
                Self {
                    status_code: StatusCode::INTERNAL_SERVER_ERROR,
                    code: "INTERNAL_ERROR",
                    message: "Internal capture storage error".to_string(),
                }
            }
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::BAD_REQUEST,
            code: "VALIDATION_ERROR",
            message: message.into(),
        }
    }

    pub fn from_json_rejection(rejection: axum::extract::rejection::JsonRejection) -> Self {
        let status_code = rejection.status();
        let code = if status_code == StatusCode::PAYLOAD_TOO_LARGE {
            "PAYLOAD_TOO_LARGE"
        } else {
            "VALIDATION_ERROR"
        };
        Self {
            status_code,
            code,
            message: rejection.body_text(),
        }
    }

    pub fn unauthorized() -> Self {
        Self {
            status_code: StatusCode::UNAUTHORIZED,
            code: "UNAUTHORIZED",
            message: "Channel authentication failed".to_string(),
        }
    }

    pub fn not_found(entity: &str) -> Self {
        Self {
            status_code: StatusCode::NOT_FOUND,
            code: "NOT_FOUND",
            message: format!("{entity} not found"),
        }
    }

    pub fn request_timeout(message: impl Into<String>) -> Self {
        Self {
            status_code: StatusCode::REQUEST_TIMEOUT,
            code: "WAIT_TIMEOUT",
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn chaos_failure(status_code: StatusCode, message: String) -> Self {
        Self {
            status_code,
            code: "CHAOS_SIMULATED_FAILURE",
            message,
        }
    }
}

impl IntoResponse for HttpCaptureError {
    fn into_response(self) -> Response {
        (
            self.status_code,
            Json(HttpErrorBody {
                code: self.code.to_string(),
                message: self.message,
            }),
        )
            .into_response()
    }
}
