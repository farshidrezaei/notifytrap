CREATE TABLE tenants (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE,
    slug TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE projects (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (tenant_id, slug)
);

CREATE INDEX idx_projects_tenant_id ON projects(tenant_id);

CREATE TABLE environments (
    id TEXT PRIMARY KEY NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('local', 'development', 'testing', 'staging')),
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE (project_id, slug)
);

CREATE INDEX idx_environments_project_id ON environments(project_id);

CREATE TABLE channels (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    environment_id TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    channel_type TEXT NOT NULL CHECK (channel_type IN ('email', 'sms', 'push', 'webhook')),
    is_enabled INTEGER NOT NULL DEFAULT 1 CHECK (is_enabled IN (0, 1)),
    credential_identifier TEXT NOT NULL UNIQUE,
    credential_hash TEXT NOT NULL,
    encrypted_credential TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_channels_scope ON channels(tenant_id, project_id, environment_id);
CREATE INDEX idx_channels_credential_identifier ON channels(credential_identifier);

CREATE TABLE notifications (
    id TEXT PRIMARY KEY NOT NULL,
    tenant_id TEXT NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    environment_id TEXT NOT NULL REFERENCES environments(id) ON DELETE CASCADE,
    channel_id TEXT NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    channel_type TEXT NOT NULL CHECK (channel_type IN ('email', 'sms', 'push', 'webhook')),
    status TEXT NOT NULL DEFAULT 'captured' CHECK (status IN ('captured', 'read', 'archived')),
    recipient TEXT NOT NULL DEFAULT '',
    sender TEXT NOT NULL DEFAULT '',
    subject TEXT NOT NULL DEFAULT '',
    preview_text TEXT NOT NULL DEFAULT '',
    payload_json TEXT NOT NULL,
    raw_payload TEXT NOT NULL DEFAULT '',
    received_at TEXT NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_notifications_inbox ON notifications(tenant_id, project_id, environment_id, received_at DESC);
CREATE INDEX idx_notifications_channel ON notifications(channel_id, received_at DESC);
CREATE INDEX idx_notifications_type ON notifications(channel_type, received_at DESC);
CREATE INDEX idx_notifications_status ON notifications(status);

CREATE TABLE notification_attachments (
    id TEXT PRIMARY KEY NOT NULL,
    notification_id TEXT NOT NULL REFERENCES notifications(id) ON DELETE CASCADE,
    filename TEXT NOT NULL,
    content_type TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    content_id TEXT,
    is_inline INTEGER NOT NULL DEFAULT 0 CHECK (is_inline IN (0, 1)),
    content_blob BLOB NOT NULL,
    created_at TEXT NOT NULL
);

CREATE INDEX idx_notification_attachments_notification_id ON notification_attachments(notification_id);

CREATE TABLE application_settings (
    key TEXT PRIMARY KEY NOT NULL,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
