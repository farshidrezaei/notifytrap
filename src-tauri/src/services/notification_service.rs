use uuid::Uuid;

use crate::domain::errors::ApplicationError;
use crate::domain::notification::{
    CaptureNotificationInput, Notification, NotificationAttachment, NotificationAttachmentSummary,
    NotificationDetail, NotificationPage, NotificationQuery, NotificationStatus,
};
use crate::domain::validation::{
    current_utc_timestamp, validate_pagination, validate_rfc3339_timestamp, validate_search_query,
    validate_uuid,
};
use crate::infrastructure::database::channel_repository::SqliteChannelRepository;
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::database::notification_repository::SqliteNotificationRepository;

/// Orchestrates notification capture, scoped inbox queries, status transitions, and retention cleanup.
#[derive(Clone)]
pub struct NotificationService {
    database_manager: DatabaseConnectionManager,
}

impl NotificationService {
    pub fn new(database_manager: DatabaseConnectionManager) -> Self {
        Self { database_manager }
    }

    /// Persists a normalized notification and any MIME attachments inside one ACID transaction.
    pub fn capture_notification(
        &self,
        input: CaptureNotificationInput,
    ) -> Result<NotificationDetail, ApplicationError> {
        let workspace_id = validate_uuid(&input.workspace_id, "workspaceId")?;
        let project_id = validate_uuid(&input.project_id, "projectId")?;
        let environment_id = validate_uuid(&input.environment_id, "environmentId")?;
        let channel_id = validate_uuid(&input.channel_id, "channelId")?;
        let now = current_utc_timestamp();
        let received_at = input
            .received_at
            .filter(|timestamp| !timestamp.trim().is_empty())
            .unwrap_or_else(|| now.clone());

        let notification = Notification {
            id: Uuid::new_v4().to_string(),
            workspace_id: workspace_id.clone(),
            project_id: project_id.clone(),
            environment_id: environment_id.clone(),
            channel_id: channel_id.clone(),
            channel_type: input.channel_type,
            status: NotificationStatus::Captured,
            recipient: input.recipient,
            sender: input.sender,
            subject: input.subject,
            preview_text: input.preview_text,
            payload_json: input.payload_json,
            raw_payload: input.raw_payload,
            received_at,
            created_at: now.clone(),
        };

        let attachments: Vec<NotificationAttachment> = input
            .attachments
            .into_iter()
            .map(|attachment_input| {
                let size_bytes =
                    i64::try_from(attachment_input.content_blob.len()).unwrap_or(i64::MAX);
                NotificationAttachment {
                    summary: NotificationAttachmentSummary {
                        id: Uuid::new_v4().to_string(),
                        notification_id: notification.id.clone(),
                        filename: attachment_input.filename,
                        content_type: attachment_input.content_type,
                        size_bytes,
                        content_id: attachment_input.content_id,
                        is_inline: attachment_input.is_inline,
                        created_at: now.clone(),
                    },
                    content_blob: attachment_input.content_blob,
                }
            })
            .collect();

        let attachment_summaries: Vec<NotificationAttachmentSummary> = attachments
            .iter()
            .map(|attachment| attachment.summary.clone())
            .collect();

        self.database_manager.with_transaction(|transaction| {
            let channel = SqliteChannelRepository::find_by_id(transaction, &channel_id)?
                .ok_or_else(|| ApplicationError::NotFound("Channel".to_string()))?;

            if channel.workspace_id != workspace_id
                || channel.project_id != project_id
                || channel.environment_id != environment_id
            {
                return Err(ApplicationError::Validation(
                    "Notification scope does not match target channel ownership".to_string(),
                ));
            }

            if !channel.is_enabled {
                return Err(ApplicationError::Conflict(
                    "Target channel is currently disabled".to_string(),
                ));
            }

            SqliteNotificationRepository::insert_notification(transaction, &notification)?;
            for attachment in &attachments {
                SqliteNotificationRepository::insert_attachment(transaction, attachment)?;
            }

            Ok(NotificationDetail {
                notification,
                attachments: attachment_summaries,
            })
        })
    }

    pub fn query_notifications(
        &self,
        mut query: NotificationQuery,
    ) -> Result<NotificationPage, ApplicationError> {
        query.workspace_id = validate_uuid(&query.workspace_id, "workspaceId")?;
        query.project_id = validate_uuid(&query.project_id, "projectId")?;
        query.environment_id = validate_uuid(&query.environment_id, "environmentId")?;
        if let Some(channel_id) = &query.channel_id {
            if !channel_id.trim().is_empty() {
                query.channel_id = Some(validate_uuid(channel_id, "channelId")?);
            } else {
                query.channel_id = None;
            }
        }
        query.search_query = validate_search_query(query.search_query.as_deref())?;
        let (limit, offset) = validate_pagination(query.limit, query.offset)?;
        query.limit = Some(limit);
        query.offset = Some(offset);

        self.database_manager.with_connection(|connection| {
            SqliteNotificationRepository::query_paginated(connection, &query)
        })
    }

    pub fn get_notification_detail(
        &self,
        raw_notification_id: &str,
    ) -> Result<NotificationDetail, ApplicationError> {
        let notification_id = validate_uuid(raw_notification_id, "notificationId")?;
        self.database_manager.with_connection(|connection| {
            let notification =
                SqliteNotificationRepository::find_by_id(connection, &notification_id)?
                    .ok_or_else(|| ApplicationError::NotFound("Notification".to_string()))?;
            let attachments =
                SqliteNotificationRepository::list_attachments(connection, &notification_id)?;
            Ok(NotificationDetail {
                notification,
                attachments,
            })
        })
    }

    pub fn get_attachment(
        &self,
        raw_attachment_id: &str,
    ) -> Result<NotificationAttachment, ApplicationError> {
        let attachment_id = validate_uuid(raw_attachment_id, "attachmentId")?;
        self.database_manager.with_connection(|connection| {
            SqliteNotificationRepository::find_attachment_blob(connection, &attachment_id)?
                .ok_or_else(|| ApplicationError::NotFound("NotificationAttachment".to_string()))
        })
    }

    pub fn update_notification_status(
        &self,
        raw_notification_id: &str,
        status: NotificationStatus,
    ) -> Result<Notification, ApplicationError> {
        let notification_id = validate_uuid(raw_notification_id, "notificationId")?;
        self.database_manager.with_connection(|connection| {
            let did_update =
                SqliteNotificationRepository::update_status(connection, &notification_id, status)?;
            if !did_update {
                return Err(ApplicationError::NotFound("Notification".to_string()));
            }
            SqliteNotificationRepository::find_by_id(connection, &notification_id)?
                .ok_or_else(|| ApplicationError::NotFound("Notification".to_string()))
        })
    }

    pub fn delete_notification(&self, raw_notification_id: &str) -> Result<(), ApplicationError> {
        let notification_id = validate_uuid(raw_notification_id, "notificationId")?;
        self.database_manager.with_connection(|connection| {
            let did_delete =
                SqliteNotificationRepository::delete_by_id(connection, &notification_id)?;
            if !did_delete {
                return Err(ApplicationError::NotFound("Notification".to_string()));
            }
            Ok(())
        })
    }

    pub fn clear_scope_notifications(
        &self,
        raw_workspace_id: &str,
        raw_project_id: &str,
        raw_environment_id: &str,
    ) -> Result<u64, ApplicationError> {
        let workspace_id = validate_uuid(raw_workspace_id, "workspaceId")?;
        let project_id = validate_uuid(raw_project_id, "projectId")?;
        let environment_id = validate_uuid(raw_environment_id, "environmentId")?;

        self.database_manager.with_connection(|connection| {
            SqliteNotificationRepository::clear_scope(
                connection,
                &workspace_id,
                &project_id,
                &environment_id,
            )
        })
    }

    pub fn prune_notifications_older_than(
        &self,
        cutoff_timestamp: &str,
    ) -> Result<u64, ApplicationError> {
        let validated_timestamp = validate_rfc3339_timestamp(cutoff_timestamp, "cutoffTimestamp")?;

        self.database_manager.with_connection(|connection| {
            SqliteNotificationRepository::delete_older_than(connection, &validated_timestamp)
        })
    }

    pub fn prune_expired_by_retention_days(
        &self,
        retention_days: u64,
    ) -> Result<u64, ApplicationError> {
        let validated_days = crate::domain::validation::validate_retention_days(retention_days)?;
        let days_i64 = i64::try_from(validated_days).unwrap_or(i64::MAX);
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days_i64);
        let cutoff_timestamp = cutoff.to_rfc3339();
        self.prune_notifications_older_than(&cutoff_timestamp)
    }
}
