# Architectural Decision Records (ADRs)

## ADR-001: Nuxt 4 SPA Mode (`ssr: false`) for Tauri 2 Desktop Packaging
- **Context**: Nuxt 4 defaults to `ssr: true`, while Tauri 2 desktop windows load static bundled HTML/JS/CSS into a native WebView without an embedded Node.js server.
- **Decision**: Configure `ssr: false` and `nitro: { preset: 'static' }` in `nuxt.config.ts`, targeting Nuxt's generated static output (`../.output/public`) in `src-tauri/tauri.conf.json`.
- **Consequences**: Zero Node runtime dependency in shipped binaries; all backend data flows exclusively through typed Tauri IPC commands and events.

## ADR-002: `rusqlite` (Bundled SQLite) Over `sqlx`
- **Context**: NotifyTrap needs reliable offline SQLite persistence with `FOREIGN KEY` enforcement, WAL mode, deterministic migrations, and isolated integration testing.
- **Decision**: Use `rusqlite` with the `bundled` feature and a dedicated `DatabaseConnectionManager` running blocking queries safely via `tokio::task::spawn_blocking`, backed by explicit versioned SQL migration files in `src-tauri/src/infrastructure/migrations/`.
- **Consequences**: Zero host `libsqlite3` version drift, no compile-time offline macro cache maintenance, instant `:memory:` / temporary DB testing, and full SQL control.

## ADR-003: Channel Credential Storage (Zero Plaintext in SQLite + Copyable Credentials)
- **Context**: Section 5.7 prohibits storing plaintext credentials in SQLite, while Sections 4.3 and 9.6 require developers to view and copy generated channel credentials (including SMTP username/password).
- **Decision**:
  1. Generate a 256-bit local master encryption key stored in the OS credential vault (`keyring` crate) under service `notifytrap`.
  2. Store each channel's **public identifier** (`credential_identifier`, e.g. `nt_ch_...`), a **SHA-256 digest** (`credential_hash`) for constant-time authentication lookups (`subtle::ConstantTimeEq`), and an **AES-256-GCM encrypted payload** (`encrypted_credential` with random 96-bit nonce) for user-triggered credential reveal/copy.
  3. Abstract key retrieval behind a `SecretKeyProvider` trait (`OsKeyringSecretProvider` in runtime, deterministic `InMemorySecretProvider` in tests, and an explicitly documented local fallback policy only if an OS keyring daemon is absent on Linux).
- **Consequences**: SQLite never contains plaintext credentials; authentication does not require decrypting rows; connection credentials remain inspectable and rotatable.

## ADR-004: Explicit Channel Ownership per `(Workspace, Project, Environment)`
- **Context**: Sections 4.3 and 4.4 describe both per-channel credentials and per-environment credential/channel isolation.
- **Decision**: Every `Project` automatically initializes four standard `Environment` records (`local`, `development`, `testing`, `staging`) with `local` marked default. Each `Channel` explicitly belongs to `(workspace_id, project_id, environment_id)` and owns its own isolated credential.
- **Consequences**: Unambiguous ownership, strict SQLite foreign-key isolation, and trivial switching between environments in the Unified Inbox without cross-environment leakage.

## ADR-005: Custom Tokio SMTP State Machine + `mail-parser`
- **Context**: Most popular Rust mail crates (`lettre`) are SMTP *clients*, not servers, and embedded SMTP server crates vary widely in maintenance and shutdown control.
- **Decision**: Implement a clean, bounded Tokio async SMTP server state machine (`EHLO`/`HELO`, `AUTH PLAIN`, `AUTH LOGIN`, `MAIL FROM`, `RCPT TO`, `DATA`, `RSET`, `NOOP`, `QUIT`) paired with the actively maintained `mail-parser` crate for RFC 5322 / MIME / multipart / attachment decoding.
- **Consequences**: Complete control over graceful shutdown, loopback-only binding, port conflict reporting, max message size enforcement, malformed email resilience, and table/matrix/integration testability.

## ADR-006: Webhook Identification & Authentication Contract
- **Context**: Section 4.8 leaves webhook channel routing unspecified compared to SMS/Push (`X-Channel-Key`). Many external/local webhook emitters only allow configuring a target URL and cannot inject custom headers, while others do support custom headers.
- **Decision**: Expose `ANY /api/v1/webhooks/{credential_identifier}` (and subpaths). Require credential verification via `X-Channel-Key`, `Authorization: Bearer <key>`, or `?channel_key=<key>` query fallback, matched in constant time against `credential_hash`.
- **Consequences**: Works seamlessly across strict header-based clients and URL-only webhook emitters while preserving per-channel authentication and isolation.

## ADR-007: Sandboxed HTML Email Inspection
- **Context**: Incoming HTML emails are untrusted input and may contain `<script>`, event handlers, or remote tracking pixels.
- **Decision**: Sanitize HTML preview strings in Rust with `ammonia` (preserving raw HTML/MIME separately) AND render previews in the frontend inside `<iframe sandbox="" referrerpolicy="no-referrer" :srcdoc="sanitizedHtml">` with a strict Content Security Policy blocking scripts and remote network requests by default.
- **Consequences**: Dual defense-in-depth against XSS and unintended outbound network calls.

## ADR-008: Missing Agent Skills Mitigation
- **Context**: Part 2 requires discovering/installing Rust, Tauri, Nuxt, UI/UX, design system, i18n, and testing skills. Marketplace inspection found no dedicated Rust, Tauri, Nuxt, or Testing skills available in the Antigravity Marketplace.
- **Decision**: Document the missing marketplace skills here and compensate using installed local skills (`ui-ux-pro-max`, `impeccable`, `lean-build`, `security-audit`, `verify-and-stop`), the live `nuxt-ui` MCP documentation server, and verified official documentation (`v2.tauri.app`, `nuxt.com`, `ui.nuxt.com`, `docs.rs`).

## ADR-009: Signed In-App Updates & Cross-Platform GitHub Releases
- **Context**: NotifyTrap targets Linux (`.deb`, `.rpm`, `.AppImage`), Windows (`.exe`, `.msi`), and macOS (`.dmg`), and requires cryptographically verified in-app updates with non-blocking background detection and toast actions while staying strictly offline-safe on boot.
- **Decision**: Adopt official Tauri 2 plugins `tauri-plugin-updater` + `tauri-plugin-process` with `bundle.createUpdaterArtifacts: true` and Ed25519 signature verification (`plugins.updater.pubkey`), paired with `.github/workflows/release.yml` using `tauri-apps/tauri-action@v0`. Run silent background checks once after mount when `app.auto_update_check_enabled` is true (never blocking startup and never raising noisy offline errors), surfacing an actionable toast (`Download & Update` / `View Details`) only when a newer signed release is found.
- **Consequences**: Tamper-proof binary upgrades across Windows, macOS, and Linux AppImage without paid vendor certificates required for manifest verification, plus full control in Settings. Linux `.deb`/`.rpm` installs continue upgrading through standard distro/release installers.

---

## ADR-006: Event-Driven HTTP Wait API & Pure Verification Extraction (`v1.1.0`)

- **Status**: Accepted
- **Decision**:
  1. Extend `EventPublisher` (`TauriEventPublisher` and `BroadcastEventPublisher`) with a Tokio `broadcast::Receiver<NotificationCapturedEvent>` subscription channel so `GET /api/v1/notifications/wait` subscribes before its initial SQLite lookup and wakes instantaneously on matching captures without polling `sleep`.
  2. Compute `verificationArtifacts` (OTP codes and verification/magic links) deterministically in pure Rust (`domain/verification_extractor.rs`) on read/capture instead of persisting duplicated extracted tokens, ensuring backwards compatibility with existing v1.0.0 SQLite rows.
- **Consequences**: Zero-polling E2E test synchronization (`408 WAIT_TIMEOUT` on expiry), strict channel-key scoping on all query/wait endpoints, and automatic extraction across existing and newly captured notifications.

---

## ADR-010: Strict Offline `cid:` Attachment Hydration & Sandboxed Image Preview (`v1.2.0`)

- **Status**: Accepted
- **Decision**:
  1. Allow `<img>` tags with `cid:` `src` attributes through `ammonia` (`html_sanitizer.rs`) while actively stripping any `<img>` element whose `src` is not a `cid:` URI, preserving complete remote tracking-pixel blocking.
  2. Resolve `cid:` images in the frontend (`EmailInspectorView.vue` / `attachmentPreview.ts`) strictly using local `data:` URIs fetched from SQLite via `get_notification_attachment` with raster MIME allowlisting (`png`, `jpeg`, `gif`, `webp`, excluding `image/svg+xml` from inline preview rendering).
  3. Implement email quality auditing, SMS GSM-7/UCS-2 counting, and LCS notification diffing as pure deterministic frontend utilities rather than additional database columns or external requests.
- **Consequences**: Authentic HTML email rendering with embedded logos and full attachment inspection without weakening `img-src data:; connect-src 'none'; script-src 'none';` sandbox isolation.
