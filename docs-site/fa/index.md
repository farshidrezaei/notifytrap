---
layout: home
title: NotifyTrap
titleTemplate: جعبه ابزار دسکتاپ لوکال-فرست برای شکار و بررسی نوتیفیکیشن‌ها

hero:
  name: NotifyTrap
  text: شکار، بررسی و دیباگ همه‌ نوتیفیکیشن‌های توسعه به‌صورت لوکال
  tagline: اپلیکیشن دسکتاپ سریع و آفلاین با Tauri 2، Rust و Nuxt 4؛ با سرورهای مجزای SMTP و HTTP، بدون هیچ ارسال واقعی به بیرون.
  image:
    src: /notifytrap-logo.png
    alt: لوگوی NotifyTrap
  actions:
    - theme: brand
      text: شروع سریع
      link: /fa/guide/getting-started
    - theme: alt
      text: کانال‌ها و API
      link: /fa/guide/channels-and-api
    - theme: alt
      text: دریافت از GitHub
      link: https://github.com/farshidrezaei/notifytrap/releases

features:
  - icon: 📨
    title: شنود واقعی SMTP و HTTP
    details: دریافت هم‌زمان ایمیل‌های SMTP (با پشتیبانی از MIME، متن ساده، HTML و پیوست‌ها) و نوتیفیکیشن‌های HTTP برای SMS، Push و Webhook روی پورت‌های لوکال.
  - icon: 🗂️
    title: جداسازی کامل با Workspace و Project
    details: تفکیک پروژه‌ها، محیط‌ها (Local، Staging، Production و غیره) و کلیدهای کانال تا پیام‌های اپلیکیشن‌های مختلف هیچ‌وقت با هم قاطی نشوند.
  - icon: 🔐
    title: احراز هویت واقعی و ذخیره امن
    details: تولید Credential با ساختار پیشوند + سکرت، بررسی با مقایسه امن (Constant-Time)، رمزنگاری سکرت‌ها با AES-256-GCM و نگهداری کلید در Keyring سیستم‌عامل.
  - icon: 🛡️
    title: ایزولاسیون سندباکس HTML ایمیل
    details: پالایش HTML در بک‌اند با ammonia و نمایش در iframe ایزوله‌شده با sandbox="" بدون اجرای اسکریپت یا لود منابع ریموت.
  - icon: 🌐
    title: دوزبانه کامل (RTL و LTR)
    details: رابط کاربری استاندارد فارسی (راست‌به‌چپ) و انگلیسی (چپ‌به‌چپ) همراه با تم روشن و تاریک مجزا و توکن‌های طراحی Nuxt UI v4.
  - icon: ⚡
    title: مونو‌لیت ماژولار روی SQLite
    details: بدون نیاز به Docker یا سرویس خارجی. داده‌ها به‌صورت محلی در SQLite ذخیره می‌شوند و با نصب یک باینری سبک آماده به کار است.
---

<div class="nt-banner-frame">
  <img src="/notifytrap-banner.png" alt="بنر NotifyTrap — شکار و بررسی ایمیل، SMS، Push و Webhook به‌صورت لوکال" />
</div>

## نصب سریع

نصب‌کننده‌های زیر آخرین نسخه منتشرشده را مستقیم از GitHub Releases دانلود و نصب می‌کنند:

::: code-group

```bash [macOS / Linux]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

```powershell [Windows (PowerShell)]
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

:::

> [!IMPORTANT] بدون ارسال واقعی
> **NotifyTrap هرگز پیام واقعی به بیرون ارسال نمی‌کند.** تمام ایمیل‌ها، SMSها، Pushها و Webhookها درون محیط امن توسعه شکار و در SQLite لوکال ذخیره می‌شوند.
