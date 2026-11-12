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

### Endpointهای اصلی دریافت

```http
POST http://127.0.0.1:4080/api/v1/sms
POST http://127.0.0.1:4080/api/v1/push
ANY  http://127.0.0.1:4080/api/v1/webhooks/{credential_identifier}[/{subpath}]
```

### هدر احراز هویت و تگ تست

هدر احراز هویت را به همراه کلید ترکیبی کامل (یا `Authorization: Bearer`) ارسال کنید؛ برای برچسب‌گذاری تست‌های خودکار از `X-NotifyTrap-Run` هم روی HTTP و هم روی ایمیل SMTP استفاده کنید:

```http
X-Channel-Key: sms-7feaad9993df8b9.ntsec_your_secret_here
X-NotifyTrap-Run: e2e-signup-42
Content-Type: application/json
```

### نمونه درخواست cURL

::: code-group

```bash [SMS]
curl -X POST http://127.0.0.1:4080/api/v1/sms \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: sms-7feaad9993df8b9.ntsec_your_secret_here" \
  -H "X-NotifyTrap-Run: e2e-signup-42" \
  -d '{
    "to": "+989121234567",
    "from": "NotifyApp",
    "message": "کد تایید شما: 482910"
  }'
```

```bash [Push Notification]
curl -X POST http://127.0.0.1:4080/api/v1/push \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: push-12ab34cd56ef.ntsec_your_secret_here" \
  -d '{
    "title": "به‌روزرسانی سفارش",
    "body": "بسته شما ارسال شد.",
    "platform": "android",
    "badge": 1,
    "deviceToken": "fcm-demo-token"
  }'
```

```bash [Webhook]
curl -X POST http://127.0.0.1:4080/api/v1/webhooks/whk-98zy76xw54vu/stripe/events \
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
curl http://127.0.0.1:4080/api/v1/health
```

پاسخ `200 OK` حاوی وضعیت سلامت شنونده‌های HTTP و SMTP و دیتابیس است.

---

## ۳. API کوئری، انتظار رویدادمحور و اتوماسیون تست

برای تست‌های E2E (Playwright، Cypress، Pest، Pytest) بدون نیاز به `sleep` مستقیم به پیام‌های همان کانال دسترسی داشته باشید:

| مسیر | کاربرد | فیلترها و جزئیات |
| --- | --- | --- |
| `GET /api/v1/notifications` | فهرست اعلان‌های کانال | پشتیبانی از `run_id`، `recipient`، `subject`، `since`، `status`، `search`، `limit`، `offset` |
| `GET /api/v1/notifications/wait` | انتظار رویدادمحور (`long-poll`) | `timeout_ms` (پیش‌فرض `5000`، حداکثر `30000`) و بازگرداندن فوری پیام جدید یا `408 WAIT_TIMEOUT` |
| `GET /api/v1/notifications/{id}` | دریافت جزئیات کامل و OTP | بازگرداندن جزئیات پیام + آرایه `verificationArtifacts` (کدهای OTP و لینک‌های تایید) با ایزولاسیون کامل کانال (`404` روی ID سایر کانال‌ها) |

```bash
curl "http://127.0.0.1:4080/api/v1/notifications/wait?run_id=e2e-signup-42&timeout_ms=5000" \
  -H "X-Channel-Key: sms-7feaad9993df8b9.ntsec_your_secret_here"
```

---

## ۴. ابزارهای پیشرفته بررسی و دیباگ (`v1.2.0`)

- **بازرس پیشرفته ایمیل**: دکمه‌های نمایش **دسکتاپ / تبلت / موبایل**، شبیه‌سازی **کلاینت تیره**، رندر محلی تصاویر درون‌خطی `cid:`، پیش‌نمایش امن پیوست‌های متنی و تصویری و دانلود فایل، به همراه تب **ممیزی (Audit)** (حجم پیام، عنوان، نسخه Plain Text، صفت `alt` تصاویر، لینک‌های نسبی، `List-Unsubscribe` و پیکسل‌های ریموت مسدودشده).
- **ابزارهای OTP، مقایسه و خروجی**: کپی سریع کدهای OTP (با نرمال‌سازی اعداد فارسی/عربی/انگلیسی) و لینک‌های جادویی، مودال **مقایسه (Compare)** برای نمایش تفاوت خط‌به‌خط بین دو اعلان، و منوی Export برای خروجی `.eml`، `JSON` (با سانسور سکرت) و دستور `cURL`.
- **پیامک و پوش نوتیفیکیشن**: شمارنده دقیق بخش‌ها و رمزنگاری پیامک (`GSM-7` و `UCS-2`) و شبیه‌ساز کارت پوش در **iOS**، **Android** و **Web Push**.

---

## ۵. پاسخ Mock وب‌هوک، Replay لوکال و قوانین Chaos (`v1.3.0`)

- **پاسخ Mock وب‌هوک**: از بخش **کانال‌ها ← قوانین (Rules)** می‌توانید برای هر کانال Webhook کد وضعیت (`100..599`)، هدرها، بدنه پاسخ و تاخیر (`0..10000 ms`) دلخواه تعریف کنید. NotifyTrap ابتدا درخواست را ذخیره و سپس پاسخ Mock را برمی‌گرداند.
- **بازپخش سخت‌گیرانه لوکال (Loopback-Only Replay)**: هر وب‌هوک شکارشده را از طریق **صندوق دریافت ← بازپخش (Replay)** فقط به مقصدهای محلی (`http://127.0.0.1`، `http://localhost` یا `http://[::1]`) بازارسال کنید. هاست‌های خارجی مسدود می‌شوند.
- **قوانین Chaos**: روی هر کانال تاخیر مصنوعی (`0..10000 ms`) و نرخ خطای احتمالی (`0..100%`) با کد خطای دلخواه HTTP یا SMTP (`400..599`) اعمال کنید تا مکانیزم Retry برنامه را بدون ریسک تست کنید.
