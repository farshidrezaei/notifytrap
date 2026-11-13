---
layout: home
title: NotifyTrap
titleTemplate: جعبه ابزار دسکتاپ لوکال-فرست برای شکار و بررسی نوتیفیکیشن‌ها

hero:
  name: NotifyTrap
  text: شکار، بررسی و دیباگ همه‌ نوتیفیکیشن‌های توسعه به‌صورت لوکال
  tagline: اپلیکیشن دسکتاپ سریع و آفلاین با Tauri 2، Rust و Nuxt 4؛ با سرورهای مجزای SMTP و HTTP، همراه با نسخه Headless برای سرورها و بدون هیچ ارسال واقعی به بیرون.
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
      text: ابزارهای بازرسی
      link: /fa/guide/advanced-features
    - theme: alt
      text: دریافت از GitHub
      link: https://github.com/farshidrezaei/notifytrap/releases

features:
  - icon: 📱
    title: قاب‌های سخت‌افزاری واقعی و شبیه‌سازها
    details: شبیه‌سازی دقیق فریم گوشی و تبلت (iPhone 15، Google Pixel 8، iPad Mini) با حاشیه‌های واقعی، بریدگی جزیره پویا، تغییر جهت افقی/عمودی و مقیاس ریسپانسیو.
  - icon: 🧩
    title: رندر آفلاین MJML و لینتر قالب
    details: تبدیل تگ‌های MJML به HTML معنایی بدون وابستگی به نود و اینترنت، همراه با لینتر خودکار جهت شناسایی متغیرهای رندر نشده (مانند تگ‌های موستاش، null، None و [object Object]).
  - icon: 🔍
    title: ممیزی زنده پیوندها و فایل‌های ریموت
    details: بررسی غیرهمزمان وضعیت لینک‌های وب، کد پاسخ، زمان لود، خطاهای SSL و حجم تصاویر با کش‌کردن محلی برای سنجش سلامت نوتیفیکیشن‌ها.
  - icon: 💾
    title: متریک‌های دیتابیس و بهینه‌سازی SQLite
    details: گزارش حجم مصرفی دیسک به تفکیک جداول، حذف دوره‌ای خودکار داده‌های قدیمی و اجرای یک‌کلیکه دستور VACUUM در بخش تنظیمات.
  - icon: 🌐
    title: تانل عمومی اینگرس برای وب‌هوک‌ها
    details: دریافت وب‌هوک‌های واقعی سرویس‌های اینترنتی (Stripe، GitHub، درگاه‌های پرداخت) روی سیستم محلی از طریق cloudflared، ngrok یا bore تنها با یک کلیک.
  - icon: 🔀
    title: خط لوله فوروارد خودکار وب‌هوک
    details: بازپخش خودکار وب‌هوک‌های شکارشده به سرویس‌های ثانویه (مثل Zapier یا میکروسرویس‌های لوکال) همراه با فیلتر نام رویداد، هدرهای سفارشی و تایم‌اوت.
  - icon: 🖥️
    title: سرور هدلس و رابط تحت‌وب Companion
    details: اجرای کامل موتور نوتیفیکیشن روی سرور، داکر و CI از طریق دستور notifytrap-cli serve به همراه سرو کامل رابط کاربری وب روی پورت مجزا.
  - icon: ⌨️
    title: بج ترای، CLI، سرور MCP و جستجوی پیشرفته
    details: آیکون ترای با شماره پیام‌ها، اعلان بومی سیستم‌عامل، پالت فرمان Ctrl/Cmd+K، جستجوی ساخت‌یافته /regex/، ابزار CLI و سرور محلی MCP.
  - icon: 🌐
    title: دوزبانه استاندارد فارسی (RTL) و انگلیسی (LTR)
    details: پشتیبانی کامل از چینش راست‌به‌چپ، فونت‌های بومی، تم روشن و تاریک مستقل، نرمال‌سازی ارقام فارسی و آپدیت خودکار درون‌برنامه‌ای.
---

<div class="nt-banner-frame">
  <img src="/notifytrap-banner.png" alt="بنر NotifyTrap — شکار و بررسی ایمیل، SMS، Push و Webhook به‌صورت لوکال" />
</div>

## نصب سریع با ترمینال

اسکریپت‌های زیر آخرین نسخه را دریافت کرده و هم اپلیکیشن دسکتاپ و هم باینری مستقل `notifytrap-cli` را روی سیستم شما نصب و در PATH قرار می‌دهند:

::: code-group

```bash [لینوکس و مک (bash)]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

```bash [سرور لینوکس، داکر و CI (فقط CLI)]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --cli-only
```

```bash [نسخه AppImage لینوکس (آپدیت درون‌برنامه‌ای)]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --method appimage
```

```powershell [ویندوز (PowerShell)]
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

```powershell [ویندوز سرور و CI (فقط CLI)]
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1))) -CliOnly
```

:::

> [!IMPORTANT] بدون ارسال واقعی
> **NotifyTrap هرگز پیام واقعی به بیرون ارسال نمی‌کند.** تمام ایمیل‌ها، SMSها، Pushها و Webhookها درون محیط امن توسعه شکار و در SQLite لوکال ذخیره می‌شوند.
