#!/bin/sh
set -eu

REPO="s1mhadri/braindump"

info() {
  printf "bd: %s\n" "$1"
}

warn() {
  printf "bd: warning: %s\n" "$1" >&2
}

error() {
  printf "bd: error: %s\n" "$1" >&2
  exit 1
}

download_stdout() {
  url="$1"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO- "$url"
  else
    error "Neither curl nor wget is available. Please install curl or wget."
  fi
}

download_file() {
  url="$1"
  output_file="$2"
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$url" -o "$output_file"
  elif command -v wget >/dev/null 2>&1; then
    wget -qO "$output_file" "$url"
  else
    error "Neither curl nor wget is available. Please install curl or wget."
  fi
}

detect_os() {
  os_raw="$(uname -s)"
  case "$os_raw" in
    Darwin)
      echo "apple-darwin"
      ;;
    Linux)
      echo "unknown-linux-gnu"
      ;;
    *)
      error "Unsupported operating system: $os_raw. braindump supports macOS and Linux."
      ;;
  esac
}

detect_arch() {
  arch_raw="$(uname -m)"
  case "$arch_raw" in
    x86_64|amd64)
      echo "x86_64"
      ;;
    aarch64|arm64)
      echo "aarch64"
      ;;
    *)
      error "Unsupported architecture: $arch_raw. braindump supports x86_64 and aarch64."
      ;;
  esac
}

resolve_tag() {
  if [ -n "${VERSION:-}" ]; then
    case "$VERSION" in
      v*) echo "$VERSION" ;;
      *)  echo "v$VERSION" ;;
    esac
    return 0
  fi

  info "Finding latest release..."
  latest_json="$(download_stdout "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null || true)"
  tag="$(echo "$latest_json" | grep '"tag_name":' | head -n 1 | sed -E 's/.*"tag_name":[[:space:]]*"([^"]+)".*/\1/' || true)"

  if [ -z "$tag" ]; then
    # Fallback to GitHub releases redirect URL if API is rate limited
    if command -v curl >/dev/null 2>&1; then
      redirect_url="$(curl -fsSIL -o /dev/null -w "%{url_effective}" "https://github.com/${REPO}/releases/latest" 2>/dev/null || true)"
      tag="$(echo "$redirect_url" | sed -E 's|.*/tag/([^/]+).*|\1|')"
    fi
  fi

  if [ -z "$tag" ]; then
    error "Failed to determine latest release tag for ${REPO}."
  fi

  echo "$tag"
}

main() {
  target_os="$(detect_os)"
  target_arch="$(detect_arch)"
  target="${target_arch}-${target_os}"
  tag="$(resolve_tag)"

  archive_name="bd-${tag}-${target}.tar.gz"
  download_url="https://github.com/${REPO}/releases/download/${tag}/${archive_name}"

  tmp_dir="$(mktemp -d 2>/dev/null || mktemp -d -t 'bd-install')"
  cleanup() {
    rm -rf "$tmp_dir"
  }
  trap cleanup EXIT INT TERM

  info "Downloading braindump ${tag} for ${target}..."
  archive_path="${tmp_dir}/${archive_name}"
  download_file "$download_url" "$archive_path" || error "Failed to download release archive from $download_url"

  tar -xzf "$archive_path" -C "$tmp_dir" || error "Failed to extract archive $archive_name"

  if [ ! -f "${tmp_dir}/bd" ]; then
    error "Archive did not contain the 'bd' binary."
  fi

  chmod +x "${tmp_dir}/bd"

  if [ -n "${INSTALL_DIR:-}" ]; then
    dest_dir="$INSTALL_DIR"
  elif [ -w "/usr/local/bin" ]; then
    dest_dir="/usr/local/bin"
  elif [ -d "$HOME/.local/bin" ] && [ -w "$HOME/.local/bin" ]; then
    dest_dir="$HOME/.local/bin"
  else
    dest_dir="$HOME/.local/bin"
  fi

  mkdir -p "$dest_dir" 2>/dev/null || true

  info "Installing bd into ${dest_dir}..."
  if [ -w "$dest_dir" ]; then
    mv "${tmp_dir}/bd" "${dest_dir}/bd"
  else
    info "Elevating permissions with sudo to write to ${dest_dir}..."
    sudo mv "${tmp_dir}/bd" "${dest_dir}/bd"
  fi

  chmod +x "${dest_dir}/bd" 2>/dev/null || sudo chmod +x "${dest_dir}/bd"

  info "Successfully installed bd ${tag} to ${dest_dir}/bd"

  case ":$PATH:" in
    *:"$dest_dir":*) ;;
    *)
      warn "${dest_dir} is not currently in your PATH."
      warn "Add it to your environment by adding this line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
      warn "  export PATH=\"${dest_dir}:\$PATH\""
      ;;
  esac
}

main "$@"
