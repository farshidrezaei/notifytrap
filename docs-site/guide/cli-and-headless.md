# Headless Server & CLI Companion

NotifyTrap ships a dedicated standalone binary — `notifytrap-cli` — engineered for developer terminals, headless remote servers, Docker environments, and automated CI pipelines.

---

## 1. Headless Server & Web Companion UI

Run the complete NotifyTrap engine without a desktop graphical environment or WebView dependencies:

```bash
notifytrap-cli serve \
  --http-port 4080 \
  --smtp-port 1025 \
  --web-port 4000 \
  --database-path ~/.local/share/notifytrap/notifications.db
```

### CLI Options:
- `--http-port`: Port for HTTP capture and API routes (default: `4080`).
- `--smtp-port`: Port for SMTP email capture (default: `1025`).
- `--web-port`: Port for serving the bundled Nuxt SPA web dashboard (default: disabled, e.g. `4000`).
- `--database-path`: Custom SQLite database path.

### Web Companion Capabilities
When `--web-port` is specified, NotifyTrap runs an embedded static asset file server with SPA routing fallbacks. You can open `http://localhost:4000` in any web browser on your network to manage channels, view live notifications, and inspect device previews.

---

## 2. CLI Subcommands

### Live Streaming (`tail`)
Stream incoming notifications directly to stdout in real time:

```bash
notifytrap-cli tail --channel-key sms-local.ntsec_your_secret --limit 10
```

### Automated OTP Extraction (`wait-otp`)
Wait for an SMS or Email verification code in automated test suites:

```bash
# Returns only the raw OTP code (e.g. 849201) to stdout
CODE=$(notifytrap-cli wait-otp --channel-key sms-local.ntsec_your_secret --timeout 20)
echo "Extracted Code: $CODE"
```

### Export Notifications (`export`)
Export captures to disk in standard formats:

```bash
# Export all captures as a single JSON array
notifytrap-cli export --channel-key mail-local.ntsec_your_secret --format json --output ./exports.json

# Export email captures as individual .eml files
notifytrap-cli export --channel-key mail-local.ntsec_your_secret --format eml --output ./eml-files/
```

### Self-Updater (`update`)
Upgrade the CLI binary directly to the latest GitHub release:

```bash
notifytrap-cli update
```

### Local Model Context Protocol (`mcp`)
Expose NotifyTrap as an MCP server over stdio for AI coding assistants (Claude Code, Antigravity, Cursor):

```bash
notifytrap-cli mcp --channel-key sms-local.ntsec_your_secret
```

---

## 3. Docker Deployment

Deploy NotifyTrap as a shared container for your development team:

```dockerfile
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

RUN curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh \
  | bash -s -- --cli-only --install-dir /usr/local/bin

EXPOSE 4080 1025 4000

VOLUME ["/data"]

CMD ["notifytrap-cli", "serve", "--http-port", "4080", "--smtp-port", "1025", "--web-port", "4000", "--database-path", "/data/notifytrap.db"]
```
