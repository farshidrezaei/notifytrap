# Channels, SMTP & HTTP API

Every captured message is strictly scoped to a single **Channel** belonging to one **Workspace → Project → Environment**.

## Default Local Endpoints

| Service / Route | Default Address | Authentication | Behavior |
| --- | --- | --- | --- |
| **ESMTP Server** | `127.0.0.1:1025` | `AUTH PLAIN` or `AUTH LOGIN` | Captures emails & attachments locally |
| **HTTP Capture API** | `127.0.0.1:4080` | `X-Channel-Key` or `Authorization: Bearer` | Ingestion, query, wait & management APIs |
| **Web Companion UI** | `127.0.0.1:4000` | Optional Loopback / Network | Full Nuxt SPA dashboard in headless mode (`--web-port`) |
| **Health Check** | `GET http://127.0.0.1:4080/api/v1/health` | None (loopback) | Returns runtime, DB, storage & listener state |
| **SMS Capture** | `POST http://127.0.0.1:4080/api/v1/sms` | `X-Channel-Key` | Captures SMS JSON payload with GSM-7 analysis |
| **Push Capture** | `POST http://127.0.0.1:4080/api/v1/push` | `X-Channel-Key` | Captures Push JSON payload with device simulators |
| **Webhook Trap** | `ANY http://127.0.0.1:4080/api/v1/webhooks/{id}[/{subpath}]` | Header or `?channel_key=` | Captures arbitrary HTTP methods & bodies |
| **Query Inbox** | `GET http://127.0.0.1:4080/api/v1/notifications` | `X-Channel-Key` | Lists captures with `run_id`, `recipient`, `search`, etc. |
| **Wait for Capture** | `GET http://127.0.0.1:4080/api/v1/notifications/wait` | `X-Channel-Key` | Event-driven long-poll (`timeout_ms`, max `30000`) for E2E tests |
| **Capture Detail** | `GET http://127.0.0.1:4080/api/v1/notifications/{id}` | `X-Channel-Key` | Returns payload, headers, links & extracted OTP tokens |

All HTTP capture routes return **`202 Accepted`** with `{ notificationId, channelId, status, receivedAt }` on success and structured `{ code, message }` JSON errors on invalid credentials or malformed payloads.

---

## Test Automation Query & Wait API

Synchronize Playwright, Cypress, Pest, or Pytest suites directly against local captures without brittle `sleep` loops:

```bash
# Send an SMS tagged with a CI run identifier
curl -X POST http://127.0.0.1:4080/api/v1/sms \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: sms-local.ntsec_your_secret_token" \
  -H "X-NotifyTrap-Run: e2e-signup-42" \
  -d '{"to":"+15550100","from":"Auth","message":"Your verification code is 482-910"}'

# Wait instantaneously for the matching notification and inspect verificationArtifacts
curl "http://127.0.0.1:4080/api/v1/notifications/wait?run_id=e2e-signup-42&timeout_ms=5000" \
  -H "X-Channel-Key: sms-local.ntsec_your_secret_token"
```

---

## Channel Rules: Mocking, Chaos & Auto-Forwarding

Under **Channels → Rules**, you can configure advanced behaviors per channel:

### 1. Webhook Mock Responses
- Configure arbitrary HTTP status codes (`100..599`), custom response headers (up to 32 headers), and response body templates (JSON, XML, or plain text up to 64 KiB).
- Add artificial response delays (`0..10000 ms`) to simulate network latency.
- Incoming captures are securely persisted in SQLite first before returning the mock response.

### 2. HTTP & SMTP Chaos Injection
- Simulate real-world network failure conditions locally.
- Configure artificial latency and probabilistic failure rates (`0..100%`) with custom HTTP or SMTP error status codes.
- Tests your client retry and exponential backoff mechanisms under controlled conditions.

### 3. Downstream Webhook Auto-Forwarding
- Automatically forward incoming webhooks to secondary URLs (e.g. Zapier, webhook.site, local microservices).
- Filter by event name: only forward when `event`, `type`, or `event_name` matches your rule.
- Append custom authorization headers (e.g. `Authorization: Bearer <token>`).
- Non-blocking async execution: captured requests are recorded locally regardless of downstream availability.

---

## Email (SMTP) Integration

Point your framework's SMTP configuration to `127.0.0.1:1025` using **SMTP Username (`credentialIdentifier`)** and **SMTP Password (`secretToken`)**. No TLS is required on loopback.

::: code-group

```dotenv [Laravel / Symfony (.env)]
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_USERNAME=mail-local
MAIL_PASSWORD=ntsec_your_secret_token
MAIL_ENCRYPTION=null
MAIL_FROM_ADDRESS="noreply@local.test"
MAIL_FROM_NAME="NotifyTrap Local"
```

```js [Node.js (nodemailer)]
import nodemailer from 'nodemailer'

const transporter = nodemailer.createTransport({
  host: '127.0.0.1',
  port: 1025,
  secure: false,
  auth: {
    user: 'mail-local',
    pass: 'ntsec_your_secret_token',
  },
})

await transporter.sendMail({
  from: 'noreply@local.test',
  to: 'user@example.test',
  subject: 'Welcome to NotifyTrap',
  html: '<p>Testing local email delivery</p>',
})
```

```python [Python (smtplib)]
import smtplib
from email.message import EmailMessage

msg = EmailMessage()
msg.set_content("Testing local email capture")
msg["Subject"] = "Local Test"
msg["From"] = "noreply@local.test"
msg["To"] = "user@example.test"

with smtplib.SMTP("127.0.0.1", 1025) as server:
    server.login("mail-local", "ntsec_your_secret_token")
    server.send_message(msg)
```

:::
