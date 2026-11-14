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

---

## 5. Virtualized Inbox & Real-Time Activity Graph

- **Zero-Dependency Virtual Scrolling (`useVirtualList`)**:
  - Dynamically calculates bounded viewports and offsets to render 10,000+ captured notifications with bounded DOM node count and smooth 60fps scrolling.
  - Automatically scrolls selected notifications into view when navigating via mouse or `j`/`k` keyboard shortcuts.
- **Notification Traffic Activity**:
  - Real-time hourly traffic distribution chart in Dashboard (`NotificationActivityCard.vue`).
  - Channel-segmented volume metrics (Email, SMS, Push, Webhook) with interactive hover tooltips and peak rate indicators.
- **Enhanced Command Palette (`Ctrl/Cmd+K`)**:
  - Instant actions to reset all inbox filters, copy the local MCP server endpoint, create new channels, and jump directly to relevant documentation.

---

## 6. Bulk Operations & Batch Tagging

Manage high-volume captured notifications efficiently from the inbox toolbar:

- **Multi-Selection Checkboxes**: Select individual notifications or toggle «Select All» across the current view.
- **Batch Status Transitions**: One-click Mark as Read, Mark as Unread, or Archive across all selected items.
- **Interactive Batch Tagging**: Assign custom tags (e.g. `e2e-run-42`, `auth-flow`, `regression`) to multiple notifications simultaneously. All tags are immediately filterable via the search grammar (`tag:<name>` or `run:<name>`).
- **Transactional Bulk Deletion**: Safely and permanently delete batches of notifications with a modal confirmation dialog.

---

## 7. Data Portability & Fixture Bundles

NotifyTrap provides end-to-end data portability, allowing complete notification fixtures (including payloads, headers, metadata, and binary MIME attachments) to be exported and imported between testing environments, CI runners, and teammate workstations.

### Fixture Bundle Format
Bundles are stored as portable `.json` files conforming to the `NotificationFixtureBundle` schema:

```json
{
  "version": "1.0",
  "generator": "NotifyTrap 1.5.0",
  "exportedAt": "2026-11-14T20:00:00Z",
  "notifications": [
    {
      "channelType": "email",
      "channelIdentifier": "auth-local",
      "sender": "auth@example.com",
      "recipient": "user@example.com",
      "subject": "Verification Code",
      "status": "captured",
      "payloadJson": { "body": "Your code is 123456" },
      "rawPayload": "From: auth@example.com...",
      "previewText": "Your code is 123456",
      "testRunIdentifier": "e2e-run-42",
      "receivedAt": "2026-11-14T20:00:00Z",
      "attachments": [
        {
          "filename": "invoice.pdf",
          "contentType": "application/pdf",
          "byteSize": 1024,
          "contentBase64": "..."
        }
      ]
    }
  ]
}
```

### Export & Import in Desktop UI
- **Export Bundle**: Click the Download icon in the Inbox toolbar to generate and save a timestamped JSON fixture bundle for the active scope.
- **Import Bundle**: Click the Upload icon in the Inbox toolbar to load any fixture JSON file. Notifications and attachments are automatically inserted into the active workspace, project, and environment.



