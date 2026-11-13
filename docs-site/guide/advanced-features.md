# Advanced Inspection & Diagnostics

NotifyTrap provides deep diagnostic and inspection capabilities to help you debug and validate notification templates before pushing code to production.

---

## 1. Realistic Hardware Device Frames

Preview how HTML emails, SMS notifications, and push alerts look inside real physical device chassis:

- **Supported Devices**:
  - **Apple iPhone 15**: Realistic curved chassis, Dynamic Island cutout, and home indicator.
  - **Google Pixel 8**: Sleek aluminum bezel, centered camera punch-hole, and Android navigation bar.
  - **Apple iPad Mini**: Tablet form factor for wide email layouts and responsive testing.
- **Controls**:
  - **Orientation Toggle**: Instantly flip between Portrait and Landscape views.
  - **Device Zoom & Scaling**: Fit large mobile frames smoothly within your desktop inspector view.
  - **One-Click Native Sandbox**: Switch back to raw full-width responsive preview at any time.

---

## 2. Offline MJML Transpiler & Template Linter

### Offline MJML Transpilation
- Transpile `<mjml>`, `<mj-body>`, `<mj-section>`, `<mj-column>`, `<mj-button>`, `<mj-text>`, `<mj-divider>`, and `<mj-image>` directly in pure TypeScript/JavaScript without external Node.js subprocesses or internet connections.
- Automatically compiles semantic tables, inline styling, and responsive CSS wrappers for safe previewing in the sandboxed iframe.

### Deterministic Template Linter
Detect unrendered template variables that accidentally leak to users:

::: v-pre
- Flags unrendered mustache variables: `{{ user.name }}`, `{{ otp_code }}`.
- Flags template literals: `${orderId}`, `#{token}`.
- Flags runtime language null pointers: `null`, `None`, `undefined`, `[object Object]`, `[Array]`.
- Displays an alert banner in the notification header with the exact detected token name and count.
:::

---

## 3. Remote Asset & Deep Link Health Audit

Validate every link and image in your captured emails without clicking manually:

- **Automatic Link Extraction**: Extracts all `href` and `src` URLs from HTML and plain text bodies.
- **Asynchronous Audit Engine**:
  - Probes HTTP/HTTPS URLs with bounded 5-second timeouts.
  - Reports HTTP status code (`200 OK`, `404 Not Found`, `500 Server Error`).
  - Measures server response latency.
  - Flags insecure HTTP links in production templates.
  - Validates `mailto:` address syntax.
- **Offline Caching**: Caches validated URLs to prevent spamming upstream servers during rapid local testing.

---

## 4. SQLite Database Metrics & Maintenance

Keep your local development database clean and performant directly from the **Settings** view:

- **Storage Footprint Analysis**:
  - Displays total SQLite database file size on disk.
  - Shows per-table row counts for notifications, attachments, audit logs, and rules.
- **Automated Retention Pruning**:
  - Configurable retention window (e.g. 7 days, 14 days, 30 days).
  - Automatically purges expired records during startup and channel key rotation.
- **One-Click SQLite VACUUM**:
  - Reclaims unused storage space and defragments SQLite pages with a single click.
  - Emits desktop notifications upon successful optimization.
