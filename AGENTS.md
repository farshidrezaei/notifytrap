# AGENTS.md — NotifyTrap Permanent Engineering Constitution

This document is the permanent engineering contract for **NotifyTrap**. Every contributor and coding agent must read and obey it before designing, editing, or reviewing code.

Source specification: `impliment.md`.

---

## 1. Product Scope & Identity

- **NotifyTrap** is a local-first, offline-capable, open-source desktop utility (Tauri 2 + Rust + Nuxt 4 + Nuxt UI + SQLite) for capturing, inspecting, and debugging application notifications during development.
- **Not a SaaS**, not distributed infrastructure, not microservices. Architecture is a **modular monolith** running in one desktop application with managed local background services.
- **Zero real-world delivery**: NotifyTrap must never send emails, SMS, push notifications, or webhooks to external real recipients.

---

## 2. Non-Negotiable Coding Rules

### 2.1. Naming and Readability
- Never use abbreviations in variable, function, method, class, component, module, or type names unless the abbreviation is an established technical term or protocol name (`HTTP`, `HTTPS`, `SMTP`, `MIME`, `API`, `URL`, `URI`, `UUID`, `JSON`, `HTML`, `CSS`, `SQL`, `TCP`, `TLS`, `IPC`, `UI`, `RTL`, `LTR`, `SMS`, `ID`).
- Forbidden abbreviations: `cfg`, `ctx`, `mgr`, `svc`, `repo`, `req`, `res`, `msg`, `usr`, `val`, `env` (use `environment`), `notif` (use `notification`), `proj` (use `project`), `chan` (use `channel`), `auth` (prefer `authentication` / `credential` where clearer).
- Boolean identifiers must carry explicit prefixes: `is_`, `has_`, `can_`, `should_` (Rust) / `is`, `has`, `can`, `should` (TypeScript).
- Functions must state intent through their names without needing explanatory comments for cryptic identifiers.

### 2.2. No Hardcoded Values
Every non-structural value must belong to one of six categories:
1. **Enum** — finite, explicitly defined set of states or variants (`ChannelType`, `EnvironmentKind`, `NotificationStatus`).
2. **Configuration value** — user- or application-configurable setting (`ServerConfiguration`).
3. **Named constant** — true invariant (`DEFAULT_HTTP_PORT`, `MAX_HTTP_PAYLOAD_BYTES`).
4. **Design token** — visual design decision (`--ui-primary`, `--space-*`, `@theme` tokens).
5. **Translation key** — any user-facing text (`$t('inbox.empty.title')`).
6. **Domain value** — value belonging to a persisted business entity.

Never scatter magic numbers, literal UI colors/hex values, or hardcoded user-facing strings inside components or handlers.

### 2.3. Modularity (No God Files or God Components)
Strictly prohibited:
- God files, God classes, God components, giant composables, giant Rust modules, excessively large functions.
- Business logic inside Vue components.
- Database queries inside Tauri command handlers or HTTP route handlers.
- Duplicated validation or persistence logic.

Split modules by domain responsibility:
- **Rust (`src-tauri/src/`)**: `application/`, `configuration/`, `domain/`, `infrastructure/`, `services/`, `commands/`, `events/`.
- **Frontend (`app/`)**: `components/` (by feature), `composables/`, `layouts/`, `pages/`, `stores/`, `types/`, `utils/`, `assets/css/`.

### 2.4. Rust Safety & Error Handling
- **Never** use `.unwrap()` or `.expect()` in production Rust paths without a documented, mathematically guaranteed invariant.
- Propagate typed domain and infrastructure errors with `thiserror` and map cleanly to safe, structured command/HTTP responses.
- Never log secrets, channel keys, or sensitive notification payloads.

### 2.5. TypeScript & Frontend Discipline
- TypeScript `strict: true` is mandatory.
- **Nuxt UI first**: Always check official Nuxt UI components before writing custom UI primitives.
- **Minimal custom CSS**: Use Nuxt UI props/variants, Tailwind v4 utilities, and centralized CSS tokens. Use CSS logical properties (`ms-*`, `me-*`, `ps-*`, `pe-*`, `start-*`, `end-*`, `text-start`, `text-end`) for automatic LTR/RTL symmetry.
- **HTML Email Sandbox**: Render captured email HTML only after backend sanitization inside a sandboxed `<iframe sandbox="" referrerpolicy="no-referrer">` with scripts and remote network requests blocked.

---

## 3. Internationalization (i18n) & Theme Matrix

- Mandatory initial locales: **English (`en`, `dir: 'ltr'`)** and **Persian (`fa`, `dir: 'rtl'`)**.
- Mandatory themes: **Light** and **Dark** (independently designed, never color-inverted).
- Every feature requires both `en` and `fa` translations and verification in both themes and both directions.
- Automated i18n validation must fail CI on missing keys, extra keys, empty values, mismatched interpolation placeholders, or invalid structure.

---

## 4. Testing Constitution

A feature is **not** complete when it compiles. Required coverage:
- **Table-driven tests** for validation, isolation, parsing, error mapping, and persistence edge cases.
- **Matrix tests** covering channels, workspaces/projects/environments, HTTP methods/payloads, SMTP/MIME cases, persistence, locales (`en`/`fa`), and themes (`light`/`dark`).
- **Property-based tests** (`proptest`) for normalization, identifier validation, MIME/payload parsing invariants, and serialization round-trips.
- **Real integration tests** starting actual local Axum HTTP and Tokio SMTP servers against isolated temporary SQLite databases (no arbitrary `sleep` calls).
- **Frontend & E2E tests** verifying rendering, validation, states (loading/empty/error), theme switching, and RTL layout.

---

## 5. Mandatory Project Documentation

Keep these documents synchronized with every architectural or behavioral change:
- `AGENTS.md` — Permanent engineering constitution
- `README.md` — Product overview, installation, development, usage
- `docs/ARCHITECTURE.md` — System architecture and module boundaries
- `docs/DECISIONS.md` — Architectural Decision Records (ADRs)
- `docs/DESIGN-SYSTEM.md` — Design token system and UI standards
- `docs/I18N.md` — Localization structure, RTL/LTR rules, CI validation
- `docs/TESTING.md` — Testing strategy, matrices, commands
- `docs/DEVELOPMENT.md` — Local environment, build, and contribution workflow

---

## 6. Definition of Done Checklist

Before marking any task complete, verify all applicable items:
1. Follows modular monolith architecture and layer boundaries.
2. Zero prohibited abbreviations in identifiers.
3. Zero unexplained hardcoded values or literal user-facing strings.
4. No God files or oversized components.
5. Nuxt UI used first; custom CSS justified if present.
6. Complete `en` and `fa` translations passing validation script.
7. Verified in RTL (`fa`) and LTR (`en`).
8. Verified in Light and Dark themes.
9. Loading, empty, and error states implemented.
10. Table-driven, matrix, property, or integration tests written and passing.
11. Formatting, linting, type checking, and test suites executed after final edits.
12. Documentation updated if contracts or behavior changed.
