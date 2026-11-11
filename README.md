<p align="center">
  <img src="notifytrap-logo.png" alt="NotifyTrap Logo" width="112" />
</p>

# NotifyTrap

**Capture, inspect, and debug application notifications in one local desktop workspace.**

NotifyTrap is a lightweight, local-first, offline-capable, open-source desktop application (Tauri 2 + Rust + Nuxt 4 + Nuxt UI + SQLite) for capturing and inspecting development notifications across **Email (SMTP)**, **SMS (HTTP)**, **Push (HTTP)**, and **Webhooks (HTTP)**—without ever sending real messages to external recipients.

---

## Key Features

- **Local-First & Zero Real Delivery**: Single desktop binary with embedded SQLite, Tokio SMTP server, and Axum HTTP server bound to `127.0.0.1` by default.
- **Workspace Isolation**: Explicit `Workspace → Project → Environment (Local / Development / Testing / Staging) → Channel` hierarchy with isolated per-channel credentials (`AES-256-GCM` + `SHA-256`).
- **Four Capture Channels**:
  - **Email**: Local ESMTP server (`AUTH PLAIN` / `AUTH LOGIN`, multipart MIME, attachments, backend `ammonia` HTML sanitization, offline CSP sandbox iframe + raw source).
  - **SMS**: Local `POST /api/v1/sms` JSON capture endpoint.
  - **Push**: Local `POST /api/v1/push` JSON capture endpoint.
  - **Webhook**: Local `ANY /api/v1/webhooks/{credential_identifier}` and subpaths capturing HTTP method, path, redacted headers/query parameters, JSON/raw body, and payload size.
- **Unified Inbox & Channel Inspector**: Real-time `notification:captured` streaming updates, channel/status filters, search, pagination, syntax-highlighted payloads (`JSON`, `HTML`, `Bash`, `.ENV`, `MIME / HTTP`), and tailored inspectors per channel.
- **Bilingual & Dual-Theme**: First-class **English (`en`, LTR)** and **Persian (`fa`, RTL)** support with Light/Dark themes and automatic Persian/Arabic-Indic/Hindi digit normalization to English ASCII before backend submission.

---

## Quick Start & Installation

### Prebuilt Desktop Packages

GitHub Releases (automated via [`.github/workflows/release.yml`](.github/workflows/release.yml)) provide installers for all major desktop platforms:

- **Linux**: `.deb` (Debian / Ubuntu / Mint / Pop!_OS), `.rpm` (Fedora / RHEL / openSUSE), `.AppImage` (Arch / Manjaro / universal Linux)
- **Windows**: `.exe` (NSIS installer) and `.msi`
- **macOS**: Universal `.dmg` (Apple Silicon & Intel)

Packaged builds include cryptographically signed in-app updates with silent background detection and toast actions (supported directly on Windows, macOS, and Linux `.AppImage`).

### Prerequisites

- **Rust** stable toolchain (`rustc >= 1.85`, `cargo`, `rustfmt`, `clippy`)
- **Bun** (`>= 1.x`, recommended matching `bun.lock`) or Node.js `>= 22`
- **Tauri 2 Linux System Libraries**: `webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `librsvg`, `openssl`, `pkg-config`

### Run from Source

```bash
# 1. Install frontend dependencies
bun install

# 2. Launch Tauri 2 desktop application in development mode
bun run tauri dev

# 3. Build release desktop binary (or omit --no-bundle where OS packagers are installed)
bun run tauri build --no-bundle
```

---

## Usage Workflow

1. Launch **NotifyTrap** — on first run, the onboarding card creates your initial **Workspace** and **Project** (auto-provisioning `local`, `development`, `testing`, and `staging` environments).
2. Open **Channels**, create a channel (`Email`, `SMS`, `Push`, or `Webhook`), and copy or reveal its generated `<credential_identifier>` and `<secret_token>` (composite key: `<credential_identifier>.<secret_token>`).
3. Point your local application's mail or notification transport to NotifyTrap's loopback endpoints (`127.0.0.1:1025` SMTP, `127.0.0.1:4080` HTTP by default, configurable in **Settings**).
4. Inspect captured messages live inside **Inbox**.

---

## SMTP Configuration Examples

Use the **Email** channel's `credential_identifier` as SMTP username and `secret_token` as SMTP password against `127.0.0.1:1025` (no TLS required on loopback; supports `AUTH PLAIN` and `AUTH LOGIN`).

### `.env` (Laravel / Symfony / Generic)

```dotenv
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_USERNAME=your-email-channel-identifier
MAIL_PASSWORD=ntsec_your_secret_token
MAIL_ENCRYPTION=null
MAIL_FROM_ADDRESS="noreply@local.test"
MAIL_FROM_NAME="NotifyTrap Local"
```

### Node.js (`nodemailer`)

```js
import nodemailer from 'nodemailer'

const transporter = nodemailer.createTransport({
  host: '127.0.0.1',
  port: 1025,
  secure: false,
  auth: {
    user: 'your-email-channel-identifier',
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

---

## Local HTTP API Examples

All HTTP capture endpoints return `202 Accepted` with `{ notificationId, channelId, status, receivedAt }` on success and structured JSON errors (`{ code, message }`) on validation or auth failure.

### 1. Health Check (`GET /api/v1/health`)

```bash
curl -s http://127.0.0.1:4080/api/v1/health
```

### 2. SMS Capture (`POST /api/v1/sms`)

Authenticate using `X-Channel-Key: <credential_identifier>.<secret_token>` or `Authorization: Bearer <credential_identifier>.<secret_token>`.

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

### 3. Push Capture (`POST /api/v1/push`)

```bash
curl -X POST http://127.0.0.1:4080/api/v1/push \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: push-local.ntsec_your_secret_token" \
  -d '{
    "device_token": "fcm-device-token-123",
    "platform": "android",
    "title": "Invoice Paid",
    "body": "Order #1042 has been settled.",
    "badge": 1,
    "data": { "orderId": "1042" }
  }'
```

### 4. Webhook Capture (`ANY /api/v1/webhooks/{credential_identifier}[/{subpath}]`)

Supports any HTTP method (`GET`, `POST`, `PUT`, `PATCH`, `DELETE`). Authenticate via `X-Channel-Key`, `Authorization: Bearer`, or query parameter `?channel_key=<key>` (sensitive headers/query secrets are automatically redacted to `[REDACTED]` in stored captures).

```bash
curl -X POST http://127.0.0.1:4080/api/v1/webhooks/billing-hook/stripe/events \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: billing-hook.ntsec_your_secret_token" \
  -d '{
    "event": "invoice.paid",
    "amount": 4900
  }'
```

---

## Documentation

- [Engineering Constitution](AGENTS.md)
- [Architecture & Contracts](docs/ARCHITECTURE.md)
- [Architectural Decisions](docs/DECISIONS.md)
- [Design System](docs/DESIGN-SYSTEM.md)
- [Internationalization](docs/I18N.md)
- [Testing Strategy](docs/TESTING.md)
- [Development, Contribution & Release Guide](docs/DEVELOPMENT.md)
