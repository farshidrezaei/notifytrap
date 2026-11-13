# Public Ingress Tunnels & Downstream Forwarding

Modern web applications depend heavily on third-party webhook providers (Stripe, GitHub, Shopify, PayPal, Twilio) that send events to publicly accessible URLs. NotifyTrap makes receiving and fanning out these webhooks effortless.

---

## 1. Public Webhook Ingress Tunnels

NotifyTrap includes a built-in tunnel runner supporting industry-standard tunneling clients:

### Supported Tunnel Providers
1. **Cloudflare Tunnel (`cloudflared`)**:
   - Zero-config temporary public URL: `https://*.trycloudflare.com`.
   - No account or credit card required.
2. **ngrok (`ngrok`)**:
   - High-speed tunnels supporting custom auth tokens and subdomains.
3. **bore (`bore`)**:
   - Lightweight, open-source TCP tunnel for private or custom infrastructure.

### How It Works
- Navigate to **Settings → Public Webhook Tunnel**.
- Select your provider binary (NotifyTrap auto-detects if installed in your `PATH` or allows specifying a custom binary path).
- Click **Start Public Tunnel**.
- NotifyTrap launches the tunnel subprocess, waits for loopback health verification, extracts the public HTTPS URL, and displays it with a copy pill.
- Any request sent to your public tunnel URL is piped directly to your local NotifyTrap HTTP server and routed to your matching Webhook channel!

---

## 2. Downstream Webhook Auto-Forwarding

Often in development, you want NotifyTrap to capture the webhook for inspection **and** simultaneously forward it to another service (e.g. a local microservice running on port 8000, Zapier, or a QA server).

### Configuration (`Channels → Rules → Forwarding`)
1. **Target URL**: The destination endpoint to receive the forwarded webhook (e.g. `http://127.0.0.1:8000/api/webhooks`).
2. **Event Filter**: Optional event name matcher (e.g. `checkout.session.completed`, `payment_intent.succeeded`). If set, NotifyTrap only forwards webhooks where `event`, `type`, or `event_name` in the JSON payload matches.
3. **Custom Headers**: Add headers such as `Authorization: Bearer <token>` or `X-Forwarded-Source: NotifyTrap`.
4. **Timeout**: Configurable HTTP timeout (default 5000 ms).

### Non-Blocking Architecture
- NotifyTrap writes the incoming webhook to SQLite and sends the response (`202 Accepted` or mock response) back to the sender first.
- The forwarding pipeline runs asynchronously in the background. Downstream network failures never interrupt your local webhook capture.
