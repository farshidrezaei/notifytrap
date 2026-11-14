use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use uuid::Uuid;

use crate::domain::asset_audit::AssetCheckResult;
use crate::domain::channel::ChannelType;
use crate::domain::database_statistics::DatabaseStorageStatistics;
use crate::domain::errors::ApplicationError;
use crate::domain::notification::{
    CaptureAttachmentInput, CaptureNotificationInput, FixtureAttachmentEntry,
    FixtureNotificationEntry, Notification, NotificationAttachment, NotificationAttachmentSummary,
    NotificationDetail, NotificationFixtureBundle, NotificationPage, NotificationQuery,
    NotificationStatus, WebhookReplayInput, WebhookReplayResult,
};
use crate::domain::validation::{
    current_utc_timestamp, validate_optional_test_run_identifier, validate_pagination,
    validate_rfc3339_timestamp, validate_search_query, validate_uuid,
};
use crate::domain::verification_extractor::extract_verification_artifacts;
use crate::infrastructure::database::channel_repository::SqliteChannelRepository;
use crate::infrastructure::database::connection_manager::DatabaseConnectionManager;
use crate::infrastructure::database::notification_repository::SqliteNotificationRepository;
use crate::infrastructure::http::asset_checker::{check_candidate_assets, extract_candidate_urls};
use crate::infrastructure::http::loopback_replay::{
    execute_loopback_http_replay, validate_loopback_target_url,
};

/// Orchestrates notification capture, scoped inbox queries, status transitions, and retention cleanup.
#[derive(Clone)]
pub struct NotificationService {
    database_manager: DatabaseConnectionManager,
}

impl NotificationService {
    pub fn new(database_manager: DatabaseConnectionManager) -> Self {
        Self { database_manager }
    }

    pub fn database_manager(&self) -> &DatabaseConnectionManager {
        &self.database_manager
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
        let test_run_identifier =
            validate_optional_test_run_identifier(input.test_run_identifier.as_deref())?
                .unwrap_or_default();
        let verification_artifacts = extract_verification_artifacts(
            input.channel_type,
            &input.subject,
            &input.preview_text,
            &input.payload_json,
            &input.raw_payload,
        );

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
            test_run_identifier,
            verification_artifacts,
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
        query.test_run_identifier =
            validate_optional_test_run_identifier(query.test_run_identifier.as_deref())?;
        query.recipient_contains = validate_search_query(query.recipient_contains.as_deref())?;
        query.subject_contains = validate_search_query(query.subject_contains.as_deref())?;
        if let Some(since_timestamp) = query
            .since
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            query.since = Some(validate_rfc3339_timestamp(since_timestamp, "since")?);
        } else {
            query.since = None;
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

    pub fn bulk_delete_notifications(
        &self,
        raw_notification_ids: Vec<String>,
    ) -> Result<u64, ApplicationError> {
        let mut validated_ids = Vec::with_capacity(raw_notification_ids.len());
        for raw_id in &raw_notification_ids {
            validated_ids.push(validate_uuid(raw_id, "notificationId")?);
        }
        self.database_manager.with_transaction(|transaction| {
            SqliteNotificationRepository::bulk_delete(transaction, &validated_ids)
        })
    }

    pub fn bulk_update_notification_status(
        &self,
        raw_notification_ids: Vec<String>,
        status: NotificationStatus,
    ) -> Result<u64, ApplicationError> {
        let mut validated_ids = Vec::with_capacity(raw_notification_ids.len());
        for raw_id in &raw_notification_ids {
            validated_ids.push(validate_uuid(raw_id, "notificationId")?);
        }
        self.database_manager.with_transaction(|transaction| {
            SqliteNotificationRepository::bulk_update_status(transaction, &validated_ids, status)
        })
    }

    pub fn bulk_set_notification_tag(
        &self,
        raw_notification_ids: Vec<String>,
        raw_tag: &str,
    ) -> Result<u64, ApplicationError> {
        let tag = validate_optional_test_run_identifier(Some(raw_tag))?.unwrap_or_default();
        let mut validated_ids = Vec::with_capacity(raw_notification_ids.len());
        for raw_id in &raw_notification_ids {
            validated_ids.push(validate_uuid(raw_id, "notificationId")?);
        }
        self.database_manager.with_transaction(|transaction| {
            SqliteNotificationRepository::bulk_set_tag(transaction, &validated_ids, &tag)
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

    /// Replays a captured webhook notification strictly against a local loopback URL.
    pub async fn replay_webhook_notification(
        &self,
        input: WebhookReplayInput,
    ) -> Result<WebhookReplayResult, ApplicationError> {
        let notification_id = validate_uuid(&input.notification_id, "notificationId")?;
        let target = validate_loopback_target_url(&input.target_url)?;
        let detail = self.get_notification_detail(&notification_id)?;

        if detail.notification.channel_type != ChannelType::Webhook {
            return Err(ApplicationError::Validation(
                "Only captured webhook notifications can be replayed".to_string(),
            ));
        }

        let payload = &detail.notification.payload_json;
        let method = payload
            .get("method")
            .and_then(|value| value.as_str())
            .unwrap_or("POST");
        let body_text = payload
            .get("textBody")
            .and_then(|value| value.as_str())
            .unwrap_or(&detail.notification.raw_payload);
        let captured_headers = extract_replay_headers(payload.get("headers"));

        execute_loopback_http_replay(&target, method, &captured_headers, body_text).await
    }

    /// Retrieves filesystem and row footprint statistics for the SQLite database.
    pub fn get_database_storage_statistics(
        &self,
    ) -> Result<DatabaseStorageStatistics, ApplicationError> {
        self.database_manager.get_storage_statistics()
    }

    /// Compiles SQLite database vacuum and returns fresh storage statistics.
    pub fn vacuum_database(&self) -> Result<DatabaseStorageStatistics, ApplicationError> {
        self.database_manager.vacuum_database()?;
        self.database_manager.get_storage_statistics()
    }

    /// Audits all candidate HTTP and HTTPS anchor links and remote images within a notification.
    pub async fn audit_notification_links_and_assets(
        &self,
        notification_id: &str,
    ) -> Result<Vec<AssetCheckResult>, ApplicationError> {
        let validated_id = validate_uuid(notification_id, "notificationId")?;
        let detail = self.get_notification_detail(&validated_id)?;

        let raw_html = detail
            .notification
            .payload_json
            .get("rawHtml")
            .and_then(|value| value.as_str())
            .unwrap_or(&detail.notification.raw_payload);

        let candidates = extract_candidate_urls(raw_html);
        check_candidate_assets(candidates).await
    }

    pub fn export_scope_fixture_bundle(
        &self,
        raw_workspace_id: &str,
        raw_project_id: &str,
        raw_environment_id: &str,
    ) -> Result<NotificationFixtureBundle, ApplicationError> {
        let workspace_id = validate_uuid(raw_workspace_id, "workspaceId")?;
        let project_id = validate_uuid(raw_project_id, "projectId")?;
        let environment_id = validate_uuid(raw_environment_id, "environmentId")?;

        let mut offset = 0;
        let page_size = 200;
        let mut fixture_notifications = Vec::new();

        loop {
            let query = NotificationQuery {
                workspace_id: workspace_id.clone(),
                project_id: project_id.clone(),
                environment_id: environment_id.clone(),
                channel_id: None,
                channel_type: None,
                status: None,
                test_run_identifier: None,
                recipient_contains: None,
                subject_contains: None,
                since: None,
                search_query: None,
                limit: Some(page_size),
                offset: Some(offset),
            };

            let page = self.query_notifications(query)?;
            let fetched_count = page.items.len();
            if fetched_count == 0 {
                break;
            }

            for summary in page.items {
                if let Ok(detail) = self.get_notification_detail(&summary.id) {
                    let mut fixture_attachments = Vec::with_capacity(detail.attachments.len());
                    for attachment in detail.attachments {
                        let content_blob = self
                            .get_attachment(&attachment.id)
                            .map(|att| att.content_blob)
                            .unwrap_or_default();
                        fixture_attachments.push(FixtureAttachmentEntry {
                            filename: attachment.filename,
                            content_type: attachment.content_type,
                            size_bytes: attachment.size_bytes,
                            content_id: attachment.content_id,
                            is_inline: attachment.is_inline,
                            content_base64: STANDARD.encode(&content_blob),
                        });
                    }

                    fixture_notifications.push(FixtureNotificationEntry {
                        channel_type: detail.notification.channel_type,
                        status: detail.notification.status,
                        recipient: detail.notification.recipient,
                        sender: detail.notification.sender,
                        subject: detail.notification.subject,
                        preview_text: detail.notification.preview_text,
                        payload_json: detail.notification.payload_json,
                        raw_payload: detail.notification.raw_payload,
                        test_run_identifier: if detail.notification.test_run_identifier.is_empty() {
                            None
                        } else {
                            Some(detail.notification.test_run_identifier)
                        },
                        received_at: Some(detail.notification.received_at),
                        attachments: fixture_attachments,
                    });
                }
            }

            offset += fetched_count as u32;
            if offset >= page.total_count as u32 {
                break;
            }
        }

        Ok(NotificationFixtureBundle {
            version: "1.0".to_string(),
            generator: "notifytrap".to_string(),
            exported_at: current_utc_timestamp(),
            notifications: fixture_notifications,
        })
    }

    pub fn import_scope_fixture_bundle(
        &self,
        raw_workspace_id: &str,
        raw_project_id: &str,
        raw_environment_id: &str,
        bundle: NotificationFixtureBundle,
    ) -> Result<u64, ApplicationError> {
        let workspace_id = validate_uuid(raw_workspace_id, "workspaceId")?;
        let project_id = validate_uuid(raw_project_id, "projectId")?;
        let environment_id = validate_uuid(raw_environment_id, "environmentId")?;

        let existing_channels = self.database_manager.with_connection(|connection| {
            SqliteChannelRepository::list_by_scope(connection, &workspace_id, &project_id, &environment_id)
        })?;

        if existing_channels.is_empty() {
            return Err(ApplicationError::Validation(
                "Cannot import fixture bundle into an environment with no channels configured".to_string(),
            ));
        }

        let mut imported_count: u64 = 0;
        for entry in bundle.notifications {
            let matched_channel = existing_channels
                .iter()
                .find(|ch| ch.channel_type == entry.channel_type)
                .unwrap_or(&existing_channels[0]);

            let mut attachments = Vec::with_capacity(entry.attachments.len());
            for att in entry.attachments {
                let content_blob = STANDARD.decode(&att.content_base64).unwrap_or_default();
                attachments.push(CaptureAttachmentInput {
                    filename: att.filename,
                    content_type: att.content_type,
                    content_id: att.content_id,
                    is_inline: att.is_inline,
                    content_blob,
                });
            }

            self.capture_notification(CaptureNotificationInput {
                workspace_id: workspace_id.clone(),
                project_id: project_id.clone(),
                environment_id: environment_id.clone(),
                channel_id: matched_channel.id.clone(),
                channel_type: entry.channel_type,
                recipient: entry.recipient,
                sender: entry.sender,
                subject: entry.subject,
                preview_text: entry.preview_text,
                payload_json: entry.payload_json,
                raw_payload: entry.raw_payload,
                test_run_identifier: entry.test_run_identifier,
                received_at: entry.received_at,
                attachments,
            })?;
            imported_count += 1;
        }

        Ok(imported_count)
    }
}

fn extract_replay_headers(raw_headers: Option<&serde_json::Value>) -> Vec<(String, String)> {
    let mut headers = Vec::new();
    let Some(map) = raw_headers.and_then(|value| value.as_object()) else {
        return headers;
    };
    for (name, value) in map {
        let text_value = match value {
            serde_json::Value::String(text) => text.clone(),
            other => other.to_string(),
        };
        headers.push((name.clone(), text_value));
    }
    headers
}
