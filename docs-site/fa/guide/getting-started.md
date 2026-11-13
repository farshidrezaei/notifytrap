# شروع سریع و نصب با خط فرمان

نرم‌افزار NotifyTrap هم در قالب نصاب‌های دسکتاپ بومی برای **لینوکس**، **مک** و **ویندوز** و هم به صورت باینری مستقل CLI از طریق [صفحه رسمی ریلیزهای گیت‌هاب](https://github.com/farshidrezaei/notifytrap/releases) منتشر می‌شود.

## نصب یک‌خطی با اسکریپت رسمی

اسکریپت‌های نصب هوشمند NotifyTrap همزمان نسخه گرافیکی دسکتاپ و ابزار خط فرمان `notifytrap-cli` را دریافت کرده و در `PATH` سیستم قرار می‌دهند.

### لینوکس و مک (macOS)

اسکریپت [`scripts/install.sh`](https://github.com/farshidrezaei/notifytrap/blob/main/scripts/install.sh) به صورت خودکار توزیع لینوکس، معماری پردازنده و مدیر بسته سیستم (`apt`/`dpkg` برای `.deb`، `dnf`/`zypper`/`yum` برای `.rpm` یا نسخه پرتابل `.AppImage`) را شناسایی می‌کند:

```bash
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

#### نصب روی سرورها، داکر و CI (فقط CLI بدون رابط گرافیکی)

برای سرورهای بدون مانیتور (Headless)، کانتینرهای داکر یا رانرهای CI که به کتابخانه‌های گرافیکی نیاز ندارند:

```bash
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --cli-only
```

#### پارامترهای اختیاری نصب

```bash
# استفاده اجباری از نسخه AppImage جهت آپدیت خودکار درون‌برنامه‌ای
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --method appimage

# نصب نسخه خاص
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --version v1.5.0

# نصب باینری در مسیر شخصی بدون نیاز به sudo
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --cli-only --install-dir "$HOME/.local/bin"
```

---

### ویندوز (PowerShell)

ترمینال پاورشل را باز کنید و دستور زیر را اجرا نمایید:

```powershell
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

#### نصب فقط نسخه CLI روی ویندوز

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1))) -CliOnly
```

---

## ابزار خط فرمان مستقل (`notifytrap-cli`)

ابزار `notifytrap-cli` همراه اپلیکیشن نصب شده و امکان کنترل کامل را بدون نیاز به باز کردن رابط دسکتاپ فراهم می‌کند:

```bash
# راه‌اندازی سرور پس‌زمینه همراه با سرو رابط کاربری تحت وب Companion
notifytrap-cli serve --http-port 4080 --smtp-port 1025 --web-port 4000

# لاگ زنده نوتیفیکیشن‌های دریافتی در ترمینال
notifytrap-cli tail --channel-key sms-local.ntsec_your_secret

# انتظار و استخراج کد تایید یکبارمصرف (OTP) در تست‌های خودکار CI
OTP_CODE=$(notifytrap-cli wait-otp --channel-key sms-local.ntsec_your_secret --timeout 15)

# خروجی گرفتن از پیام‌ها به صورت فایل JSON یا .eml
notifytrap-cli export --channel-key mail-local.ntsec_your_secret --format eml --output ./emails/

# بروزرسانی خودکار ابزار CLI به آخرین ریلیز گیت‌هاب
notifytrap-cli update
```

---

## ابزار تعاملی تست و سناریوها (Tester TUI)

NotifyTrap شامل یک کنسول متنی تعاملی است که با دستور `npm run tester` (یا `bun run tester`) اجرا می‌شود و به شما اجازه می‌دهد سناریوهای واقعی را با یک کلید شبیه‌سازی کنید:

```bash
npm run tester
```

امکانات تستر:
- منوی تعاملی و تمام‌رنگی در ترمینال.
- **سناریوی ۱:** ایمیل خوش‌آمدگویی با تگ‌های MJML و متغیرهای تست نشده جهت آزمایش لینتر قالب.
- **سناریوی ۲:** وب‌هوک سفارش مشتری جهت بررسی خط لوله فوروارد خودکار.
- **سناریوی ۳:** کد تایید ۶ رقمی پیامک (با ارقام فارسی و انگلیسی).
- **سناریوی ۴:** پوش نوتیفیکیشن موبایل با دیپ‌لینک و پیلود سفارشی.

---

## راه‌اندازی ۶۰ ثانیه‌ای نسخه دسکتاپ

1. **اجرای برنامه:** نرم‌افزار را از منوی برنامه‌ها باز کنید یا در ترمینال `notifytrap` را بنویسید.
2. **ساخت فضای کاری:** در فرم خوش‌آمدگویی اولیه نام فضای کاری و پروژه خود را وارد کنید تا محیط‌های پیش‌فرض ساخته شوند.
3. **ساخت کانال:** از منوی **کانال‌ها** یک کانال ایمیل، پیامک، پوش یا وب‌هوک بسازید.
4. **تنظیم اطلاعات احراز هویت در برنامه:**
   - **ایمیل:** نام‌کاربری و رمز عبور SMTP را در فایل `.env` پروژه وارد کنید.
   - **پیامک / پوش:** کلید کانال (`X-Channel-Key`) را برای ارسال هدر استفاده کنید.
   - **وب‌هوک:** آدرس URL محلی را برای مقصد وب‌هوک‌ها ست کنید.
5. **بررسی آنی پیام‌ها:** نوتیفیکیشن تستی بفرستید. در صفحه **صندوق ورودی** فریم‌های واقعی موبایل و تبلت (آیفون ۱۵، پیکسل ۸، آیپد مینی) را ببینید، لینک‌ها و تصاویر ریموت را ممیزی کنید، از عدم وجود متغیرهای رندر نشده مطمئن شوید و وضعیت حافظه دیتابیس را بهینه‌سازی نمایید.
