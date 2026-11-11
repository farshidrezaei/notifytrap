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
    title: Embedded ESMTP Server
    details: Captures local development emails on 127.0.0.1:1025 with AUTH PLAIN/LOGIN, multipart MIME, attachments, backend ammonia HTML sanitization, and sandboxed preview.
  - icon: 💬
    title: SMS & Push HTTP Endpoints
    details: Dedicated POST /api/v1/sms and POST /api/v1/push endpoints returning structured 202 Accepted capture confirmations with constant-time credential verification.
  - icon: 🪝
    title: Universal Webhook Trap
    details: Captures ANY HTTP method on /api/v1/webhooks/{id}[/{subpath}] with redacted sensitive headers/query strings, JSON formatting, and syntax-highlighted payloads.
  - icon: 🔒
    title: Workspace Isolation & Security
    details: Explicit Workspace → Project → Environment → Channel hierarchy backed by AES-256-GCM credentials, OS keyring + 0o600 fallback, and strict loopback binding.
  - icon: 🌐
    title: Bilingual EN (LTR) & FA (RTL)
    details: Native Light and Dark themes, symmetrical RTL/LTR layout, bundled fonts, and automatic Persian/Arabic-Indic/Devanagari digit normalization.
  - icon: 🚀
    title: Cross-Platform & Auto-Updated
    details: Native builds for Linux (.deb, .rpm, .AppImage), macOS (Universal .dmg), and Windows (.exe, .msi) with cryptographically signed in-app updates.
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
