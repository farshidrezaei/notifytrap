---
layout: home

hero:
  name: NotifyTrap
  text: Local Notification Capture Utility
  tagline: Capture, inspect, and debug Email (SMTP), SMS, Push, and Webhook notifications in one offline desktop workspace — with zero real-world delivery.
  image:
    src: /notifytrap-logo.png
    alt: NotifyTrap Logo
  actions:
    - theme: brand
      text: Install via CLI
      link: /guide/getting-started
    - theme: alt
      text: Channels & API
      link: /guide/channels-and-api
    - theme: alt
      text: فارسی
      link: /fa/

features:
  - icon: 📧
    title: Email Studio & Audit Sandbox
    details: Embedded ESMTP (127.0.0.1:1025) with multipart MIME, inline cid image hydration, safe attachment preview, Desktop/Tablet/Mobile + Dark Client views, and offline email audits.
  - icon: 💬
    title: SMS, Push & Device Previews
    details: Dedicated POST /api/v1/sms and POST /api/v1/push endpoints with GSM-7/UCS-2 segment counting and iOS, Android, and Web Push device card previews.
  - icon: 🧪
    title: Test Automation, OTP & Query API
    details: Event-driven /api/v1/notifications/wait long-polling, X-NotifyTrap-Run test tagging, automatic OTP & magic-link extraction, diff comparison, and .eml/JSON/cURL export.
  - icon: 🪝
    title: Webhook Trap, Mock & Loopback Replay
    details: Captures ANY HTTP method on /api/v1/webhooks/{id}[/{subpath}], returns configurable mock responses, replays strictly to 127.0.0.1/localhost/[::1], and injects HTTP/SMTP chaos rules.
  - icon: 🔒
    title: Workspace Isolation & Security
    details: Explicit Workspace → Project → Environment → Channel hierarchy backed by AES-256-GCM credentials, OS keyring + 0o600 fallback, and strict loopback binding.
  - icon: ⌨️
    title: Tray Badge, CLI, MCP & Search Syntax
    details: System tray & taskbar unread badges, native OS notifications, Ctrl/Cmd+K Command Palette, structured /regex/ search, notifytrap-cli, and built-in local MCP server.
  - icon: 🌐
    title: Bilingual EN (LTR) & FA (RTL)
    details: Native Light and Dark themes, symmetrical RTL/LTR layout, bundled fonts, and automatic Persian/Arabic-Indic/Devanagari digit normalization with signed cross-platform updates.
---

<div class="notifytrap-banner-frame">
  <img src="/notifytrap-banner.png" alt="NotifyTrap — Email, SMS, Push, and Webhook Local Capture" />
</div>

## Install in Seconds

Choose your operating system terminal command to pull the latest release automatically:

::: code-group

```bash [Linux & macOS (bash)]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

```bash [Linux AppImage (in-app auto-update)]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --method appimage
```

```powershell [Windows (PowerShell)]
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

:::
