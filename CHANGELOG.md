# Changelog

All notable changes to **NotifyTrap** are documented in this file.

## [1.3.0] - 2026-11-12

### Added
- **Per-Channel Webhook Mock Responses**: Configure custom HTTP status codes (`100..599`), response headers (up to 32 validated headers), response bodies (up to 64 KiB), and artificial response delays (`0..10000 ms`) on any Webhook channel (`Channels -> Rules`). Incoming webhook captures are persisted to SQLite first before returning the configured mock response.
- **Strict Loopback-Only Webhook Replay**: Replay captured webhook requests directly from `Inbox -> Webhook -> Replay` to local development targets (`http://127.0.0.1`, `http://localhost`, `http://[::1]`) over `tokio::net::TcpStream` with bounded 5s connect / 10s I/O timeouts and a 1 MiB response cap. External hosts, userinfo, and fragments are strictly rejected.
- **HTTP & SMTP Chaos Rules**: Inject per-channel artificial latency (`0..10000 ms`) and probabilistic failure rates (`0..100%`) with custom HTTP (`400..599`) or SMTP (`400..599`) error codes across SMS, Push, Webhook, and SMTP `DATA` ingestion to test client retry and backoff logic locally.
- **Database Migration & Channel Rule UI**: Added SQLite migration `0004_add_channel_rules.sql` (`webhook_mock_json`, `chaos_rule_json`), `ChannelRulesModal.vue` with active `Mock` / `Chaos` badges on `ChannelCredentialCard.vue`, and `WebhookReplayModal.vue` with full `en` (LTR) and `fa` (RTL) translations.
- **Tests**: Added Rust unit test `allows_strict_loopback_targets_and_rejects_external_hosts`, end-to-end integration test `webhook_mock_loopback_replay_and_chaos_rules_end_to_end`, and Vitest suite `tests/phase3MockReplayChaos.spec.ts`.

## [1.2.0] - 2026-11-12

### Added
- **Email Viewport Switcher & Dark Client Simulation**:
  - Added Desktop, Tablet (`768px`), and Mobile (`375px`) preview width controls above the sandboxed HTML email viewer (`EmailInspectorView.vue`).
  - Added an opt-in **Dark Client** simulator (`emailSandbox.ts`) inverting light-coded HTML email canvases while preserving photo luminance without loosening the strict zero-network offline CSP (`script-src 'none'`, `connect-src 'none'`).
- **Inline `cid:` Image Hydration & Safe Attachment Preview/Download**:
  - Updated backend HTML sanitization (`html_sanitizer.rs`) to preserve sanitized `cid:` inline image references while still blocking external `http://` and `https://` tracking pixels.
  - Added Tauri command `get_notification_attachment` returning base64-encoded MIME attachment bytes scoped by attachment ID.
  - Added inline `cid:` `data:` URI resolution in HTML previews (`attachmentPreview.ts`), safe raster-image (`PNG`/`JPEG`/`WEBP`/`GIF`) and syntax-highlighted text (`TXT`/`CSV`/`JSON`/`HTML`/`XML`) preview modal (`EmailAttachmentModal.vue`), SVG XSS exclusion, and local file downloads.
- **Offline Email Audit & Diagnostics Tab**:
  - Added deterministic offline checks (`emailAudit.ts`) reporting payload size (`> 100 KB` Gmail clipping warning), subject presence, multipart plain-text fallback coverage, missing image `alt` attributes, blocked remote tracking pixels, unresolved `cid:` references, relative/empty links, and `List-Unsubscribe` header presence.
- **SMS GSM-7 / UCS-2 Segment Counter & Push Device Previews**:
  - Added accurate SMS encoding detection and segment calculation (`smsAnalyzer.ts`: GSM-7 basic + extension septet counting with `160/153` split limits and `UCS-2` UTF-16 `70/67` limits) alongside a mobile SMS message preview (`SmsInspectorView.vue`).
  - Added **iOS**, **Android**, and **Web Push** lock-screen / desktop notification card mockups (`PushInspectorView.vue`).
- **Notification Diff Viewer**:
  - Added **Compare** modal (`NotificationDiffModal.vue`) and LCS line-diff engine (`textDiff.ts`) comparing formatted payloads, raw payloads, or metadata across captured notifications.
- **Bilingual & Quality Coverage**:
  - Added full `en` (LTR) and `fa` (RTL) translations for all Inspector Pro tools, audit findings, and diff modes.
  - Added unit tests (`tests/phase2InspectorPro.spec.ts`) and Rust sanitizer `cid:` unit tests.

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
