---
title: کانال‌ها و API دریافت
---

# کانال‌ها و API دریافت

هر نوتیفیکیشن دریافتی از طریق **اعتبارنامه کانال (Channel Credential)** مستقیماً به همان `Workspace`، `Project` و `Environment` متصل می‌شود.

## ساختار کلید و اعتبارنامه

هنگام ساخت یک کانال یا چرخش (Rotate) کلید، کلید ترکیبی با قالب زیر ایجاد می‌شود:

```text
<credential_identifier>.<secret>
```

برای نمونه در یک کانال ایمیل:

- **SMTP Username (`MAIL_USERNAME`):** `mail-947ce3807353e6f5`
- **SMTP Password (`MAIL_PASSWORD`):** `ntsec_...`
- **کلید ترکیبی (`X-Channel-Key`):** `mail-947ce3807353e6f5.ntsec_...`

> [!WARNING] نگهداری امن کلیدها
> NotifyTrap کلیدهای شما را به صورت متن ساده در دیتابیس ذخیره نمی‌کند. `Hash` کلید با SHA-256 برای تطبیق و مقدار قابل نمایش با **AES-256-GCM** رمزنگاری می‌شود تا قابلیت نمایش (`Reveal`) با کلید امن Keyring سیستم‌عامل کار کند.

---

## ۱. دریافت ایمیل با SMTP

اپلیکیشن خود را به شنونده `127.0.0.1:1025` متصل کنید. نمونه تنظیم `.env` در لاراول یا Symfony Mailer:

```dotenv
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_USERNAME=mail-947ce3807353e6f5
MAIL_PASSWORD=ntsec_your_secret_here
MAIL_ENCRYPTION=null
```

قابلیت‌های پشتیبانی‌شده در سرور SMTP:

- احراز هویت `AUTH PLAIN` و `AUTH LOGIN`
- تجزیه پیام‌های `MIME` تودرتو (`multipart/alternative`، `multipart/mixed`)
- استخراج هم‌زمان متن ساده (`text/plain`)، نسخه HTML پالایش‌شده و فهرست پیوست‌ها

---

## ۲. دریافت HTTP (SMS، Push و Webhook)

### Endpoint اصلی دریافت

```http
POST http://127.0.0.1:4080/ingest/{channel_type}
```

مقادیر معتبر برای `{channel_type}` عبارت‌اند از:
- `sms`
- `push`
- `webhook`

### هدر احراز هویت

هدر زیر را به همراه کلید ترکیبی کامل ارسال کنید:

```http
X-Channel-Key: sms-7feaad9993df8b9.ntsec_your_secret_here
Content-Type: application/json
```

### نمونه درخواست cURL

::: code-group

```bash [SMS]
curl -X POST http://127.0.0.1:4080/ingest/sms \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: sms-7feaad9993df8b9.ntsec_your_secret_here" \
  -d '{
    "to": "+989121234567",
    "from": "NotifyApp",
    "message": "کد تایید شما: 482910"
  }'
```

```bash [Push Notification]
curl -X POST http://127.0.0.1:4080/ingest/push \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: push-12ab34cd56ef.ntsec_your_secret_here" \
  -d '{
    "title": "به‌روزرسانی سفارش",
    "body": "بسته شما ارسال شد.",
    "deviceToken": "fcm-demo-token"
  }'
```

```bash [Webhook]
curl -X POST http://127.0.0.1:4080/ingest/webhook \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: whk-98zy76xw54vu.ntsec_your_secret_here" \
  -d '{
    "event": "invoice.paid",
    "invoiceId": "INV-2026-009"
  }'
```

:::

### بررسی وضعیت سرویس (Health Check)

```bash
curl http://127.0.0.1:4080/health
```

پاسخ `200 OK` حاوی وضعیت سلامت شنونده HTTP است.
