# Contributing to NotifyTrap

Thank you for helping improve **NotifyTrap**! Before opening a pull request, please read [`AGENTS.md`](AGENTS.md) — it is the permanent engineering contract for this repository.

## Development Setup

### Prerequisites
- **Bun** (`>= 1.1`)
- **Rust** (`stable`, with `rustfmt` and `clippy`)
- **Tauri 2 system dependencies** (WebKitGTK 4.1 / AppIndicator on Linux, Xcode CLT on macOS, WebView2 + MSVC Build Tools on Windows)

```bash
bun install
bun run tauri dev
```

## Engineering Rules

1. **Zero Real-World Delivery**: NotifyTrap must never send or forward notifications to external servers or real recipients. Loopback replay must remain strictly restricted to `127.0.0.1`, `localhost`, and `[::1]`.
2. **Modular Monolith**: Keep business logic out of `.vue` components and SQL queries out of Tauri/HTTP handlers.
3. **Zero Prohibited Abbreviations**: Use clear, descriptive identifiers (no `cfg`, `ctx`, `mgr`, `svc`, `repo`, `req`, `res`, `msg`, `usr`, `val`, `env`, `notif`, `proj`, `chan`).
4. **Rust Safety**: Never use `.unwrap()` or `.expect()` in production Rust paths. Propagate typed errors with `thiserror`.
5. **Bilingual & Dual-Theme Parity**: Every UI feature requires complete `en` (LTR) and `fa` (RTL) translations, logical CSS (`ms-*`, `me-*`, `ps-*`, `pe-*`, `text-start`, `text-end`), and verification in both Light and Dark themes.
6. **Tests Required**: Add unit, table-driven, property (`proptest`), or integration tests for every behavioral change.

## Verification Checklist

Run all checks locally before opening a PR:

```bash
bun run validate:i18n
bun run typecheck
bun run test:unit
bun run build
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```
