# کانال‌ها، SMTP و HTTP API

هر پیام شکار شده در NotifyTrap به صورت دقیق به یک **کانال (Channel)** متعلق به سلسله‌مراتب **Workspace → Project → Environment** متصل است.

## آدرس‌های محلی پیش‌فرض سرویس‌ها

| سرویس / مسیر | آدرس پیش‌فرض | احراز هویت | عملکرد |
| --- | --- | --- | --- |
| **سرور ESMTP** | `127.0.0.1:1025` | `AUTH PLAIN` یا `AUTH LOGIN` | شکار ایمیل‌ها و پیوست‌ها روی لوکال |
| **API دریافت و کنترل** | `127.0.0.1:4080` | هدر `X-Channel-Key` یا `Bearer` | مسیرهای دریافت، جستجو، لاگ و مدیریت |
| **رابط وب Companion** | `127.0.0.1:4000` | اختیاری (پیش‌فرض Loopback) | داشبورد کامل Nuxt SPA در حالت سرور هدلس (`--web-port`) |
| **بررسی وضعیت سلامت** | `GET http://127.0.0.1:4080/api/v1/health` | بدون نیاز به کلید (لوپ‌بک) | گزارش وضعیت موتور، دیتابیس، حجم و پورت‌ها |
| **شکار پیامک (SMS)** | `POST http://127.0.0.1:4080/api/v1/sms` | هدر `X-Channel-Key` | دریافت پیامک با شمارش بخش‌های GSM-7 |
| **شکار پوش (Push)** | `POST http://127.0.0.1:4080/api/v1/push` | هدر `X-Channel-Key` | دریافت نوتیفیکیشن موبایل و وب |
| **تله وب‌هوک** | `ANY http://127.0.0.1:4080/api/v1/webhooks/{id}[/{subpath}]` | هدر یا کوئری `?channel_key=` | دریافت هر متد HTTP و هر بادی دلخواه |
| **کوئری اینباکس** | `GET http://127.0.0.1:4080/api/v1/notifications` | هدر `X-Channel-Key` | لیست پیام‌های کانال با فیلتر شناسه تست و وضعیت |
| **انتظار برای رویداد** | `GET http://127.0.0.1:4080/api/v1/notifications/wait` | هدر `X-Channel-Key` | لانگ‌پولینگ آنی (`timeout_ms`) برای تست‌های CI |
| **جزئیات پیام** | `GET http://127.0.0.1:4080/api/v1/notifications/{id}` | هدر `X-Channel-Key` | بازگرداندن پیلود کامل، پیوست‌ها و کدهای OTP |

کلیه متدهای دریافت HTTP در صورت موفقیت کد **`202 Accepted`** به همراه شیء شامل شناسه اعلان، کانال و زمان دریافت بازمی‌گردانند.

---

## هماهنگی تست‌های خودکار (Query & Wait API)

برای اجرای تست‌های یکپارچگی ابزارهایی مانند Pest، Cypress، Playwright یا Pytest بدون ایجاد حلقه‌های تاخیر (`sleep`):

```bash
# ارسال یک پیامک با شناسه اجرای تست (Run ID)
curl -X POST http://127.0.0.1:4080/api/v1/sms \
  -H "Content-Type: application/json" \
  -H "X-Channel-Key: sms-local.ntsec_your_secret_token" \
  -H "X-NotifyTrap-Run: e2e-signup-42" \
  -d '{"to":"+989120000000","from":"Auth","message":"کد ورود شما: ۴۸۲-۹۱۰"}'

# انتظار آنی برای پیام مطابق با این شناسه تست
curl "http://127.0.0.1:4080/api/v1/notifications/wait?run_id=e2e-signup-42&timeout_ms=5000" \
  -H "X-Channel-Key: sms-local.ntsec_your_secret_token"
```

---

## قوانین کانال: ماک، آشوب و فوروارد خودکار

در تب **کانال‌ها → قوانین (Rules)** می‌توانید تنظیمات پیشرفته زیر را برای هر کانال فعال کنید:

### ۱. پاسخ‌های ماک وب‌هوک (Mock Responses)
- تنظیم کدهای وضعیت دلخواه HTTP (`100..599`)، هدرهای سفارشی و بادی پاسخ (JSON یا متن تا ۶۴ کیلوبایت).
- اعمال تاخیر ساختگی برای شبیه‌سازی لیتنسی واقعی شبکه.
- درخواست ابتدا در SQLite ذخیره شده و سپس پاسخ ماک ارسال می‌گردد.

### ۲. تزریق آشوب (Chaos Rules)
- شبیه‌سازی شرایط بحرانی شبکه و قطعی سرور.
- تعیین درصد شکست تصادفی (`0..100%`) و خطای سفارشی جهت سنجش منطق retry در کلاینت‌های شما.

### ۳. فوروارد خودکار وب‌هوک به سرویس‌های ثانویه (Auto-Forwarding)
- ارسال خودکار وب‌هوک‌های شکارشده به URL دیگر (مانند Zapier یا سرویس‌های محلی دیگر).
- اعمال فیلتر بر اساس نام رویداد (مثلاً فقط رویدادهای `order.created`).
- افزودن هدرهای احراز هویت سفارشی بدون ایجاد تاخیر در پاسخ به فرستنده اصلی.

---

## تنظیمات ارسال ایمیل (SMTP)

کافیست در تنظیمات پروژه لوکال خود هاست را `127.0.0.1` و پورت را `1025` بگذارید:

::: code-group

```dotenv [لاراول و فریم‌ورک‌های PHP (.env)]
MAIL_MAILER=smtp
MAIL_HOST=127.0.0.1
MAIL_PORT=1025
MAIL_USERNAME=mail-local
MAIL_PASSWORD=ntsec_your_secret_token
MAIL_ENCRYPTION=null
MAIL_FROM_ADDRESS="noreply@local.test"
MAIL_FROM_NAME="NotifyTrap Local"
```

```js [نود جی‌اس (nodemailer)]
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
  subject: 'خوش‌آمدید به NotifyTrap',
  html: '<p>تست دریافت ایمیل روی محیط توسعه محلی</p>',
})
```

:::
