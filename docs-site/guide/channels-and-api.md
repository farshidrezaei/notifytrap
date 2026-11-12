# Channels, SMTP & HTTP API

Every captured message is strictly scoped to a single **Channel** belonging to one **Workspace → Project → Environment**.

## Default Local Endpoints

| Service / Route | Default Address | Authentication | Behavior |
| --- | --- | --- | --- |
| **ESMTP Server** | `127.0.0.1:1025` | `AUTH PLAIN` or `AUTH LOGIN` | Captures emails & attachments locally |
| **Health Check** | `GET http://127.0.0.1:4080/api/v1/health` | None (loopback) | Returns runtime, DB & listener state |
| **SMS Capture** | `POST http://127.0.0.1:4080/api/v1/sms` | `X-Channel-Key` or `Authorization: Bearer` | Captures SMS JSON payload |
| **Push Capture** | `POST http://127.0.0.1:4080/api/v1/push` | `X-Channel-Key` or `Authorization: Bearer` | Captures Push JSON payload |
| **Webhook Trap** | `ANY http://127.0.0.1:4080/api/v1/webhooks/{id}[/{subpath}]` | Header or `?channel_key=` | Captures arbitrary HTTP requests |
| **Query Inbox** | `GET http://127.0.0.1:4080/api/v1/notifications` | `X-Channel-Key` or `Authorization: Bearer` | Lists channel-scoped captures (`run_id`, `recipient`, `subject`, `since`, `status`, `search`) |
| **Wait for Capture** | `GET http://127.0.0.1:4080/api/v1/notifications/wait` | `X-Channel-Key` or `Authorization: Bearer` | Event-driven long-poll (`timeout_ms`, max `30000`) for E2E CI assertions |
| **Capture Detail** | `GET http://127.0.0.1:4080/api/v1/notifications/{id}` | `X-Channel-Key` or `Authorization: Bearer` | Returns full payload plus extracted `verificationArtifacts` (OTP & magic links) |

All HTTP capture routes return **`202 Accepted`** with `{ notificationId, channelId, status, receivedAt }` on success and structured `{ code, message }` JSON errors on invalid credentials or malformed payloads. Pass `X-NotifyTrap-Run: <run-id>` on HTTP requests or SMTP messages to tag and filter notifications per automated test suite run.

---

## Test Automation Query & Wait API

Synchronize Playwright, Cypress, Pest, or Pytest suites directly against local captures without `sleep` loops:

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

## Rich Inspector Capabilities (`v1.2.0`)

- **Email Inspector Pro**: Responsive **Desktop**, **Tablet (`768px`)**, and **Mobile (`375px`)** viewport toggles, opt-in **Dark Client** simulation, inline `cid:` image hydration, safe raster/text attachment previews + downloads, offline **Audit** checks (HTML payload size, subject, plain-text fallback, missing image `alt`, blocked trackers, `List-Unsubscribe`, relative links), and one-click `.eml` export.
- **OTP & Diff Compare**: Instant copy pills for detected OTP codes (normalized across English, Persian, and Arabic-Indic numerals) and magic links, side-by-side notification **Compare** diffing (formatted payload, raw payload, metadata), plus redacted `JSON` and secret-safe `cURL` exports.
- **SMS & Push Tools**: `GSM-7` vs `UCS-2` segment calculator and mobile SMS bubble mockup alongside **iOS**, **Android**, and **Web Push** notification previews.

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
  to: 'developer@example.com',
  subject: 'Verify your account',
  text: 'Verification code: 482-910',
  html: '<p>Verification code: <strong>482-910</strong></p>',
})
```

```python [Python (smtplib)]
import smtplib
from email.message import EmailMessage

msg = EmailMessage()
msg["From"] = "noreply@local.test"
msg["To"] = "developer@example.com"
msg["Subject"] = "Password Reset"
msg.set_content("Reset code: 913-204")

with smtplib.SMTP("127.0.0.1", 1025) as server:
    server.login("mail-local", "ntsec_your_secret_token")
    server.send_message(msg)
```

:::

---

## HTTP Channel Examples

::: code-group

```bash [SMS (POST /api/v1/sms)]
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

```bash [Push (POST /api/v1/push)]
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

```bash [Webhook (ANY /api/v1/webhooks/{id})]
curl -X POST http://127.0.0.1:4080/api/v1/webhooks/billing-hook/stripe/events \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: billing-hook.ntsec_your_secret_token" \
  -d '{
    "event": "invoice.paid",
    "amount": 4900
  }'
```

:::

For webhook integrations that only allow URL-based secrets, pass `?channel_key=<credential_identifier>.<secret_token>` (or `?channel_key=<secret_token>` when `{credential_identifier}` is in the URL path). Sensitive query parameters and headers are redacted before storage.

---

## Webhook Mocks, Loopback Replay & Chaos Rules (`v1.3.0`)

- **Webhook Mock Responses**: Configure per-channel custom HTTP status (`100..599`), response headers, response body, and delay (`0..10000 ms`) from **Channels → Rules**. NotifyTrap persists the incoming capture first, then returns your mock response.
- **Strict Loopback-Only Replay**: Replay any captured webhook from **Inbox → Replay** exclusively to local targets (`http://127.0.0.1`, `http://localhost`, or `http://[::1]`). External hosts, userinfo, and fragments are rejected.
- **Chaos Rules**: Inject artificial latency (`0..10000 ms`) and probabilistic failures (`0..100%`) with custom HTTP (`400..599`) or SMTP (`400..599`) error codes on any channel to verify retry and backoff handling locally.
