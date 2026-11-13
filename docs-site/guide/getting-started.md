# Getting Started & CLI Installation

NotifyTrap ships cross-platform desktop installers and standalone CLI binaries for **Linux**, **macOS**, and **Windows** via [GitHub Releases](https://github.com/farshidrezaei/notifytrap/releases).

## One-Command Installers

The official installer scripts download both the **Desktop GUI** and the **`notifytrap-cli`** tool, automatically placing the CLI binary into your system `PATH`.

### Linux & macOS

The [`scripts/install.sh`](https://github.com/farshidrezaei/notifytrap/blob/main/scripts/install.sh) installer automatically detects your operating system, CPU architecture, and Linux package manager (`apt`/`dpkg` for `.deb`, `dnf`/`zypper`/`yum` for `.rpm`, or `.AppImage` fallback with desktop entry + symlink to `~/.local/bin/notifytrap` and `/usr/local/bin/notifytrap-cli`):

```bash
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

#### Headless Servers, CI & Docker (`--cli-only`)

For headless Linux servers, GitHub Actions runners, or Docker containers that do not require desktop GUI libraries (like WebKitGTK):

```bash
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --cli-only
```

#### Useful Install Flags

```bash
# Force AppImage on Ubuntu/Debian/Fedora for direct signed in-app self-updates
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --method appimage

# Pin a specific release tag
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --version v1.5.0

# Install standalone CLI without sudo prompt (into ~/.local/bin)
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --cli-only --install-dir "$HOME/.local/bin"
```

Supported `--method` values: `auto`, `deb`, `rpm`, `appimage`, `dmg`.

---

### Windows (PowerShell)

Run PowerShell as Administrator or regular user:

```powershell
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

#### Windows Headless / CI (`-CliOnly`)

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1))) -CliOnly
```

#### Silent Installer Flags

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1))) -Method msi -Silent
```

---

## The Standalone CLI (`notifytrap-cli`)

The `notifytrap-cli` companion provides a full suite of headless operations:

```bash
# Start full background capture server + Web Companion UI
notifytrap-cli serve --http-port 4080 --smtp-port 1025 --web-port 4000

# Stream real-time captured notifications to terminal
notifytrap-cli tail --channel-key sms-local.ntsec_your_secret

# Wait for an OTP code in automated CI/test suites
OTP_CODE=$(notifytrap-cli wait-otp --channel-key sms-local.ntsec_your_secret --timeout 15)

# Export captures to JSON or .eml files
notifytrap-cli export --channel-key mail-local.ntsec_your_secret --format eml --output ./captured-emails/

# Self-update the CLI binary to the latest GitHub release
notifytrap-cli update
```

---

## Interactive Scenario Testing TUI

NotifyTrap includes an interactive Node.js terminal utility (`npm run tester` or `bun run tester`) to fire real-world test notifications across all channels:

```bash
npm run tester
```

Features:
- Step-by-step interactive curses-style menu.
- **Scenario 1**: Welcome email with embedded MJML and intentional unrendered placeholders to test template linting.
- **Scenario 2**: Customer order webhook with auto-forwarding verification.
- **Scenario 3**: 6-digit Arabic/Persian/English OTP verification code over SMS.
- **Scenario 4**: Mobile push notification with deep-link action payload.
- Real-time color-coded capture assertions and payload inspection.

---

## 60-Second First Run (Desktop App)

1. **Launch NotifyTrap**: Open the application from your desktop launcher or run `notifytrap` in your terminal.
2. **Provision Workspace**: Complete the onboarding step to create your workspace, project, and default environments (`local`, `development`, `testing`, `staging`).
3. **Configure Channel**: Go to **Channels** and create your notification channel (Email, SMS, Push, or Webhook).
4. **Copy Credentials**:
   - **Email**: Copy **SMTP Username** and **SMTP Password** into your `.env`.
   - **SMS / Push**: Copy the `X-Channel-Key` (`<identifier>.<secret>`).
   - **Webhook**: Copy the local trap URL and set optional mock responses or chaos rules.
5. **Inspect Live**: Send your first test notification. In the **Inbox**, view realistic device chassis (`iPhone 15`, `Pixel 8`, `iPad Mini`), audit remote links and images, check unrendered template variables, or trigger SQLite maintenance with one click in **Settings**.
