---
title: توسعه و مشارکت
---

# توسعه و مشارکت

برای توسعه، تست و ساخت باینری‌های محلی NotifyTrap از دستورات زیر استفاده کنید.

## راه‌اندازی محیط توسعه

```bash
git clone https://github.com/farshidrezaei/notifytrap.git
cd notifytrap
bun install
bun run tauri dev
```

## دستورات کیفیت، تست و بررسی

پیش از هر PR، تمام بررسی‌های زیر را اجرا کنید:

| بررسی | دستور | پوشش |
| --- | --- | --- |
| اعتبارسنجی ترجمه‌ها (i18n) | `bun run validate:i18n` | هم‌خوانی کلیدها و Placeholderهای `en.json` و `fa.json` |
| اعتبارسنجی تایپ‌ها | `bun run typecheck` | بررسی `strict: true` در TypeScript و Vue |
| تست‌های یونیت فرانت‌اند | `bun run test:unit` | تست‌های Vitest برای توابع کمکی، ماسک‌ها، sanitizer و منطق فرانت‌اند |
| فرمت کد Rust | `cargo fmt --manifest-path src-tauri/Cargo.toml --check` | بررسی قالب‌بندی استاندارد `rustfmt` |
| کلیپی (Clippy) Rust | `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` | استانداردهای کدنویسی بدون Warning |
| تست‌های بک‌اند و Integration | `cargo test --manifest-path src-tauri/Cargo.toml --all-targets` | پارس MIME، رمزنگاری، پایگاه داده SQLite، property testها و سرورهای زنده SMTP/HTTP |

## ساختار مستندات این سایت

سایت مستندات با **VitePress** در مسیر `docs-site/` پیاده‌سازی شده است و به‌صورت خودکار با هر پوش روی شاخه `main` از طریق GitHub Actions روی GitHub Pages منتشر می‌شود:

```bash
cd docs-site
bun install
bun run dev
```
