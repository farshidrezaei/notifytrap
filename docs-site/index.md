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
      text: Inspection Tools
      link: /guide/advanced-features
    - theme: alt
      text: فارسی
      link: /fa/

features:
  - icon: 📱
    title: Realistic Device Frames & Simulators
    details: Pixel-perfect hardware chassis views for iPhone 15, Google Pixel 8, and iPad Mini with responsive scaling, portrait/landscape flipping, and Dynamic Island/punch-hole indicators.
  - icon: 🧩
    title: Offline MJML & Template Linter
    details: Zero-network MJML tag transpilation and deterministic template linting alerting immediately on unrendered tokens (mustache tags, ${...}, null, None, [object Object]).
  - icon: 🔍
    title: Live Link & Remote Asset Health Audit
    details: Async background checker inspecting external HTTP/HTTPS assets and links with real-time status code, response time, SSL error, and offline caching metrics.
  - icon: 💾
    title: SQLite Metrics & One-Click VACUUM
    details: Granular disk footprint and table row count breakdown, automatic retention pruning, and single-click SQLite VACUUM optimization in Settings.
  - icon: 🌐
    title: Public Webhook Ingress Tunneling
    details: Expose local capture endpoints to external webhooks (Stripe, GitHub, Shopify) via cloudflared, ngrok, or bore with one click and automatic loopback health checks.
  - icon: 🔀
    title: Downstream Webhook Auto-Forwarding
    details: Asynchronously fan out captured webhooks to secondary targets (Zapier, local microservices) with event name filtering, custom headers, and retry timeouts.
  - icon: 🖥️
    title: Headless Server & Web Companion UI
    details: Run notifytrap-cli serve on headless servers, Docker containers, or CI environments with an embedded Axum web server hosting the complete Nuxt SPA interface.
  - icon: ⌨️
    title: Tray Badge, CLI, MCP & Search Syntax
    details: Unread badges on tray & taskbar, Ctrl/Cmd+K Command Palette, structured /regex/ queries, notifytrap-cli tail/wait-otp, and built-in local MCP server.
  - icon: 🌐
    title: Fully Bilingual EN (LTR) & FA (RTL)
    details: Native Light and Dark themes, symmetrical RTL/LTR layout, bundled Persian typography, and automatic numeral normalization with signed cross-platform auto-updates.
---

<div class="notifytrap-banner-frame">
  <img src="/notifytrap-banner.png" alt="NotifyTrap — Email, SMS, Push, and Webhook Local Capture" />
</div>

## Install in Seconds

Choose your operating system terminal command to pull the latest release automatically. Both the Desktop GUI and standalone `notifytrap-cli` are installed to your PATH:

::: code-group

```bash [Linux & macOS (bash)]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

```bash [Headless / Docker / CI (--cli-only)]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --cli-only
```

```bash [Linux AppImage (in-app auto-update)]
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --method appimage
```

```powershell [Windows (PowerShell)]
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

```powershell [Windows Headless / CI (-CliOnly)]
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1))) -CliOnly
```

:::
