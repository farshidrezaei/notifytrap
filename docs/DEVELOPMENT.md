# NotifyTrap — Development, Contribution & Release Workflow

## 1. Prerequisites

- **Rust**: stable toolchain (`rustc >= 1.85`, `cargo`, `rustfmt`, `clippy`)
- **Bun** (`>= 1.x`, matching `bun.lock`) or Node.js `>= 22`
- **Tauri 2 Linux System Dependencies** (Arch / Debian / Fedora):
  - `webkit2gtk-4.1`, `gtk3`, `libayatana-appindicator`, `librsvg`, `openssl`, `pkg-config`

---

## 2. Local Setup & Verification

```bash
# Install frontend dependencies
bun install

# Verify i18n keys across en / fa
bun run validate:i18n

# Run frontend typecheck and unit tests
bun run typecheck
bun run test:unit

# Run Rust format check, strict clippy, and full unit/matrix/integration tests
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets

# Run Tauri 2 desktop app in development mode
bun run tauri dev

# Build production desktop binary (no OS bundle tools required)
bun run tauri build --no-bundle

# Or build full OS installer bundles (.deb / .rpm / .AppImage / .dmg / .msi) when host packagers are installed
bun run tauri build
```

---

## 3. Contribution Workflow

1. Read [AGENTS.md](../AGENTS.md) before modifying code. Never introduce forbidden abbreviations, `.unwrap()`/`.expect()` in production Rust paths, raw hex colors in Vue templates, or unlocalized UI copy.
2. Keep Tauri commands thin (`src-tauri/src/commands/`): validate input in domain/services and map `ApplicationError` to `CommandErrorPayload`.
3. Keep Vue components presentation-focused: place state, caching, and IPC calls inside `app/composables/` and schemas inside `app/schemas/`.
4. Every UI string change requires symmetric updates in both `i18n/locales/en.json` and `i18n/locales/fa.json`, verified via `bun run validate:i18n`.
5. Update `docs/DECISIONS.md`, `docs/ARCHITECTURE.md`, or `docs/TESTING.md` whenever modifying architectural contracts or adding dependencies.
6. Never commit local `.db` databases, `master.key` files, `.env` files, or generated secrets.

---

## 4. Cross-Platform Release & In-App Updater Setup

GitHub Actions workflow [`.github/workflows/release.yml`](../.github/workflows/release.yml) builds signed release bundles on every tag push (`v*`):
- **Linux (`ubuntu-22.04`)**: `.deb` (Debian/Ubuntu), `.rpm` (Fedora/openSUSE), and `.AppImage` (Arch/Manjaro/generic Linux)
- **Windows (`windows-latest`)**: `.exe` (NSIS) and `.msi`
- **macOS (`macos-latest`)**: Universal `.dmg` and `.app.tar.gz`

### Required One-Time Repository Setup Before First Public Release

1. Generate your release signing keypair (keep the private key secret; never commit it):
   ```bash
   bun run tauri signer generate -w ~/.tauri/notifytrap.key
   ```
2. Update [`src-tauri/tauri.conf.json`](../src-tauri/tauri.conf.json) with:
   - `plugins.updater.pubkey`: your generated public key (`*.key.pub` contents)
   - `plugins.updater.endpoints`: verified in `src-tauri/tauri.conf.json` (`https://github.com/farshidrezaei/notifytrap/releases/latest/download/latest.json`)
3. Add GitHub Actions Repository Secrets:
   - `TAURI_SIGNING_PRIVATE_KEY` (contents of private key)
   - `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (passphrase chosen during generation)

---

## 5. Release Checklist

- Bump version consistently in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
- Verify all quality gates pass cleanly:
  - `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
  - `cargo test --manifest-path src-tauri/Cargo.toml --all-targets`
  - `bun run validate:i18n`
  - `bun run test:unit`
  - `bun run typecheck`
  - `bun run build`
  - `bun run tauri build` (or `--no-bundle` for standalone release binary)
- Manually smoke-test in Light & Dark themes and `en` (LTR) & `fa` (RTL) directions.
