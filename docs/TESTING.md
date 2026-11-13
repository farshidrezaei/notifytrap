# NotifyTrap — Testing Strategy & Quality Gate

## 1. Testing Layers

1. **Rust Unit & Table-Driven Tests (`cargo test`)**
   - Credential validation & constant-time matching
   - Domain identifier & payload validation
   - Workspace / Project / Environment / Channel isolation invariants
   - SMTP command state machine & MIME parsing (`mail-parser`)
   - HTTP request validation & error code mapping
   - HTML sanitization (`ammonia`)

2. **Rust Property-Based Tests (`proptest`)**
   - Slug & identifier normalization invariants
   - Credential token parsing round-trips
   - Arbitrary MIME & webhook payload parser safety (no panics on malformed inputs)
   - Pagination bounds & serialization round-trips

3. **Rust Matrix & Real Integration Tests (`src-tauri/tests/`)**
   - Real Axum HTTP server bound to ephemeral `127.0.0.1:0`
   - Real Tokio SMTP server bound to ephemeral `127.0.0.1:0`
   - Isolated temporary SQLite database per test (`tempfile::TempDir`, zero shared mutable state)
   - Full matrix coverage:
     - Channels (`email`, `sms`, `push`, `webhook`) x (valid, invalid payload, missing fields, invalid key, disabled channel, deleted channel, cross-workspace/project/environment)
     - HTTP matrix (`GET`, `POST`, unsupported methods, valid/invalid JSON, empty/oversized payload, unknown endpoint)
     - SMTP matrix (plain text, HTML, multipart, multiple recipients, attachments, invalid MIME, missing subject, empty body, malformed message, auth failure)
     - Persistence matrix (CRUD, missing entity, FK violation, transaction rollback)

4. **Frontend Unit, Validation, i18n, and Hardening Tests (`vitest`)**
   - Translation completeness & placeholder parity (`validate:i18n` across `en` & `fa`, `tests/i18nValidation.spec.ts`)
   - Design tokens & skeleton dimension invariants (`tests/designTokens.spec.ts`, `tests/loadingSkeletons.spec.ts`)
   - Zod schemas, Maska presets, multi-script digit normalization (`۰-۹` / `٠-٩` / `०-९`), English-only & slug validation (`tests/formValidationAndMasks.spec.ts`, `tests/phase5Helpers.spec.ts`)
   - Code language detection & offline syntax highlighting (`tests/codeHighlighting.spec.ts`)
   - Plain-text email HTML escaping, offline CSP iframe rendering across Light/Dark themes, RTL logical CSS verification, and icon-button `aria-label` checks (`tests/securityAndHardening.spec.ts`)

5. **End-to-End Desktop Verification**
   - Launch flow -> Workspace -> Project -> Channel -> Copy credentials -> Real SMTP/HTTP capture -> Unified Inbox -> Notification Inspector -> Switch to `fa` (RTL) & Dark mode -> Restart persistence.

---

## 2. Mandatory Test Rules

- **No arbitrary `sleep()`** in asynchronous tests — use channels, readiness signals, or `tokio::net::TcpListener` ephemeral ports.
- **No trivial assertions** (`assert!(result.is_some())` without verifying actual values).
- **Deterministic fixtures** and isolated databases for every test.
- Test failure paths with the same rigor as happy paths.
- **v1.1.0 Additions**:
  - Property-based (`proptest`) & table-driven OTP/magic-link extraction tests (`domain/verification_extractor.rs`).
  - End-to-end Axum/SMTP integration test (`http_query_wait_run_tag_and_verification_extraction_end_to_end`) verifying `X-NotifyTrap-Run`, cross-channel `404` isolation, race-free `/api/v1/notifications/wait` wakeup, and `408 WAIT_TIMEOUT`.
  - Vitest unit suite (`tests/notificationExport.spec.ts`) covering `.eml`, `JSON`, and secret-safe `cURL` export.
- **v1.2.0 Additions**:
  - Rust unit test (`preserves_cid_images_and_strips_remote_images_and_scripts`) verifying `cid:` inline image survival alongside remote tracker/XSS removal.
  - Vitest unit suite (`tests/phase2InspectorPro.spec.ts`) verifying GSM-7/UCS-2 SMS segment math, offline email accessibility/delivery auditing, LCS diff computation, raster MIME preview safety (blocking SVG inline preview), `cid:` resolution, and Dark Client CSP sandbox generation.
- **v1.3.0 Additions**:
  - Rust unit test (`allows_strict_loopback_targets_and_rejects_external_hosts`) verifying loopback-only replay URL validation (`127.0.0.1`, `localhost`, `[::1]`) and rejection of external hosts, HTTPS, userinfo, and fragments.
  - End-to-end Axum/SMTP integration test (`webhook_mock_loopback_replay_and_chaos_rules_end_to_end`) verifying webhook mock responses (`201` + custom headers/body + SQLite capture), ephemeral `127.0.0.1` webhook replay + external URL rejection, HTTP `429` chaos failure injection, and SMTP `451` chaos failure injection.
  - Vitest unit suite (`tests/phase3MockReplayChaos.spec.ts`) verifying frontend loopback URL validation, default replay target construction, and mock header JSON parsing.
- **v1.4.0 Additions**:
  - Rust unit & `proptest` suite (`domain::search_query::tests`) verifying structured search qualifiers, negation, quoted phrases, bounded `/regex/`, and panic-free arbitrary inputs.
  - End-to-end Axum integration test (`structured_search_and_local_mcp_json_rpc_end_to_end`) verifying `channel:sms otp:true /regex/` search and MCP `initialize`, `tools/list`, and `tools/call` (`extract_otp_codes`).
  - Vitest unit suite (`tests/phase4DxSavedFilters.spec.ts`) verifying saved filter JSON parsing and project-scoped key generation.
- **v1.5.0 Additions**:
  - Rust integration test (`tests/headless_companion_integration.rs`) covering headless server, SMTP, and Web Companion Axum SPA serving lifecycle.
  - Rust matrix test (`channel_forwarding_rules_persistence_and_matrix_update`) verifying SQLite persistence and domain rule validation.
  - Rust database maintenance test (`database_storage_statistics_and_vacuum_lifecycle`) verifying SQLite disk statistics, retention cleanup, and database VACUUM.
  - Vitest suites (`tests/phase1SimulationAndTemplateAudit.spec.ts`, `tests/phase2DeepAssetAndStorageLifecycle.spec.ts`, `tests/phase3IngressAndForwardingPipeline.spec.ts`) covering realistic device frames, MJML parser, placeholder linter, asset auditor, and auto-forwarding rule parsing.

---

## 3. Verification Commands

```bash
# Rust formatting, lints, unit + integration tests
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets

# Frontend i18n, typecheck, unit tests, static build
bun run validate:i18n
bun run typecheck
bun run test:unit
bun run build
```
