# Getting Started & CLI Installation

NotifyTrap ships native desktop installers for **Linux**, **macOS**, and **Windows** via [GitHub Releases](https://github.com/farshidrezaei/notifytrap/releases).

## One-Command CLI Installers

### Linux & macOS

The [`scripts/install.sh`](https://github.com/farshidrezaei/notifytrap/blob/main/scripts/install.sh) installer automatically detects your operating system, CPU architecture, and Linux package manager (`apt`/`dpkg` for `.deb`, `dnf`/`zypper`/`yum` for `.rpm`, or `.AppImage` fallback with desktop entry + `~/.local/bin/notifytrap` symlink):

```bash
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash
```

Optional flags:

```bash
# Force AppImage on Ubuntu/Debian/Fedora for direct signed in-app self-updates
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --method appimage

# Pin a specific release tag
curl -fsSL https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.sh | bash -s -- --version v1.0.0
```

Supported `--method` values: `auto`, `deb`, `rpm`, `appimage`, `dmg`.

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1 | iex
```

With explicit installer format and silent mode:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/farshidrezaei/notifytrap/main/scripts/install.ps1))) -Method msi -Silent
```

---

## Manual CLI Commands by OS & Distro

::: code-group

```bash [Debian / Ubuntu (.deb)]
ASSET_URL=$(curl -fsSL https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest \
  | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+\.deb"' \
  | head -n 1 | cut -d'"' -f4)
curl -fL "$ASSET_URL" -o /tmp/notifytrap.deb
sudo apt-get update && sudo apt-get install -y /tmp/notifytrap.deb
```

```bash [Fedora / RHEL / openSUSE (.rpm)]
ASSET_URL=$(curl -fsSL https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest \
  | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+\.rpm"' \
  | head -n 1 | cut -d'"' -f4)
curl -fL "$ASSET_URL" -o /tmp/notifytrap.rpm
sudo dnf install -y /tmp/notifytrap.rpm
```

```bash [Arch / Universal (.AppImage)]
mkdir -p ~/.local/bin ~/.local/share/notifytrap
ASSET_URL=$(curl -fsSL https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest \
  | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+\.AppImage"' \
  | grep -v '\.sig"' | head -n 1 | cut -d'"' -f4)
curl -fL "$ASSET_URL" -o ~/.local/share/notifytrap/NotifyTrap.AppImage
chmod +x ~/.local/share/notifytrap/NotifyTrap.AppImage
ln -sf ~/.local/share/notifytrap/NotifyTrap.AppImage ~/.local/bin/notifytrap
```

```bash [macOS (Universal .dmg)]
ASSET_URL=$(curl -fsSL https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest \
  | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+\.dmg"' \
  | head -n 1 | cut -d'"' -f4)
curl -fL "$ASSET_URL" -o /tmp/NotifyTrap.dmg
MOUNT_DIR=$(hdiutil attach /tmp/NotifyTrap.dmg -nobrowse -readonly | awk '/\/Volumes\// { idx = index($0, "/Volumes/"); print substr($0, idx); exit }')
rm -rf /Applications/NotifyTrap.app && cp -R "$MOUNT_DIR/NotifyTrap.app" /Applications/
hdiutil detach "$MOUNT_DIR" -quiet && rm -f /tmp/NotifyTrap.dmg
```

```powershell [Windows (.exe)]
$asset = (Invoke-RestMethod https://api.github.com/repos/farshidrezaei/notifytrap/releases/latest).assets `
  | Where-Object { $_.name -match '-setup\.exe$' } | Select-Object -First 1
$installer = Join-Path $env:TEMP $asset.name
Invoke-WebRequest $asset.browser_download_url -OutFile $installer
Start-Process $installer -Wait
```

:::

## 60-Second First Run

1. **Create Workspace & Project**: On first launch, complete the onboarding card to provision your first workspace and project with `local`, `development`, `testing`, and `staging` environments.
2. **Create a Channel**: Open **Channels** and create an **Email**, **SMS**, **Push**, or **Webhook** channel.
3. **Copy Credentials**:
   - **Email** displays **SMTP Username (`MAIL_USERNAME`)** and **SMTP Password (`MAIL_PASSWORD`)** matching `.env`.
   - **SMS / Push** display `Credential Identifier` and `X-Channel-Key / Composite Key` (`<identifier>.<secret>`).
   - **Webhook** displays the local endpoint URL and `X-Channel-Key`.
4. **Inspect Live**: Send a test notification and watch it stream into **Inbox** immediately.

## Signed In-App Updates

NotifyTrap verifies update payloads using minisign public key cryptography before applying updates. Silent background checks run after launch when enabled in **Settings → Application Updates** and present toast actions (`Download & Update`, `Restart Now`) directly on Windows, macOS, and Linux `.AppImage`.
