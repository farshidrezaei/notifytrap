use rusqlite::types::ToSql;
use rusqlite::{Connection, Row, params, params_from_iter};

use crate::domain::channel::ChannelType;
use crate::domain::errors::ApplicationError;
use crate::domain::notification::{
    Notification, NotificationAttachment, NotificationAttachmentSummary, NotificationPage,
    NotificationQuery, NotificationStatus, NotificationSummary,
};
use crate::domain::validation::{DEFAULT_PAGE_SIZE, MAXIMUM_PAGE_SIZE};
use crate::domain::verification_extractor::extract_verification_artifacts;
use crate::infrastructure::database::sqlite_helpers::query_optional_row;

/// SQLite persistence operations for captured notifications and attachments.
pub struct SqliteNotificationRepository;

impl SqliteNotificationRepository {
    pub fn insert_notification(
        connection: &Connection,
        notification: &Notification,
    ) -> Result<(), ApplicationError> {
        let payload_text = serde_json::to_string(&notification.payload_json).map_err(|error| {
            ApplicationError::Internal(format!("Failed to serialize notification payload: {error}"))
        })?;

        connection.execute(
            "INSERT INTO notifications (
                id, workspace_id, project_id, environment_id, channel_id, channel_type, status,
                recipient, sender, subject, preview_text, payload_json, raw_payload,
                test_run_identifier, received_at, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            params![
                notification.id,
                notification.workspace_id,
                notification.project_id,
                notification.environment_id,
                notification.channel_id,
                notification.channel_type.as_str(),
                notification.status.as_str(),
                notification.recipient,
                notification.sender,
                notification.subject,
                notification.preview_text,
                payload_text,
                notification.raw_payload,
                notification.test_run_identifier,
                notification.received_at,
                notification.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn insert_attachment(
        connection: &Connection,
        attachment: &NotificationAttachment,
    ) -> Result<(), ApplicationError> {
        let inline_flag: i64 = if attachment.summary.is_inline { 1 } else { 0 };
        connection.execute(
            "INSERT INTO notification_attachments (
                id, notification_id, filename, content_type, size_bytes,
                content_id, is_inline, content_blob, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                attachment.summary.id,
                attachment.summary.notification_id,
                attachment.summary.filename,
                attachment.summary.content_type,
                attachment.summary.size_bytes,
                attachment.summary.content_id,
                inline_flag,
                attachment.content_blob,
                attachment.summary.created_at,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_id(
        connection: &Connection,
        notification_id: &str,
    ) -> Result<Option<Notification>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, workspace_id, project_id, environment_id, channel_id, channel_type, status,
                    recipient, sender, subject, preview_text, payload_json, raw_payload,
                    test_run_identifier, received_at, created_at
             FROM notifications
             WHERE id = ?1",
        )?;
        query_optional_row(
            &mut statement,
            params![notification_id],
            map_notification_row,
        )
    }

    pub fn list_attachments(
        connection: &Connection,
        notification_id: &str,
    ) -> Result<Vec<NotificationAttachmentSummary>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, notification_id, filename, content_type, size_bytes,
                    content_id, is_inline, created_at
             FROM notification_attachments
             WHERE notification_id = ?1
             ORDER BY is_inline DESC, filename ASC",
        )?;
        let rows = statement.query_map(params![notification_id], map_attachment_summary_row)?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(ApplicationError::from)
    }

    pub fn find_attachment_blob(
        connection: &Connection,
        attachment_id: &str,
    ) -> Result<Option<NotificationAttachment>, ApplicationError> {
        let mut statement = connection.prepare(
            "SELECT id, notification_id, filename, content_type, size_bytes,
                    content_id, is_inline, created_at, content_blob
             FROM notification_attachments
             WHERE id = ?1",
        )?;
        query_optional_row(&mut statement, params![attachment_id], |row| {
            let summary = map_attachment_summary_row(row)?;
            let content_blob: Vec<u8> = row.get(8)?;
            Ok(NotificationAttachment {
                summary,
                content_blob,
            })
        })
    }

    pub fn query_paginated(
        connection: &Connection,
        query: &NotificationQuery,
    ) -> Result<NotificationPage, ApplicationError> {
        let limit = query
            .limit
            .unwrap_or(DEFAULT_PAGE_SIZE)
            .clamp(1, MAXIMUM_PAGE_SIZE);
        let offset = query.offset.unwrap_or(0);

        let (where_clause, filter_parameters) = build_scope_where_clause(query);

        let count_sql = format!("SELECT COUNT(*) FROM notifications WHERE {where_clause}");
        let parameter_refs: Vec<&dyn ToSql> = filter_parameters
            .iter()
            .map(|value| value.as_ref())
            .collect();
        let total_count_i64: i64 =
            connection.query_row(&count_sql, params_from_iter(parameter_refs), |row| {
                row.get(0)
            })?;

        let list_sql = format!(
            "SELECT id, workspace_id, project_id, environment_id, channel_id, channel_type, status,
                    recipient, sender, subject, preview_text, payload_json, raw_payload,
                    test_run_identifier, received_at
             FROM notifications
             WHERE {where_clause}
             ORDER BY received_at DESC, id DESC
             LIMIT ?{} OFFSET ?{}",
            filter_parameters.len() + 1,
            filter_parameters.len() + 2
        );

        let mut list_parameters = filter_parameters;
        list_parameters.push(Box::new(i64::from(limit)));
        list_parameters.push(Box::new(i64::from(offset)));
        let list_refs: Vec<&dyn ToSql> =
            list_parameters.iter().map(|value| value.as_ref()).collect();

        let mut statement = connection.prepare(&list_sql)?;
        let rows =
            statement.query_map(params_from_iter(list_refs), map_notification_summary_row)?;
        let items = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(ApplicationError::from)?;

        Ok(NotificationPage {
            items,
            total_count: total_count_i64.max(0) as u64,
            limit,
            offset,
        })
    }

    pub fn update_status(
        connection: &Connection,
        notification_id: &str,
        status: NotificationStatus,
    ) -> Result<bool, ApplicationError> {
        let affected = connection.execute(
            "UPDATE notifications SET status = ?2 WHERE id = ?1",
            params![notification_id, status.as_str()],
        )?;
        Ok(affected > 0)
    }

    pub fn delete_by_id(
        connection: &Connection,
        notification_id: &str,
    ) -> Result<bool, ApplicationError> {
        let affected = connection.execute(
            "DELETE FROM notifications WHERE id = ?1",
            params![notification_id],
        )?;
        Ok(affected > 0)
    }

    pub fn clear_scope(
        connection: &Connection,
        workspace_id: &str,
        project_id: &str,
        environment_id: &str,
    ) -> Result<u64, ApplicationError> {
        let affected = connection.execute(
            "DELETE FROM notifications
             WHERE workspace_id = ?1 AND project_id = ?2 AND environment_id = ?3",
            params![workspace_id, project_id, environment_id],
        )?;
        Ok(affected as u64)
    }

    pub fn delete_older_than(
        connection: &Connection,
        cutoff_timestamp: &str,
    ) -> Result<u64, ApplicationError> {
        let affected = connection.execute(
            "DELETE FROM notifications WHERE received_at < ?1",
            params![cutoff_timestamp],
        )?;
        Ok(affected as u64)
    }
}

fn build_scope_where_clause(query: &NotificationQuery) -> (String, Vec<Box<dyn ToSql>>) {
    let mut clauses = vec![
        "workspace_id = ?1".to_string(),
        "project_id = ?2".to_string(),
        "environment_id = ?3".to_string(),
    ];
    let mut parameters: Vec<Box<dyn ToSql>> = vec![
        Box::new(query.workspace_id.clone()),
        Box::new(query.project_id.clone()),
        Box::new(query.environment_id.clone()),
    ];

    if let Some(channel_id) = query
        .channel_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parameters.push(Box::new(channel_id.to_string()));
        clauses.push(format!("channel_id = ?{}", parameters.len()));
    }

    if let Some(channel_type) = query.channel_type {
        parameters.push(Box::new(channel_type.as_str().to_string()));
        clauses.push(format!("channel_type = ?{}", parameters.len()));
    }

    if let Some(status) = query.status {
        parameters.push(Box::new(status.as_str().to_string()));
        clauses.push(format!("status = ?{}", parameters.len()));
    }

    if let Some(test_run_identifier) = query
        .test_run_identifier
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parameters.push(Box::new(test_run_identifier.to_string()));
        clauses.push(format!("test_run_identifier = ?{}", parameters.len()));
    }

    if let Some(recipient_filter) = query
        .recipient_contains
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{}%", escape_sql_like(recipient_filter));
        parameters.push(Box::new(pattern));
        clauses.push(format!("recipient LIKE ?{} ESCAPE '\\'", parameters.len()));
    }

    if let Some(subject_filter) = query
        .subject_contains
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let pattern = format!("%{}%", escape_sql_like(subject_filter));
        parameters.push(Box::new(pattern));
        clauses.push(format!("subject LIKE ?{} ESCAPE '\\'", parameters.len()));
    }

    if let Some(since_timestamp) = query
        .since
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        parameters.push(Box::new(since_timestamp.to_string()));
        clauses.push(format!("received_at >= ?{}", parameters.len()));
    }

    if let Some(search) = query
        .search_query
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let escaped = escape_sql_like(search);
        let pattern = format!("%{escaped}%");
        parameters.push(Box::new(pattern));
        let index = parameters.len();
        clauses.push(format!(
            "(recipient LIKE ?{index} ESCAPE '\\' OR sender LIKE ?{index} ESCAPE '\\' OR subject LIKE ?{index} ESCAPE '\\' OR preview_text LIKE ?{index} ESCAPE '\\' OR test_run_identifier LIKE ?{index} ESCAPE '\\')"
        ));
    }

    (clauses.join(" AND "), parameters)
}

fn escape_sql_like(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for character in input.chars() {
        if matches!(character, '\\' | '%' | '_') {
            escaped.push('\\');
        }
        escaped.push(character);
    }
    escaped
}

fn map_notification_row(row: &Row<'_>) -> rusqlite::Result<Notification> {
    let raw_type: String = row.get(5)?;
    let channel_type = ChannelType::parse(&raw_type).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let raw_status: String = row.get(6)?;
    let status = NotificationStatus::parse(&raw_status).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let raw_json: String = row.get(11)?;
    let payload_json: serde_json::Value = serde_json::from_str(&raw_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(11, rusqlite::types::Type::Text, Box::new(error))
    })?;

    let subject: String = row.get(9)?;
    let preview_text: String = row.get(10)?;
    let raw_payload: String = row.get(12)?;
    let verification_artifacts = extract_verification_artifacts(
        channel_type,
        &subject,
        &preview_text,
        &payload_json,
        &raw_payload,
    );

    Ok(Notification {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        project_id: row.get(2)?,
        environment_id: row.get(3)?,
        channel_id: row.get(4)?,
        channel_type,
        status,
        recipient: row.get(7)?,
        sender: row.get(8)?,
        subject,
        preview_text,
        payload_json,
        raw_payload,
        test_run_identifier: row.get(13)?,
        verification_artifacts,
        received_at: row.get(14)?,
        created_at: row.get(15)?,
    })
}

fn map_notification_summary_row(row: &Row<'_>) -> rusqlite::Result<NotificationSummary> {
    let raw_type: String = row.get(5)?;
    let channel_type = ChannelType::parse(&raw_type).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(5, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let raw_status: String = row.get(6)?;
    let status = NotificationStatus::parse(&raw_status).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(6, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let raw_json: String = row.get(11)?;
    let payload_json: serde_json::Value = serde_json::from_str(&raw_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(11, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let subject: String = row.get(9)?;
    let preview_text: String = row.get(10)?;
    let raw_payload: String = row.get(12)?;
    let verification_artifacts = extract_verification_artifacts(
        channel_type,
        &subject,
        &preview_text,
        &payload_json,
        &raw_payload,
    );

    Ok(NotificationSummary {
        id: row.get(0)?,
        workspace_id: row.get(1)?,
        project_id: row.get(2)?,
        environment_id: row.get(3)?,
        channel_id: row.get(4)?,
        channel_type,
        status,
        recipient: row.get(7)?,
        sender: row.get(8)?,
        subject,
        preview_text,
        test_run_identifier: row.get(13)?,
        verification_artifacts,
        received_at: row.get(14)?,
    })
}

fn map_attachment_summary_row(row: &Row<'_>) -> rusqlite::Result<NotificationAttachmentSummary> {
    let is_inline_flag: i64 = row.get(6)?;
    Ok(NotificationAttachmentSummary {
        id: row.get(0)?,
        notification_id: row.get(1)?,
        filename: row.get(2)?,
        content_type: row.get(3)?,
        size_bytes: row.get(4)?,
        content_id: row.get(5)?,
        is_inline: is_inline_flag == 1,
        created_at: row.get(7)?,
    })
}
