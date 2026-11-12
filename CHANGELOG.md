# Changelog

All notable changes to **NotifyTrap** are documented in this file.

## [1.1.0] - 2026-11-12

### Added
- **Local Query & Wait HTTP API**:
  - `GET /api/v1/notifications` — scoped channel-key listing with `run_id`, `recipient`, `subject`, `since`, `status`, `search`, `limit`, and `offset` filters.
  - `GET /api/v1/notifications/{notification_id}` — full detail retrieval strictly isolated to the authenticated channel (`404` on cross-channel access).
  - `GET /api/v1/notifications/wait` — event-driven long-polling (`timeout_ms`, default `5000`, max `30000`) backed by an internal Tokio broadcast hub for zero-polling Playwright, Cypress, Pest, and Pytest synchronization (`408 WAIT_TIMEOUT` on expiry).
- **Test-Run Tagging (`X-NotifyTrap-Run`)**:
  - Added SQLite migration `0003_add_test_run_identifier.sql` and indexed filtering across HTTP (`sms`, `push`, `webhooks`) and SMTP email capture.
  - Added run-tag input filter and card/inspector badges in the Inbox UI.
- **Automatic OTP & Magic-Link Extraction**:
  - Added deterministic Rust extractor (`domain/verification_extractor.rs`) detecting 4–8 digit OTP codes (including dashed codes and Persian/Arabic-Indic digit normalization) and verification/magic links without false positives on hex colors or years.
  - Added one-click OTP copy buttons on Inbox list cards and extracted verification artifact pills in the Notification Inspector.
- **Offline Export (`.eml`, `JSON`, `cURL`)**:
  - Added an **Export** menu in the Notification Inspector supporting RFC-5322 `.eml` download, redacted `JSON` download/copy, and secret-safe `cURL` command generation (`<CHANNEL_KEY>` placeholder) for HTTP channels.
- **Bilingual & Quality Coverage**:
  - Added symmetric `en` (LTR) and `fa` (RTL) translations for all new UI controls.
  - Added `proptest` invariants, end-to-end Axum/SMTP wait-wakeup integration tests, and Vitest export unit tests.

## [1.0.0] - 2026-11-12

### Added
- Initial stable release of NotifyTrap with local ESMTP + HTTP (`sms`, `push`, `webhook`) capture engines, Workspace/Project/Environment scoping, encrypted channel credentials, offline sanitized HTML email sandbox, bilingual `en`/`fa` UI, Light/Dark themes, and signed cross-platform auto-updates.
