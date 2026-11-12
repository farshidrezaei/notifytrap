# NotifyTrap — Architecture & Roadmap

## 1. System Overview

NotifyTrap is a **modular monolith** desktop application built with **Tauri 2**, **Rust**, **Nuxt 4 (SPA mode)**, **Nuxt UI 4**, **Tailwind CSS v4**, and **SQLite**.

```
+-------------------------------------------------------------------+
|                        Tauri 2 Desktop Shell                      |
|  +-----------------------------+   +---------------------------+  |
|  |   Nuxt 4 Frontend (WebView) |   |   Rust Application Core   |  |
|  |  - Nuxt UI 4 + Tailwind v4  |<->|  - Tauri Commands (IPC)   |  |
|  |  - @nuxtjs/i18n (en / fa)   |   |  - Tauri Events (Realtime)|  |
|  |  - Design System (/design-  |   +-------------+-------------+  |
|  |    system)                  |                 |                |
|  +-----------------------------+                 v                |
|                                    +---------------------------+  |
|                                    |     Service Layer         |  |
|                                    | Workspace/Project/Channel/|  |
|                                    |  Notification Services    |  |
|                                    +------+-------------+------+  |
|                                           |             |         |
|           +-------------------------------+             |         |
|           v                                             v         |
|  +-----------------------------+   +---------------------------+  |
|  |  Background Capture Servers |   |     Persistence & Sec     |  |
|  |  - Local SMTP Server (Tokio)|   |  - SQLite (rusqlite WAL)  |  |
|  |  - Local HTTP Server (Axum) |   |  - Secret Provider (AES-  |  |
|  +-----------------------------+   |    GCM + OS Keyring)      |  |
|                                    +---------------------------+  |
+-------------------------------------------------------------------+
```

---

## 2. Rust Backend Modules (`src-tauri/src/`)

| Module | Responsibility |
| --- | --- |
| `main.rs` / `lib.rs` | Binary entry point, Tauri builder wiring, capability/plugin registration |
| `application/` | `ApplicationState`, `ApplicationLifecycle` (startup, graceful shutdown, service health) |
| `configuration/` | `ApplicationConfiguration`, `ServerConfiguration` (ports, bind host, payload limits, pagination) |
| `domain/` | Pure domain entities, enums, validation rules, typed errors (`workspace`, `project`, `environment`, `channel`, `notification`) |
| `infrastructure/database/` | SQLite connection manager, transaction helpers, repository implementations |
| `infrastructure/migrations/` | Versioned deterministic SQL migrations (`0001_initial_schema.sql`, `0002_rename_tenant_to_workspace.sql`, etc.) |
| `infrastructure/security/` | Constant-time credential hashing/verification, AES-256-GCM envelope encryption, `SecretKeyProvider`, HTML sanitization (`ammonia`) |
| `infrastructure/smtp/` | Managed Tokio local SMTP server + MIME parser (`mail-parser`) |
| `infrastructure/http/` | Managed Axum local HTTP capture server (`POST /api/v1/sms`, `POST /api/v1/push`, `ANY /api/v1/webhooks/:channel_identifier`, `GET /api/v1/health`) |
| `services/` | Orchestration & business invariants (`WorkspaceService`, `ProjectService`, `EnvironmentService`, `ChannelService`, `NotificationService`) |
| `commands/` | Thin Tauri IPC command boundaries validating inputs and delegating to services |
| `events/` | Typed Tauri event payloads (`notification:captured`, `server:status_changed`) |

---

## 3. Database Entities & Relationships

All tables enforce `PRAGMA foreign_keys = ON`.

```
workspaces (1) ──< projects (N) ──< environments (N)
                                          │
                                          ▼
                                 channels (N) ──< notifications (N)
                                                       │
                                                       ▼
                                          notification_attachments (N)
```

### Core Tables
1. **`workspaces`**: `id` (UUID PK), `name` (unique), `slug` (unique), `description`, `created_at`, `updated_at`.
2. **`projects`**: `id` (UUID PK), `workspace_id` (FK -> `workspaces.id` CASCADE), `name`, `slug`, `description`, `created_at`, `updated_at`. Unique `(workspace_id, slug)`.
3. **`environments`**: `id` (UUID PK), `project_id` (FK -> `projects.id` CASCADE), `name`, `slug`, `kind` (`local` | `development` | `testing` | `staging`), `is_default`, `created_at`, `updated_at`. Unique `(project_id, slug)`.
4. **`channels`**: `id` (UUID PK), `workspace_id` (FK -> `workspaces.id` CASCADE), `project_id` (FK -> `projects.id` CASCADE), `environment_id` (FK -> `environments.id` CASCADE), `name`, `channel_type` (`email` | `sms` | `push` | `webhook`), `is_enabled`, `credential_identifier` (unique public prefix/username), `credential_hash` (SHA-256 hex for constant-time auth), `encrypted_credential` (AES-256-GCM ciphertext+nonce), `created_at`, `updated_at`.
5. **`notifications`**: `id` (UUID PK), `workspace_id` (FK), `project_id` (FK), `environment_id` (FK), `channel_id` (FK -> `channels.id` CASCADE), `channel_type`, `status` (`captured` | `read` | `archived`), `recipient`, `sender`, `subject`, `preview_text`, `payload_json` (canonical structured JSON per channel), `raw_payload` (raw MIME source or raw HTTP request/body), `received_at`, `created_at`.
   - Indexed on `(workspace_id, project_id, environment_id, received_at DESC)`, `(channel_id, received_at DESC)`, `(channel_type)`, and `(status)`.
6. **`notification_attachments`**: `id` (UUID PK), `notification_id` (FK -> `notifications.id` CASCADE), `filename`, `content_type`, `size_bytes`, `content_id`, `is_inline`, `content_blob`, `created_at`.
7. **`application_settings`**: `key` (PK), `value_json`, `updated_at` (server ports, bind address, retention policy, active workspace state).

---

## 4. Local Capture API Contracts

Default loopback binding: `127.0.0.1`. Default HTTP port: `4080`. Default SMTP port: `1025`.

### 4.1. Email Capture (Local SMTP)
- **Host/Port**: `127.0.0.1:1025` (configurable)
- **Auth**: `AUTH PLAIN` and `AUTH LOGIN`
  - `username` = `channel.credential_identifier`
  - `password` = channel secret (verified via `credential_hash` using `subtle::ConstantTimeEq`)
- **Parsing**: MIME headers, `From`, `To`/`Cc`/`Bcc`, `Subject`, `text/plain`, sanitized `text/html` preview + raw `text/html`, multipart & attachments, full raw RFC 5322 source.
- **Safety**: Reject messages exceeding `MAX_SMTP_MESSAGE_BYTES` (default 10 MiB); handle missing subject/HTML/body gracefully.

### 4.2. SMS Capture (`POST /api/v1/sms`)
- **Headers**: `X-Channel-Key: <credential_identifier>.<secret>`, `Content-Type: application/json`
- **Request Body**:
  ```json
  {
    "to": "+989120000000",
    "message": "Your verification code is 123456",
    "from": "NotifyApp",
    "metadata": {}
  }
  ```
- **Response (`202 Accepted`)**:
  ```json
  {
    "notificationId": "uuid",
    "channelId": "uuid",
    "status": "captured",
    "receivedAt": "2026-11-11T19:10:00Z"
  }
  ```

### 4.3. Push Capture (`POST /api/v1/push`)
- **Headers**: `X-Channel-Key: <credential_identifier>.<secret>`, `Content-Type: application/json`
- **Request Body**:
  ```json
  {
    "device_token": "sample-device-token",
    "platform": "android",
    "title": "Welcome",
    "body": "Your account is ready",
    "data": { "screen": "dashboard" }
  }
  ```
- **Response (`202 Accepted`)**: Same canonical capture receipt JSON.

### 4.4. Webhook Capture (`ANY /api/v1/webhooks/{credential_identifier}` and `/api/v1/webhooks/{credential_identifier}/{*subpath}`)
- **Auth**: Channel identified by `credential_identifier` in path AND authenticated via `X-Channel-Key` header or `Authorization: Bearer <key>` (or `?channel_key=<key>` when custom headers are unsupported by caller, verified against `credential_hash`).
- **Captured Fields**: HTTP method, request path, query parameters, headers (sensitive auth header masked in display metadata unless needed), `Content-Type`, raw body (JSON parsed when valid + raw text preserved), received timestamp, remote peer metadata.
- **Response (`202 Accepted`)**: Same canonical capture receipt JSON.

---

## 5. Frontend Structure (`app/`)

- **Rendering**: `ssr: false` (Tauri 2 static WebView bundle).
- **Root**: `<UApp>` synced automatically with `@nuxtjs/i18n` (`en` LTR / `fa` RTL) and `useLocaleHead()`.
- **Layout**: Custom frameless `DesktopAppBar` + `USidebar` (collapsible inset sidebar with hierarchical Tree `WorkspaceProjectSwitcher`, `EnvironmentContextBar`, project-scoped primary navigation, and bottom utility navigation) + `DesktopStatusFooter`.
- **Pages**:
  - `/` (`Overview` / First-Run Onboarding): Initializes initial Workspace + Project when empty, or renders the active Project Dashboard (SMTP & HTTP capture health with copyable endpoints, channel count matrix, quick integration snippet, and recent captured notifications).
  - `/inbox` (`Unified Inbox`): Real-time list via `notification:captured` events, channel/status/search filters, pagination, and channel-specific `NotificationInspector` (sandboxed HTML iframe with strict offline CSP for email, SMS bubble, Push card, Webhook HTTP breakdown).
  - `/workspaces` (`Workspaces`): Workspace & nested Project overview and CRUD with instant dashboard jump.
  - `/projects` (`Projects`): Project CRUD, environment inspection, and default-environment configuration.
  - `/channels` (`Channels`): Channel CRUD, enable/disable switch, credential reveal/copy/rotate, and copy-ready integration snippets.
  - `/settings` (`Settings`): Appearance, language (`en`/`fa`), live local server endpoint/payload configuration persisted to SQLite and applied to running servers, and retention rules.
  - `/design-system`: Mandatory living visual source of truth (tokens, component states, Light/Dark + LTR/RTL verification).

---

## 6. Phased Implementation Roadmap

- **Phase 0 (Complete)**: Repository & skill audit, official doc alignment, `AGENTS.md`, architecture, ADRs, DB schema, API contracts, design tokens, i18n structure, testing & dev docs.
- **Phase 1 (Complete)**: Foundation shell — Nuxt 4 + Nuxt UI 4 + `@nuxtjs/i18n` + Tauri 2 + Rust modular skeleton + SQLite migrations + CSS design tokens + i18n CI validator + passing checks.
- **Phase 2 (Complete)**: Full Design System & `/design-system` showcase (`Midnight Indigo v1.0`, frameless inset shell, custom AppBar/Footer, Zod + Maska form validation, Light/Dark, `en` LTR / `fa` RTL).
- **Phase 3 (Complete)**: Domain & persistence (`Workspace`, `Project`, `Environment`, `Channel`, `Notification`), SQLite repositories, service layer, AES-256-GCM + SHA-256 constant-time secret management, `ammonia` HTML sanitization, Tauri CRUD/query IPC commands, table-driven, property (`proptest`), and isolation matrix tests.
- **Phase 4 (Complete)**: Local capture infrastructure (`CaptureServerCoordinator`, Tokio SMTP server with `AUTH PLAIN`/`AUTH LOGIN` + `mail-parser`, Axum HTTP APIs `/api/v1/health`, `/api/v1/sms`, `/api/v1/push`, `/api/v1/webhooks/{id}`, real-time `EventPublisher` / `TauriEventPublisher`, live integration tests over real TCP sockets).
- **Phase 5 (Complete)**: Desktop UX — First-Run Onboarding, hierarchical Tree Workspace/Project switcher + Environment bar + Project-scoped sidebar, live Project Dashboard, Channel management & reveal/rotate credentials, real-time Unified Inbox with channel-specific inspectors and sandboxed HTML preview iframe, Workspaces/Projects CRUD, and live Settings persistence (`en`/`fa`, Light/Dark).
- **Phase 6 (Complete)**: Hardening — bounded SMTP line/connection reads (`Semaphore` + idle timeouts), expanded HTTP sensitive header/query redaction & safe internal errors, OS keyring + atomic `0o600` fallback master key caching, plain-text email HTML escaping inside offline CSP iframe sandbox, automatic notification retention pruning on startup and settings save, accessibility `aria-pressed` / `aria-label` checks, full matrix verification, and release desktop binary build (`tauri build --no-bundle`).
- **Phase 7 (Complete)**: Documentation & release — comprehensive `README.md` with installation, SMTP configuration examples, HTTP API examples, contribution workflow, release checklist, and synchronized technical docs.
- **v1.1.0 (Phase 1 — Test Automation, OTP Extraction & Local Query API)**:
  - Migration `0003_add_test_run_identifier.sql` adding `test_run_identifier` (`X-NotifyTrap-Run` across HTTP & SMTP ingestion) and indexed scoped filtering.
  - Pure domain extractor (`src-tauri/src/domain/verification_extractor.rs`) detecting OTP codes (including Persian/Arabic-Indic digit normalization) and verification/magic links.
  - Event-driven HTTP query/wait endpoints (`src-tauri/src/infrastructure/http/query_routes.rs`) backed by `EventPublisher::subscribe_notifications()` broadcast wakeup.
  - Local `.eml`, `JSON`, and `cURL` export formatter (`app/utils/notificationExport.ts`) and Inbox verification artifact pills.
- **v1.2.0 (Phase 2 — Rich Inspector Pro: Email, SMS, Push & Diff)**:
  - Preserved `cid:` inline image references in `html_sanitizer.rs` (filtering out all remote `http://`/`https://` image tags) and added `get_notification_attachment` IPC command returning base64-encoded MIME payloads.
  - Added pure offline frontend analyzers (`smsAnalyzer.ts`, `emailAudit.ts`, `textDiff.ts`, `attachmentPreview.ts`) plus responsive Email viewport/Dark Client controls, inline `cid:` hydration, attachment preview modal (`EmailAttachmentModal.vue`), SMS GSM-7/UCS-2 segment meter, Push iOS/Android/Web Push device mockups, and notification diff modal (`NotificationDiffModal.vue`).
