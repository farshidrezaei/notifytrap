# سرور هدلس و ابزار خط فرمان CLI

علاوه بر نرم‌افزار دسکتاپ، باینری مستقل `notifytrap-cli` به طور ویژه برای ترمینال توسعه‌دهندگان، سرورهای لینوکس، کانتینرهای داکر و خطوط تست CI توسعه یافته است.

---

## ۱. اجرای سرور هدلس و رابط تحت وب Companion

اجرای کامل موتور دریافت و پردازش NotifyTrap بدون نیاز به محیط گرافیکی یا وب‌ویو:

```bash
notifytrap-cli serve \
  --http-port 4080 \
  --smtp-port 1025 \
  --web-port 4000 \
  --database-path ~/.local/share/notifytrap/notifications.db
```

### سوییچ‌های دستوری:
- `--http-port`: پورت دریافت درخواست‌های وب‌هوک و API (پیش‌فرض: `4080`).
- `--smtp-port`: پورت دریافت ایمیل‌های لوکال (پیش‌فرض: `1025`).
- `--web-port`: پورت سرو رابط کاربری وب اپلیکیشن Nuxt SPA (پیش‌فرض: غیرفعال، مثلاً `4000`).
- `--database-path`: تعیین مسیر دلخواه فایل پایگاه داده SQLite.

### ویژگی‌های رابط Companion
با مشخص کردن پورت وب (`--web-port`)، فایل‌های استاتیک پروژه به صورت کامل سرو شده و می‌توانید از طریق مرورگر با مراجعه به آدرس `http://localhost:4000` به صندوق پیام‌ها، ابزارهای بازرسی و مدیریت کانال‌ها دسترسی پیدا کنید.

---

## ۲. زیرفرمان‌های ابزار CLI

### مانیتور زنده پیام‌ها (`tail`)
مشاهده لاگ آنی نوتیفیکیشن‌های دریافتی در خروجی استاندارد ترمینال:

```bash
notifytrap-cli tail --channel-key sms-local.ntsec_your_secret --limit 10
```

### انتظار و دریافت کد یکبارمصرف (`wait-otp`)
دریافت مستقیم کد تایید ایمیل یا پیامک در تست‌های خودکار بدون پردازش اضافه:

```bash
# خروجی این دستور تنها خود کد رقمی (مثلاً 849201) است
CODE=$(notifytrap-cli wait-otp --channel-key sms-local.ntsec_your_secret --timeout 20)
echo "کد استخراج‌شده: $CODE"
```

### خروجی گرفتن از پیام‌ها (`export`)
ذخیره اعلان‌های ثبت‌شده در قالب‌های استاندارد:

```bash
# ذخیره همه پیام‌ها در یک فایل JSON
notifytrap-cli export --channel-key mail-local.ntsec_your_secret --format json --output ./exports.json

# ذخیره ایمیل‌ها به صورت فایل‌های مجزای .eml
notifytrap-cli export --channel-key mail-local.ntsec_your_secret --format eml --output ./eml-files/
```

### آپدیت خودکار CLI (`update`)
بروزرسانی مستقیم باینری ترمینال به آخرین نسخه منتشر شده در گیت‌هاب:

```bash
notifytrap-cli update
```

### پروتکل محلی MCP (`mcp`)
اتصال مستقیم NotifyTrap به عنوان سرور ابزار هوش مصنوعی (برای Claude Code، Cursor و Antigravity):

```bash
notifytrap-cli mcp --channel-key sms-local.ntsec_your_secret
```

---

## ۳. اجرای NotifyTrap در داکر (Docker)

راه‌اندازی به عنوان یک سرویس متمرکز در تیم‌های توسعه:

```dockerfile
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh \
  | bash -s -- --cli-only --install-dir /usr/local/bin

EXPOSE 4080 1025 4000

VOLUME ["/data"]

CMD ["notifytrap-cli", "serve", "--http-port", "4080", "--smtp-port", "1025", "--web-port", "4000", "--database-path", "/data/notifytrap.db"]
```
