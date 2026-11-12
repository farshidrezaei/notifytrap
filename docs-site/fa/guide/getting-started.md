---
title: شروع سریع
---

# شروع سریع

**NotifyTrap** جایگزینی سریع و سبک برای محیط‌های تست بیرونی و کانتینرهای پراکنده است. یک بار نصبش کنید و با راه‌اندازی خودکار سرویس‌های **SMTP** و **HTTP** تمام پیام‌های خروجی سیستم‌های خود را یک‌جا ببینید.

## نصب برنامه

### ۱) اسکریپت نصب خودکار

::: code-group

```bash [Linux و macOS]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

```powershell [Windows (PowerShell)]
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

:::

### ۲) دریافت فایل نصب مستقیم

از صفحه [GitHub Releases](https://github.com/farshidrezaei/notifytrap/releases) متناسب با سیستم‌عامل خود دانلود کنید:

- **macOS**: `.dmg` یا `.app` (پشتیبانی از Apple Silicon و Intel)
- **Windows**: نصب‌کننده `.exe` (NSIS) یا `.msi`
- **Linux**: `.AppImage`، `.deb` یا `.rpm`

### ۳) بیلد مستقیم از سورس

```bash
git clone https://github.com/farshidrezaei/notifytrap.git
cd notifytrap
bun install
bun run tauri dev
```

پیشنیازها: **Rust (stable)**، **Bun** (یا Node.js 22+) و وابستگی‌های سیستمی Tauri 2 (مانند `libwebkit2gtk-4.1-dev` در اوبونتو/دبیان).

---

## پورت‌ها و سرویس‌های پیش‌فرض

در نخستین اجرا، NotifyTrap یک Workspace، یک Project، یک Environment پیش‌فرض و دو کانال پایه (ایمیل و HTTP) می‌سازد و دو شنونده لوکال را استارت می‌زند:

| سرویس | آدرس و پورت پیش‌فرض | کاربرد |
| --- | --- | --- |
| سرور SMTP | `127.0.0.1:1025` | دریافت ایمیل با احراز هویت `PLAIN` / `LOGIN` |
| سرور HTTP | `127.0.0.1:4080` | دریافت `SMS`، `Push` و `Webhook` + کوئری/انتظار `/api/v1/notifications*` + `GET /api/v1/health` |

> [!TIP] مدیریت پورت‌ها
> در بخش **Settings** می‌توانید آدرس Bind و شماره پورت‌های SMTP و HTTP را تغییر دهید و وضعیت آنلاین هر شنونده را در نوار وضعیت پایین برنامه ببینید.

---

## گردش کار پایه در ۳ قدم

1. **انتخاب محیط (Workspace → Project → Environment):** در نوار بالای برنامه یا صفحه Workspaces محیط خود (مثلاً `Local`) را بسازید یا انتخاب کنید.
2. **کپی Credential از صفحه Channels:** کلید کانال را از کارت مربوطه کپی کنید (یا بخش `.env` آماده را مستقیماً داخل پروژه Laravel / Node / Go خود قرار دهید).
3. **ارسال پیام و بررسی حرفه‌ای در Inbox:** به‌محض دریافت، پیام به‌صورت زنده در **Inbox** ظاهر می‌شود؛ می‌توانید بدنه HTML ایمیل را در حالت دسکتاپ/تبلت/موبایل یا کلاینت تیره و همراه تصاویر `cid:` ببینید، کدهای OTP و لینک جادویی را با یک کلیک کپی کنید، گزارش ممیزی ایمیل و شمارنده بخش‌های پیامک را بررسی کنید، دو پیام را خط‌به‌خط مقایسه کنید و خروجی `.eml` / `JSON` / `cURL` بگیرید.
