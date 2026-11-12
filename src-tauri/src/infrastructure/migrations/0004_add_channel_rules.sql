ALTER TABLE channels ADD COLUMN webhook_mock_json TEXT NOT NULL DEFAULT '';
ALTER TABLE channels ADD COLUMN chaos_rule_json TEXT NOT NULL DEFAULT '';
