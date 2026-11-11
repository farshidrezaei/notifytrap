# Build from Source & Contributing

## System Prerequisites

- **Rust** stable toolchain (`rustc >= 1.85`, `cargo`, `rustfmt`, `clippy`)
- **Bun** (`>= 1.x` matching `bun.lock`) or Node.js `>= 22`
- **OS Dependencies**:
  - **Debian / Ubuntu**: `sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf rpm libssl-dev`
  - **Fedora**: `sudo dnf install -y webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel`
  - **Arch Linux**: `sudo pacman -S --needed webkit2gtk-4.1 libappindicator-gtk3 librsvg openssl`
  - **macOS**: Xcode Command Line Tools (`xcode-select --install`)
  - **Windows**: Microsoft C++ Build Tools + WebView2 Runtime

## Local Development

```bash
# Install frontend dependencies
bun install

# Start desktop app (Rust backend + Nuxt UI)
bun run tauri dev

# Build release installers for your current OS
bun run tauri build
```

## Quality & Testing Gates

All pull requests and release tags must pass the full backend + frontend matrix:

```bash
# Frontend localization, types & Vitest matrix (table-driven + form masks + updater + sanitizer)
bun run validate:i18n
bun run typecheck
bun run test:unit

# Rust formatting, strict Clippy, property-based & live HTTP/SMTP SQLite integration tests
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml --all-targets
```

See the repository [`AGENTS.md`](https://github.com/farshidrezaei/notifytrap/blob/main/AGENTS.md) for permanent naming, modularity, i18n, and testing rules.
