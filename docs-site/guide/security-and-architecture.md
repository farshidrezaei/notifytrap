# Security & Architecture

NotifyTrap is engineered as a **modular monolith** running inside one desktop binary — never relaying traffic to external mail, SMS, push, or webhook destinations.

## Security Guarantees

- **Strict Loopback Network Isolation**: Embedded Axum HTTP and Tokio ESMTP listeners bind strictly to `127.0.0.1` by default.
- **Encrypted Channel Keys (`AES-256-GCM` + `SHA-256`)**:
  - Raw secrets (`ntsec_...`) are encrypted at rest with a 256-bit master key stored in the native OS keyring (`apple-native`, `windows-native`, `sync-secret-service`), backed by an atomic `0o600` owner-only `master.key` fallback when no daemon is present.
  - Authentication hashes are verified using `SHA-256` with constant-time byte comparison (`subtle::ConstantTimeEq`).
- **Untrusted Email HTML Sandboxing**:
  - Email HTML bodies are sanitized in Rust with `ammonia` before SQLite storage and Tauri IPC transport, preserving only `cid:` inline image references while stripping scripts and remote tracking pixels (`http://` / `https://`).
  - The frontend resolves `cid:` parts locally via `data:` URIs (`get_notification_attachment`, raster images only — blocking inline SVG previews) and renders HTML inside an isolated `<iframe sandbox="" referrerpolicy="no-referrer">` with strict CSP (`default-src 'none'; img-src data:; script-src 'none'; connect-src 'none';`).
- **Scoped Automation & Pure Extraction**:
  - Test-run tags (`X-NotifyTrap-Run`, migration `0003_add_test_run_identifier.sql`) and `GET /api/v1/notifications*` query/wait routes enforce strict channel-key scoping via `SHA-256` constant-time checks, returning `404` for cross-channel IDs.
  - OTP codes and verification links are extracted deterministically in pure Rust (`domain/verification_extractor.rs`) without persisting extra tokens.
- **Automatic Credential Redaction**:
  - Captured HTTP requests scrub `authorization`, `proxy-authorization`, `x-channel-key`, `x-api-key`, `cookie`, and `set-cookie` headers, plus sensitive query params (`channel_key`, `token`, `api_key`, `apikey`, `secret`, `password`, `authorization`), replacing them with `[REDACTED]`.
- **Bounded Resource & Retention Controls**:
  - Bounded SMTP line reads, 60-second idle session timeouts, max 64 concurrent SMTP sessions, configurable HTTP payload caps, and automatic retention pruning on startup and settings save.
- **Strict Loopback-Only Webhook Replay & Chaos Simulation (`v1.3.0`)**:
  - Webhook replay runs directly over `tokio::net::TcpStream` (`infrastructure/http/loopback_replay.rs`) and strictly permits only `http://127.0.0.1`, `http://localhost`, and `http://[::1]` targets without userinfo or fragments, enforcing bounded 5s connect / 10s I/O timeouts and a 1 MiB response cap.
  - Per-channel Webhook Mock & Chaos Rules (`webhook_mock_json`, `chaos_rule_json`, migration `0004_add_channel_rules.sql`) stay strictly local to the embedded HTTP/SMTP listeners.

## Modular Monolith Layers

### Rust Backend (`src-tauri/src/`)

| Layer | Responsibility |
| --- | --- |
| `commands/` | Thin Tauri IPC handlers delegating to services with structured error mapping |
| `application/` | Runtime state & lifecycle management for local HTTP/SMTP capture servers |
| `services/` | Domain orchestration, constant-time auth verification, retention cleanup |
| `domain/` | Typed entities (`Workspace`, `Project`, `Environment`, `Channel`, `Notification`), enums & validation |
| `infrastructure/` | SQLite repositories & migrations, Axum HTTP routes, Tokio ESMTP + `mail-parser`, `ammonia` & crypto |
| `events/` | Real-time `notification:captured` desktop IPC broadcaster |

### Frontend (`app/`)

- Built with **Nuxt 4 (`ssr: false`)**, **Nuxt UI v4**, **Tailwind CSS v4**, and **TypeScript (`strict: true`)**.
- **Bilingual & Symmetrical Layout**: Complete `en` (LTR) and `fa` (RTL) dictionaries checked by automated CI validation (`bun run validate:i18n`), plus automatic conversion of Persian (`۰-۹`), Arabic-Indic (`٠-٩`), and Devanagari (`०-९`) numerals inside numeric fields.
