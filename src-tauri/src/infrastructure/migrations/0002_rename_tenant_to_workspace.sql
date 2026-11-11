ALTER TABLE tenants RENAME TO workspaces;

ALTER TABLE projects RENAME COLUMN tenant_id TO workspace_id;
DROP INDEX IF EXISTS idx_projects_tenant_id;
CREATE INDEX idx_projects_workspace_id ON projects(workspace_id);

ALTER TABLE channels RENAME COLUMN tenant_id TO workspace_id;
DROP INDEX IF EXISTS idx_channels_scope;
CREATE INDEX idx_channels_scope ON channels(workspace_id, project_id, environment_id);

ALTER TABLE notifications RENAME COLUMN tenant_id TO workspace_id;
DROP INDEX IF EXISTS idx_notifications_inbox;
CREATE INDEX idx_notifications_inbox ON notifications(workspace_id, project_id, environment_id, received_at DESC);

UPDATE application_settings
SET value_json = json_object(
    'activeWorkspaceId', json_extract(value_json, '$.activeTenantId'),
    'activeProjectId', json_extract(value_json, '$.activeProjectId'),
    'activeEnvironmentId', json_extract(value_json, '$.activeEnvironmentId')
)
WHERE key = 'workspace.active_selection'
  AND json_valid(value_json)
  AND json_type(value_json, '$.activeTenantId') IS NOT NULL
  AND json_type(value_json, '$.activeWorkspaceId') IS NULL;
