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

All HTTP capture routes return **`202 Accepted`** with `{ notificationId, channelId, status, receivedAt }` on success and structured `{ code, message }` JSON errors on invalid credentials or malformed payloads.

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
