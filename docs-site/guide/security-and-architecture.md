# Security & Architecture

NotifyTrap is engineered as a **modular monolith** running inside one desktop binary — never relaying traffic to external mail, SMS, push, or webhook destinations.

## Security Guarantees

- **Strict Loopback Network Isolation**: Embedded Axum HTTP and Tokio ESMTP listeners bind strictly to `127.0.0.1` by default.
- **Encrypted Channel Keys (`AES-256-GCM` + `SHA-256`)**:
  - Raw secrets (`ntsec_...`) are encrypted at rest with a 256-bit master key stored in the native OS keyring (`apple-native`, `windows-native`, `sync-secret-service`), backed by an atomic `0o600` owner-only `master.key` fallback when no daemon is present.
  - Authentication hashes are verified using `SHA-256` with constant-time byte comparison (`subtle::ConstantTimeEq`).
- **Untrusted Email HTML Sandboxing**:
  - Email HTML bodies are sanitized in Rust with `ammonia` before SQLite storage and Tauri IPC transport.
  - The frontend renders sanitized HTML inside an isolated `<iframe sandbox="" referrerpolicy="no-referrer">` with strict CSP (`default-src 'none'`).
- **Automatic Credential Redaction**:
  - Captured HTTP requests scrub `authorization`, `proxy-authorization`, `x-channel-key`, `x-api-key`, `cookie`, and `set-cookie` headers, plus sensitive query params (`channel_key`, `token`, `api_key`, `apikey`, `secret`, `password`, `authorization`), replacing them with `[REDACTED]`.
- **Bounded Resource & Retention Controls**:
  - Bounded SMTP line reads, 60-second idle session timeouts, max 64 concurrent SMTP sessions, configurable HTTP payload caps, and automatic retention pruning on startup and settings save.

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
