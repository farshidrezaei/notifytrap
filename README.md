<p align="center">
  <img src="notifytrap-banner.png" alt="NotifyTrap Banner — Capture, Inspect, and Debug Notifications Locally" width="840" />
</p>

<h1 align="center">NotifyTrap</h1>

<p align="center">
  <strong>Local-first, offline-capable desktop utility for capturing, inspecting, and debugging application notifications—with zero real-world delivery.</strong>
</p>

<p align="center">
  <a href="https://farshidrezaei.github.io/notifytrap/">
    <img src="https://img.shields.io/badge/docs-GitHub%20Pages%20(EN%20%7C%20FA)-4f46e5" alt="Documentation" />
  </a>
  <a href="https://github.com/farshidrezaei/notifytrap/actions/workflows/release.yml">
    <img src="https://github.com/farshidrezaei/notifytrap/actions/workflows/release.yml/badge.svg" alt="Build and Release" />
  </a>
  <a href="https://github.com/farshidrezaei/notifytrap/releases">
    <img src="https://img.shields.io/github/v/release/farshidrezaei/notifytrap?display_name=tag&sort=semver" alt="GitHub Release" />
  </a>
  <img src="https://img.shields.io/badge/platforms-Linux%20%7C%20macOS%20%7C%20Windows-blue" alt="Platforms" />
  <img src="https://img.shields.io/badge/stack-Tauri%202%20%C2%B7%20Rust%20%C2%B7%20Nuxt%204%20%C2%B7%20SQLite-4f46e5" alt="Tech Stack" />
  <img src="https://img.shields.io/badge/i18n-English%20(LTR)%20%7C%20%D9%81%D8%A7%D8%B1%D8%B3%DB%8C%20(RTL)-0891b2" alt="Languages" />
  <a href="src-tauri/Cargo.toml">
    <img src="https://img.shields.io/badge/license-MIT-15803d" alt="MIT License" />
  </a>
</p>

---

## Table of Contents

- [Why NotifyTrap?](#why-notifytrap)
- [Core Capabilities](#core-capabilities)
- [CLI & Desktop Installation](#cli--desktop-installation)
  - [One-Command Installers](#one-command-installers)
  - [Manual CLI Commands by OS & Distro](#manual-cli-commands-by-os--distro)
  - [Signed In-App Auto-Updates](#signed-in-app-auto-updates)
- [Quick Start in 60 Seconds](#quick-start-in-60-seconds)
- [Transport & API Cookbook](#transport--api-cookbook)
- [Security & Privacy Model](#security--privacy-model)
- [Architecture Overview](#architecture-overview)
- [Run & Build from Source](#run--build-from-source)
- [Documentation](#documentation)

---

## Why NotifyTrap?

During local development and staging verification, applications frequently emit emails, SMS codes, push alerts, and outbound webhooks. Using production gateways risks accidental real delivery, while scraping terminal logs hides HTML rendering, MIME attachments, HTTP headers, and multi-environment isolation.

| Problem in Development | How NotifyTrap Solves It |
| --- | --- |
| Accidental emails/SMS sent to real users | Embedded ESMTP & HTTP servers bound strictly to `127.0.0.1` — **zero external delivery** |
| Hardcoded test secrets mixed across projects | Explicit `Workspace → Project → Environment → Channel` isolation with scoped credentials |
| Untrusted HTML emails or leaked auth tokens | Backend `ammonia` HTML sanitization, offline CSP iframe sandbox, and automatic `[REDACTED]` scrubbing |
| Switching tools for Email vs SMS/Push/Webhooks | Unified real-time Inbox inspecting **Email, SMS, Push, and Webhooks** in one native binary |
| Cloud SaaS mock traps & rate limits | 100% local SQLite persistence, instant startup, offline-first with zero external accounts |

---

## Core Capabilities

- **Four Local Capture Channels in One Engine**:
  - **Email (SMTP)**: Embedded Tokio ESMTP listener (`127.0.0.1:1025` default) supporting `AUTH PLAIN` and `AUTH LOGIN`, multipart `text/plain` + `text/html`, attachment preview/download, inline `cid:` hydration, responsive Desktop/Tablet/Mobile + Dark Client preview, offline Email Audit diagnostics, and raw MIME source.
  - **SMS (HTTP)**: `POST /api/v1/sms` capturing recipient, sender ID, message text, JSON metadata, GSM-7 / UCS-2 encoding diagnostics, and multi-segment counter.
  - **Push (HTTP)**: `POST /api/v1/push` capturing device token, platform, notification title/body, badge count, custom payload data, and iOS / Android / Web Push device previews.
  - **Webhook (HTTP)**: `ANY /api/v1/webhooks/{credential_identifier}[/{subpath}]` supporting every HTTP method (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`), subpaths, request headers, query strings, and JSON/raw request bodies.
- **Scoped Multi-Project Organization**: Organize captures across **Workspaces → Projects → Environments (`local`, `development`, `testing`, `staging`) → Channels**, each with its own generated `<credential_identifier>.<secret_token>`.
- **Real-Time Unified Inbox & Diff Inspector**: Live desktop event streaming (`notification:captured`), channel/status/run-tag (`X-NotifyTrap-Run`) filtering, automatic OTP & magic-link extraction, one-click `.eml` / redacted `JSON` / `cURL` export, side-by-side notification diffing, search, pagination, and syntax highlighting (`JSON`, `HTML`, `Bash`, `.ENV`, `MIME / HTTP`).
- **First-Class Bilingual & Dual-Theme UI**:
  - Native **English (`en`, LTR)** and **Persian (`fa`, RTL)** with logical CSS layout symmetry and bundled offline typography (`IBM Plex Sans`, `Vazirmatn`, `JetBrains Mono`).
  - Independent **Light** and **Dark** themes.
  - Automatic normalization of **Persian (`۰-۹`)**, **Arabic-Indic (`٠-٩`)**, and **Devanagari (`०-९`)** digits to English ASCII digits inside numeric inputs before backend persistence.
- **Signed Cross-Platform Updates**: Silent background release check on startup with actionable toast notifications (`Download & Update`, `Restart Now`) and manual update management in **Settings**.

---

## CLI & Desktop Installation

Prebuilt packages for Linux, macOS, and Windows are published via [GitHub Releases](https://github.com/farshidrezaei/notifytrap/releases).

### One-Command Installers

#### Linux & macOS (`bash`)

Auto-detects your OS (`Linux` / `macOS`), CPU architecture, and Linux package manager (`apt`/`dpkg` for `.deb`, `dnf`/`zypper`/`yum` for `.rpm`, or `.AppImage` fallback with desktop entry + CLI symlink):

```bash
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

Optional flags (`--method` / `--version`):

```bash
# Force AppImage on Ubuntu/Fedora (recommended if you want direct in-app self-updates on Linux)
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --method appimage

# Install a specific release tag
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --version v1.0.0
```

Supported `--method` values: `auto` (default), `deb`, `rpm`, `appimage`, `dmg`.

#### Windows (`PowerShell`)

Downloads the latest Windows installer (`.exe` NSIS by default, or `.msi`) from GitHub Releases and runs it cleanly:

```powershell
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

With options (for example silent `.msi` or pinned version):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1))) -Method msi -Silent
```

Supported parameters: `-Method auto|exe|msi`, `-Version latest|v1.0.0`, `-Silent`.

---

### Manual CLI Commands by OS & Distro

Prefer downloading the exact installer directly without piping a script? Use the commands below for your platform:

#### Debian / Ubuntu / Pop!_OS / Linux Mint (`.deb`)

```bash
ASSET_URL=$(curl -fsSL https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest \
  | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+\.deb"' \
  | head -n 1 | cut -d'"' -f4)
curl -fL "$ASSET_URL" -o /tmp/notifytrap.deb
sudo apt-get update && sudo apt-get install -y /tmp/notifytrap.deb
```

#### Fedora / RHEL / Rocky / Alma / openSUSE (`.rpm`)

```bash
ASSET_URL=$(curl -fsSL https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest \
  | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+\.rpm"' \
  | head -n 1 | cut -d'"' -f4)
curl -fL "$ASSET_URL" -o /tmp/notifytrap.rpm
sudo dnf install -y /tmp/notifytrap.rpm
# openSUSE: sudo zypper --non-interactive install /tmp/notifytrap.rpm
```

#### Arch Linux / Manjaro / Universal Linux (`.AppImage`)

```bash
mkdir -p ~/.local/bin ~/.local/share/notifytrap
ASSET_URL=$(curl -fsSL https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest \
  | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+\.AppImage"' \
  | grep -v '\.sig"' | head -n 1 | cut -d'"' -f4)
curl -fL "$ASSET_URL" -o ~/.local/share/notifytrap/NotifyTrap.AppImage
chmod +x ~/.local/share/notifytrap/NotifyTrap.AppImage
ln -sf ~/.local/share/notifytrap/NotifyTrap.AppImage ~/.local/bin/notifytrap
notifytrap
```

> [!TIP]
> On Linux, **AppImage** supports seamless cryptographic in-app self-updates. For `.deb` and `.rpm` installs, you will still receive update notifications and can upgrade by re-running the CLI installer or downloading the new package.

#### macOS — Apple Silicon & Intel (Universal `.dmg`)

```bash
ASSET_URL=$(curl -fsSL https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest \
  | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+\.dmg"' \
  | head -n 1 | cut -d'"' -f4)
curl -fL "$ASSET_URL" -o /tmp/NotifyTrap.dmg
MOUNT_DIR=$(hdiutil attach /tmp/NotifyTrap.dmg -nobrowse -readonly | awk '/\/Volumes\// { idx = index($0, "/Volumes/"); print substr($0, idx); exit }')
rm -rf /Applications/NotifyTrap.app && cp -R "$MOUNT_DIR/NotifyTrap.app" /Applications/
hdiutil detach "$MOUNT_DIR" -quiet && rm -f /tmp/NotifyTrap.dmg
```

#### Windows 10 / 11 (`.exe` / `.msi` via PowerShell)

```powershell
$asset = (Invoke-RestMethod https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest).assets `
  | Where-Object { $_.name -match '-setup\.exe$' } | Select-Object -First 1
$installer = Join-Path $env:TEMP $asset.name
Invoke-WebRequest $asset.browser_download_url -OutFile $installer
Start-Process $installer -Wait
```

---

### Signed In-App Auto-Updates

- Release artifacts are signed with minisign (`TAURI_SIGNING_PRIVATE_KEY`) and verified against the embedded public key in [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) before applying any update.
- When `app.auto_update_check_enabled` is enabled (on by default), NotifyTrap silently checks [`latest.json`](https://github.com/farshidrezaei/notifytrap/releases/latest/download/latest.json) shortly after startup and shows an interactive toast if a newer version exists.
- Manual checks and toggle preferences are available anytime under **Settings → Application Updates**.

---

## Quick Start in 60 Seconds

1. **Initialize Workspace & Project**: On first run, complete the onboarding card to create your first **Workspace** and **Project** (`local`, `development`, `testing`, and `staging` environments are created automatically).
2. **Create a Channel**: Navigate to **Channels**, select **Email**, **SMS**, **Push**, or **Webhook**, and copy the generated `<credential_identifier>` and `<secret_token>` (composite HTTP key: `<credential_identifier>.<secret_token>`).
3. **Configure Your App**: Point your development `.env`, mailer, SMS service, or webhook client to NotifyTrap's loopback listeners (`127.0.0.1:1025` SMTP / `127.0.0.1:4080` HTTP by default, customizable in **Settings**).
4. **Inspect Live**: Captured messages appear immediately in **Inbox** with unread counters, search, status controls (`unread` / `read` / `archived`), and channel-specific views.

---

## Transport & API Cookbook

### Default Local Endpoints

| Service / Endpoint | Default Address / Route | Authentication | Purpose |
| --- | --- | --- | --- |
| **ESMTP Server** | `127.0.0.1:1025` | SMTP `AUTH PLAIN` / `AUTH LOGIN` (`credential_identifier` / `secret_token`) | Captures local application emails & attachments |
| **Health Check** | `GET http://127.0.0.1:4080/api/v1/health` | Public (loopback only) | Returns runtime, database, and server readiness |
| **SMS Capture** | `POST http://127.0.0.1:4080/api/v1/sms` | `X-Channel-Key` or `Authorization: Bearer` | Captures SMS payloads |
| **Push Capture** | `POST http://127.0.0.1:4080/api/v1/push` | `X-Channel-Key` or `Authorization: Bearer` | Captures mobile/web push payloads |
| **Webhook Capture** | `ANY http://127.0.0.1:4080/api/v1/webhooks/{id}[/{subpath}]` | `X-Channel-Key`, `Authorization: Bearer`, or `?channel_key=` | Captures arbitrary HTTP webhook calls |
| **Query Notifications** | `GET http://127.0.0.1:4080/api/v1/notifications` | `X-Channel-Key` or `Authorization: Bearer` | Lists captured notifications scoped to the channel (supports `run_id`, `recipient`, `subject`, `since`, `status`, `search`) |
| **Wait for Notification** | `GET http://127.0.0.1:4080/api/v1/notifications/wait` | `X-Channel-Key` or `Authorization: Bearer` | Event-driven long-poll (`timeout_ms`, default `5000`, max `30000`) for E2E tests |
| **Notification Detail** | `GET http://127.0.0.1:4080/api/v1/notifications/{id}` | `X-Channel-Key` or `Authorization: Bearer` | Fetches full detail + extracted `verificationArtifacts` (OTP & magic links) |

Successful HTTP ingestion returns `202 Accepted` with `{ notificationId, channelId, status, receivedAt }`. Validation or credential failures return structured `{ code, message }` errors.

### SMTP Email Examples

#### Laravel / Symfony (`.env`)

```dotenv
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_USERNAME=email-local
MAIL_PASSWORD=ntsec_your_secret_token
MAIL_ENCRYPTION=null
MAIL_FROM_ADDRESS="noreply@local.test"
MAIL_FROM_NAME="NotifyTrap Local"
```

#### Node.js (`nodemailer`)

```js
import nodemailer from 'nodemailer'

const transporter = nodemailer.createTransport({
  host: '127.0.0.1',
  port: 1025,
  secure: false,
  auth: {
    user: 'email-local',
    pass: 'ntsec_your_secret_token',
  },
})

await transporter.sendMail({
  from: 'noreply@local.test',
  to: 'developer@example.com',
  subject: 'Verify your account',
  text: 'Verification code: 482-910',
  html: '<p>Verification code: <strong>482-910</strong></p>',
})
```

#### Python (`smtplib`)

```python
import smtplib
from email.message import EmailMessage

message = EmailMessage()
message["From"] = "noreply@local.test"
message["To"] = "developer@example.com"
message["Subject"] = "Password Reset"
message.set_content("Reset code: 913-204")

with smtplib.SMTP("127.0.0.1", 1025) as server:
    server.login("email-local", "ntsec_your_secret_token")
    server.send_message(message)
```

### HTTP SMS Capture (`POST /api/v1/sms`)

```bash
curl -X POST http://127.0.0.1:4080/api/v1/sms \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: sms-local.ntsec_your_secret_token" \
  -d '{
    "to": "+989120000000",
    "from": "NotifyApp",
    "message": "Your OTP code is 482-910",
    "metadata": { "purpose": "login" }
  }'
```

### HTTP Push Capture (`POST /api/v1/push`)

```bash
curl -X POST http://127.0.0.1:4080/api/v1/push \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer push-local.ntsec_your_secret_token" \
  -d '{
    "device_token": "fcm-device-token-123",
    "platform": "android",
    "title": "Invoice Paid",
    "body": "Order #1042 has been settled.",
    "badge": 1,
    "data": { "orderId": "1042" }
  }'
```

### HTTP Webhook Capture (`ANY /api/v1/webhooks/{credential_identifier}[/{subpath}]`)

```bash
curl -X POST "http://127.0.0.1:4080/api/v1/webhooks/billing-hook/stripe/events" \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: billing-hook.ntsec_your_secret_token" \
  -d '{
    "event": "invoice.paid",
    "amount": 4900
  }'
```

### Test Automation, Run Tagging & OTP Extraction

- Tag any HTTP capture (`POST /api/v1/sms`, `/push`, `/webhooks/*`) or SMTP email with header `X-NotifyTrap-Run: ci-run-42` to isolate and filter test runs in the Inbox or API.
- Query or long-poll captured messages without `sleep` in Playwright, Cypress, Pest, or Pytest:

```bash
# Wait up to 10s for a verification email/SMS tagged with run_id=ci-run-42
curl -fsS "http://127.0.0.1:4080/api/v1/notifications/wait?run_id=ci-run-42&timeout_ms=10000" \
  -H "X-Channel-Key: sms-local.ntsec_your_secret_token"
```

- Responses and Inbox cards automatically expose `verificationArtifacts` (extracted English/Persian OTP codes and verification/magic links) plus one-click `.eml`, `JSON`, and `cURL` export in the Inspector.

---

## Security & Privacy Model

- **Strict Loopback Binding**: HTTP and SMTP servers listen on `127.0.0.1` by default and never relay messages externally.
- **Encrypted Channel Keys & Secret Hashing**: Channel secrets are encrypted at rest with `AES-256-GCM` (backed by the OS keychain when available, with strict `0o600` atomic fallback key file), hashed via `SHA-256`, and verified in constant time.
- **Untrusted Email HTML Defense in Depth**:
  1. Backend `ammonia` HTML sanitization strips `<script>`, event handlers, `javascript:` URLs, and dangerous tags before storage and IPC delivery.
  2. Frontend renders email HTML inside a locked `<iframe sandbox="" referrerpolicy="no-referrer">` with injected CSP (`default-src 'none'`).
- **Automatic Credential Redaction**: Case-insensitive scrubbing replaces sensitive HTTP headers (`authorization`, `proxy-authorization`, `x-channel-key`, `x-api-key`, `cookie`, `set-cookie`) and query parameters (`channel_key`, `token`, `api_key`, `apikey`, `secret`, `password`, `authorization`) with `[REDACTED]`.
- **Resource & Retention Guards**: Bounded SMTP line length, 60-second SMTP idle read timeout, max 64 concurrent SMTP sessions, configurable HTTP payload caps, and automatic retention cleanup on startup and settings save.

---

## Architecture Overview

NotifyTrap follows a **modular monolith** architecture splitting presentation (Nuxt 4 + Nuxt UI + Tauri IPC) from Rust backend layers:

```mermaid
flowchart TD
  UI["Nuxt 4 UI (Vue 3 + Nuxt UI v4 + Tailwind v4)"] -->|Tauri IPC Commands| CMD["src-tauri/src/commands/"]
  UI <-->|tauri://notification:captured| EVT["src-tauri/src/events/"]
  EXT["Your App (Laravel / Node / Python / cURL)"] -->|SMTP :1025| SMTP["infrastructure/smtp/"]
  EXT -->|HTTP :4080| HTTP["infrastructure/http/"]
  CMD --> SVC["services/"]
  SMTP --> SVC
  HTTP --> SVC
  SVC --> DOM["domain/ & security/"]
  SVC --> DB["infrastructure/database/ (SQLite)"]
```

- **Backend (`src-tauri/src/`)**: `application/`, `commands/`, `configuration/`, `domain/`, `events/`, `infrastructure/` (`database/`, `http/`, `migrations/`, `security/`, `smtp/`), `services/`.
- **Frontend (`app/`)**: `components/`, `composables/`, `layouts/`, `pages/`, `schemas/`, `types/`, `utils/`, `assets/css/`.

---

## Run & Build from Source

### Prerequisites

- **Rust** stable toolchain (`rustc`, `cargo`, `rustfmt`, `clippy`)
- **Bun** (`>= 1.x`, matching `bun.lock`)
- **Platform Libraries**:
  - **Ubuntu / Debian**: `sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libssl-dev`
  - **Fedora**: `sudo dnf install -y webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel`
  - **Arch Linux**: `sudo pacman -S --needed webkit2gtk-4.1 libappindicator-gtk3 librsvg openssl`
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Windows**: Microsoft C++ Build Tools + WebView2 Runtime

### Development & Build Commands

```bash
# Install frontend dependencies
bun install

# Launch desktop app in dev mode (starts Rust backend + Nuxt UI)
bun run tauri dev

# Build distributable bundles for the current host OS
bun run tauri build

# Or build binary only without OS installers
bun run tauri build --no-bundle
```

### Verification Suite

```bash
# Frontend quality gates
bun run validate:i18n
bun run typecheck
bun run test:unit

# Backend quality gates & integration tests
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

---

## Documentation

- **Online Documentation (English & فارسی RTL):** [farshidrezaei.github.io/notifytrap](https://farshidrezaei.github.io/notifytrap/)
- [Engineering Constitution (`AGENTS.md`)](AGENTS.md)
- [Architecture & Contracts](docs/ARCHITECTURE.md)
- [Architectural Decisions (ADRs)](docs/DECISIONS.md)
- [Design System & Tokens](docs/DESIGN-SYSTEM.md)
- [Internationalization & RTL Guide](docs/I18N.md)
- [Testing Matrix & Strategy](docs/TESTING.md)
- [Development, CI Signing & Release Guide](docs/DEVELOPMENT.md)
