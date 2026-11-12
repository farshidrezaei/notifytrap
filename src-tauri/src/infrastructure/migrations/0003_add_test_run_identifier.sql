ALTER TABLE notifications
    ADD COLUMN test_run_identifier TEXT NOT NULL DEFAULT '';

CREATE INDEX idx_notifications_test_run
    ON notifications(workspace_id, project_id, environment_id, test_run_identifier, received_at DESC);
