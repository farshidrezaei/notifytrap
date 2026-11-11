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
