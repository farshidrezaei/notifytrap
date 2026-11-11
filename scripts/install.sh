#!/usr/bin/env bash
set -euo pipefail

REPOSITORY="farshidrezaei/notifytrap"
APPLICATION_NAME="NotifyTrap"
SELECTED_METHOD="auto"
REQUESTED_VERSION="latest"
TEMPORARY_DIRECTORY=""
MOUNT_DIRECTORY=""

print_usage() {
  cat <<'EOF'
NotifyTrap CLI Installer (Linux & macOS)

Usage:
  install.sh [--method auto|deb|rpm|appimage|dmg] [--version latest|v1.0.0]

Options:
  --method   Force a specific package format (default: auto)
  --version  Install a specific GitHub release tag (default: latest)
  --help     Show this help message
EOF
}

log_info() {
  printf '[NotifyTrap] %s\n' "$1"
}

fail_with_error() {
  printf '[NotifyTrap Error] %s\n' "$1" >&2
  exit 1
}

cleanup() {
  if [[ -n "${MOUNT_DIRECTORY}" ]]; then
    hdiutil detach "${MOUNT_DIRECTORY}" -quiet >/dev/null 2>&1 || true
  fi
  if [[ -n "${TEMPORARY_DIRECTORY}" && -d "${TEMPORARY_DIRECTORY}" ]]; then
    rm -rf "${TEMPORARY_DIRECTORY}"
  fi
}

trap cleanup EXIT

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    fail_with_error "Required command '$1' was not found in PATH."
  fi
}

parse_arguments() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --method)
        [[ $# -lt 2 ]] && fail_with_error "--method requires a value."
        SELECTED_METHOD="$2"
        shift 2
        ;;
      --version)
        [[ $# -lt 2 ]] && fail_with_error "--version requires a tag value (for example: v1.0.0)."
        REQUESTED_VERSION="$2"
        shift 2
        ;;
      --help|-h)
        print_usage
        exit 0
        ;;
      *)
        fail_with_error "Unknown argument: $1"
        ;;
    esac
  done
}

detect_operating_system() {
  local kernel_name
  kernel_name="$(uname -s)"
  case "${kernel_name}" in
    Linux) printf 'linux' ;;
    Darwin) printf 'macos' ;;
    *) fail_with_error "Unsupported operating system: ${kernel_name}" ;;
  esac
}

detect_architecture() {
  local machine_name
  machine_name="$(uname -m)"
  case "${machine_name}" in
    x86_64|amd64) printf 'x86_64' ;;
    aarch64|arm64) printf 'aarch64' ;;
    *) fail_with_error "Unsupported CPU architecture: ${machine_name}" ;;
  esac
}

resolve_linux_method() {
  if [[ "${SELECTED_METHOD}" != "auto" ]]; then
    printf '%s' "${SELECTED_METHOD}"
    return
  fi

  if command -v apt-get >/dev/null 2>&1 && command -v dpkg >/dev/null 2>&1; then
    printf 'deb'
  elif command -v dnf >/dev/null 2>&1 || command -v zypper >/dev/null 2>&1 || command -v yum >/dev/null 2>&1; then
    printf 'rpm'
  else
    printf 'appimage'
  fi
}

fetch_release_metadata() {
  local api_url
  if [[ "${REQUESTED_VERSION}" == "latest" ]]; then
    api_url="https://api.github.com/repos/${REPOSITORY}/releases/latest"
  else
    local normalized_tag="${REQUESTED_VERSION}"
    [[ "${normalized_tag}" != v* ]] && normalized_tag="v${normalized_tag}"
    api_url="https://api.github.com/repos/${REPOSITORY}/releases/tags/${normalized_tag}"
  fi

  curl -fsSL \
    -H "Accept: application/vnd.github+json" \
    -H "User-Agent: NotifyTrap-CLI-Installer" \
    "${api_url}"
}

extract_asset_url() {
  local release_json="$1"
  local pattern="$2"

  printf '%s' "${release_json}" \
    | grep -Eo '"browser_download_url":[[:space:]]*"[^"]+"' \
    | sed -E 's/^"browser_download_url":[[:space:]]*"([^"]+)"$/\1/' \
    | grep -vE '\.(sig|tar\.gz|sha256)$' \
    | grep -E "${pattern}" \
    | head -n 1 || true
}

run_with_privilege() {
  if [[ "$(id -u)" -eq 0 ]]; then
    "$@"
  else
    require_command sudo
    sudo "$@"
  fi
}

install_linux_deb() {
  local asset_url="$1"
  local deb_path="${TEMPORARY_DIRECTORY}/notifytrap.deb"
  log_info "Downloading Debian package from ${asset_url}"
  curl -fL --progress-bar "${asset_url}" -o "${deb_path}"
  log_info "Installing package via apt-get..."
  run_with_privilege apt-get update
  run_with_privilege apt-get install -y "${deb_path}"
}

install_linux_rpm() {
  local asset_url="$1"
  local rpm_path="${TEMPORARY_DIRECTORY}/notifytrap.rpm"
  log_info "Downloading RPM package from ${asset_url}"
  curl -fL --progress-bar "${asset_url}" -o "${rpm_path}"
  log_info "Installing package..."
  if command -v dnf >/dev/null 2>&1; then
    run_with_privilege dnf install -y "${rpm_path}"
  elif command -v zypper >/dev/null 2>&1; then
    run_with_privilege zypper --non-interactive install "${rpm_path}"
  elif command -v yum >/dev/null 2>&1; then
    run_with_privilege yum localinstall -y "${rpm_path}"
  else
    run_with_privilege rpm -Uvh "${rpm_path}"
  fi
}

install_linux_appimage() {
  local asset_url="$1"
  local install_directory="${HOME}/.local/share/notifytrap"
  local binary_directory="${HOME}/.local/bin"
  local applications_directory="${HOME}/.local/share/applications"
  local appimage_path="${install_directory}/NotifyTrap.AppImage"
  local symlink_path="${binary_directory}/notifytrap"
  local desktop_entry_path="${applications_directory}/notifytrap.desktop"

  mkdir -p "${install_directory}" "${binary_directory}" "${applications_directory}"

  log_info "Downloading AppImage from ${asset_url}"
  curl -fL --progress-bar "${asset_url}" -o "${appimage_path}"
  chmod +x "${appimage_path}"
  ln -sf "${appimage_path}" "${symlink_path}"

  cat >"${desktop_entry_path}" <<EOF
[Desktop Entry]
Name=${APPLICATION_NAME}
Comment=Local notification capture desktop utility
Exec=${appimage_path} %U
Terminal=false
Type=Application
Categories=Development;Utility;
StartupWMClass=${APPLICATION_NAME}
EOF

  log_info "Installed AppImage to ${appimage_path}"
  log_info "Command symlink created at ${symlink_path}"
  if [[ ":${PATH}:" != *":${binary_directory}:"* ]]; then
    log_info "Add '${binary_directory}' to your PATH to launch with 'notifytrap'."
  fi
}

install_macos_dmg() {
  local asset_url="$1"
  local dmg_path="${TEMPORARY_DIRECTORY}/NotifyTrap.dmg"
  local target_bundle="/Applications/${APPLICATION_NAME}.app"

  log_info "Downloading macOS installer from ${asset_url}"
  curl -fL --progress-bar "${asset_url}" -o "${dmg_path}"

  log_info "Mounting disk image..."
  MOUNT_DIRECTORY="$(
    hdiutil attach "${dmg_path}" -nobrowse -readonly \
      | awk '/\/Volumes\// { idx = index($0, "/Volumes/"); print substr($0, idx); exit }'
  )"

  if [[ -z "${MOUNT_DIRECTORY}" || ! -d "${MOUNT_DIRECTORY}/${APPLICATION_NAME}.app" ]]; then
    fail_with_error "Could not locate ${APPLICATION_NAME}.app inside downloaded disk image."
  fi

  log_info "Installing ${APPLICATION_NAME}.app to /Applications..."
  if [[ -w "/Applications" ]]; then
    rm -rf "${target_bundle}"
    cp -R "${MOUNT_DIRECTORY}/${APPLICATION_NAME}.app" "${target_bundle}"
  else
    run_with_privilege rm -rf "${target_bundle}"
    run_with_privilege cp -R "${MOUNT_DIRECTORY}/${APPLICATION_NAME}.app" "${target_bundle}"
  fi

  hdiutil detach "${MOUNT_DIRECTORY}" -quiet
  MOUNT_DIRECTORY=""
  log_info "Installed ${target_bundle}."
}

main() {
  parse_arguments "$@"
  require_command curl
  require_command grep
  require_command sed

  TEMPORARY_DIRECTORY="$(mktemp -d)"

  local operating_system
  local architecture
  local release_json
  operating_system="$(detect_operating_system)"
  architecture="$(detect_architecture)"

  log_info "Fetching release metadata for ${REPOSITORY} (${REQUESTED_VERSION})..."
  release_json="$(fetch_release_metadata)"

  if [[ "${operating_system}" == "macos" ]]; then
    if [[ "${SELECTED_METHOD}" != "auto" && "${SELECTED_METHOD}" != "dmg" ]]; then
      fail_with_error "Method '${SELECTED_METHOD}' is not supported on macOS. Use 'dmg' or 'auto'."
    fi
    local dmg_url
    dmg_url="$(extract_asset_url "${release_json}" '(universal|aarch64|x64|x86_64)?\.dmg$')"
    [[ -z "${dmg_url}" ]] && fail_with_error "No macOS .dmg asset found in the selected GitHub release."
    install_macos_dmg "${dmg_url}"
    log_info "Installation complete. Open NotifyTrap from Applications or Spotlight."
    return
  fi

  local linux_method
  linux_method="$(resolve_linux_method)"

  local arch_regex="amd64|x86_64"
  [[ "${architecture}" == "aarch64" ]] && arch_regex="arm64|aarch64"

  case "${linux_method}" in
    deb)
      local deb_url
      deb_url="$(extract_asset_url "${release_json}" "(${arch_regex}).*\\.deb$|\\.deb$")"
      [[ -z "${deb_url}" ]] && fail_with_error "No .deb release asset found for ${architecture}."
      install_linux_deb "${deb_url}"
      ;;
    rpm)
      local rpm_url
      rpm_url="$(extract_asset_url "${release_json}" "(${arch_regex}).*\\.rpm$|\\.rpm$")"
      [[ -z "${rpm_url}" ]] && fail_with_error "No .rpm release asset found for ${architecture}."
      install_linux_rpm "${rpm_url}"
      ;;
    appimage)
      local appimage_url
      appimage_url="$(extract_asset_url "${release_json}" "(${arch_regex}).*\\.AppImage$|\\.AppImage$")"
      [[ -z "${appimage_url}" ]] && fail_with_error "No .AppImage release asset found for ${architecture}."
      install_linux_appimage "${appimage_url}"
      ;;
    *)
      fail_with_error "Unsupported Linux install method: ${linux_method} (expected: auto, deb, rpm, appimage)."
      ;;
  esac

  log_info "Installation complete."
}

main "$@"
