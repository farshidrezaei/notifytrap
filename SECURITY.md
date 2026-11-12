# Security Policy

## Supported Versions

Security fixes are released on the latest minor version of NotifyTrap (`v1.4.x`).

## Security Architecture

NotifyTrap is designed for local development and testing:
- Embedded HTTP and SMTP listeners bind strictly to `127.0.0.1` by default.
- Channel secrets are encrypted at rest with `AES-256-GCM` (OS keyring master key with `0o600` fallback) and verified using constant-time `SHA-256` comparison.
- Untrusted HTML emails are sanitized in Rust with `ammonia` and rendered inside an isolated `<iframe sandbox="" referrerpolicy="no-referrer">` with strict offline CSP (`script-src 'none'; connect-src 'none'; img-src data:;`).
- Sensitive HTTP headers and query parameters are redacted before SQLite persistence.
- Webhook replay is strictly restricted to loopback addresses (`127.0.0.1`, `localhost`, `[::1]`).

## Reporting a Vulnerability

Please **do not** open a public GitHub issue for security vulnerabilities.
Instead, report security issues privately via GitHub Security Advisories (**Security → Advisories → New draft security advisory**) on `farshidrezaei/notifytrap`, including:
- Affected component and version
- Reproduction steps or proof-of-concept
- Potential impact

We will acknowledge receipt within 48 hours and coordinate a fix before public disclosure.
