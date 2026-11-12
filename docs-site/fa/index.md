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
  - icon: 📧
    title: پیش‌نمایش و ممیزی حرفه‌ای ایمیل
    details: شنود SMTP روی 127.0.0.1:1025 با رندر ایمن تصاویر درون‌خطی cid، پیش‌نمایش ریسپانسیو (دسکتاپ/تبلت/موبایل)، شبیه‌سازی کلاینت تیره، پیش‌نمایش پیوست و تب ممیزی آفلاین.
  - icon: 💬
    title: شمارنده SMS و پیش‌نمایش دستگاه Push
    details: تشخیص خودکار رمزنگاری GSM-7 / UCS-2 و محاسبه تعداد بخش‌های پیامک در کنار پیش‌نمایش کارت اعلان برای iOS، اندروید و وب پوش.
  - icon: 🪝
    title: شبیه‌سازی Webhook، Replay لوکال و Chaos
    details: شکار هر متد HTTP روی /api/v1/webhooks/{id} همراه با پاسخ Mock دلخواه، بازپخش امن فقط به 127.0.0.1/localhost/[::1] و تزریق تاخیر یا خطای تصادفی HTTP و SMTP.
  - icon: 🗂️
    title: جداسازی کامل با Workspace و Project
    details: تفکیک پروژه‌ها، محیط‌ها (Local، Development، Testing، Staging) و کلیدهای کانال تا پیام‌های اپلیکیشن‌های مختلف هیچ‌وقت با هم قاطی نشوند.
  - icon: 🔐
    title: امنیت و ایزولاسیون بدون ارسال خارجی
    details: پالایش با ammonia و iframe sandbox="" با مسدودسازی پیکسل‌های ردیاب، مقایسه امن Constant-Time، رمزنگاری AES-256-GCM و نگهداری کلید در Keyring سیستم‌عامل.
  - icon: ⌨️
    title: بج ترای، CLI، سرور MCP و جستجوی حرفه‌ای
    details: نمایش تعداد خوانده‌نشده روی ترای و تسک‌بار، اعلان بومی سیستم‌عامل، پالت فرمان Ctrl/Cmd+K، جستجوی ساخت‌یافته /regex/، ابزار notifytrap-cli و سرور محلی MCP.
  - icon: 🌐
    title: دوزبانه کامل (RTL و LTR)
    details: رابط کاربری استاندارد فارسی (راست‌به‌چپ) و انگلیسی (چپ‌به‌چپ) همراه با تم روشن و تاریک مجزا، نرمال‌سازی اعداد و آپدیت امضاشده درون‌برنامه‌ای.
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
