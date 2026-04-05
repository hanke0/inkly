#!/usr/bin/env bash
# Install Rust (via rustup) and Node.js 22 (via nvm if needed), then build the
# inkly release binary tuned for the current CPU (-C target-cpu=native).
# Optional: first argument = destination directory; the built binary is copied there.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$ROOT"

info() {
  printf '%s\n' "$*"
}

warn() {
  printf 'install.sh: %s\n' "$*" >&2
}

die() {
  warn "$*"
  exit 1
}

ensure_rust() {
  if command -v cargo >/dev/null 2>&1; then
    return 0
  fi

  if ! command -v curl >/dev/null 2>&1; then
    die "curl is required to install rustup; install curl and retry."
  fi

  info "Installing Rust toolchain (rustup, stable)..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

  # shellcheck source=/dev/null
  if [[ -f "${HOME}/.cargo/env" ]]; then
    source "${HOME}/.cargo/env"
  fi

  command -v cargo >/dev/null 2>&1 || die "cargo not on PATH after rustup; run: source \"\$HOME/.cargo/env\""
}

node_major_ok() {
  command -v node >/dev/null 2>&1 || return 1
  local major
  major="$(node -p "Number(process.versions.node.split('.')[0])" 2>/dev/null || echo 0)"
  [[ "${major}" -ge 22 ]]
}

ensure_nvm() {
  export NVM_DIR="${NVM_DIR:-${HOME}/.nvm}"
  if [[ -s "${NVM_DIR}/nvm.sh" ]]; then
    # shellcheck source=/dev/null
    source "${NVM_DIR}/nvm.sh"
    return 0
  fi

  if ! command -v curl >/dev/null 2>&1; then
    die "curl is required to install nvm; install curl and retry."
  fi

  info "Installing nvm (Node version manager)..."
  local nvm_ver="v0.40.1"
  curl -fsSL "https://raw.githubusercontent.com/nvm-sh/nvm/${nvm_ver}/install.sh" | bash

  # shellcheck source=/dev/null
  source "${NVM_DIR}/nvm.sh"
}

ensure_node() {
  if node_major_ok; then
    info "Node $(node -v) is already available (>= 22)."
    return 0
  fi

  ensure_nvm
  info "Installing Node.js 22 (LTS line) via nvm..."
  nvm install 22
  nvm use 22
  nvm alias default 22 >/dev/null 2>&1 || true

  node_major_ok || die "Node 22+ required after install; check nvm and your PATH."
}

ensure_rust
ensure_node

export RUSTFLAGS="${RUSTFLAGS:--C target-cpu=native}"

info "Building inkly (release, locked), RUSTFLAGS=${RUSTFLAGS}..."
cargo build --locked --release -p inkly

binary="${ROOT}/target/release/inkly"
if [[ -f "${binary}" ]]; then
  if command -v strip >/dev/null 2>&1; then
    case "$(uname -s 2>/dev/null || true)" in
      Darwin | Linux)
        strip "${binary}" || warn "strip failed (binary still at ${binary})"
        ;;
    esac
  fi
  info "Done: ${binary}"

  if [[ -n "${1:-}" ]]; then
    dest_dir="$1"
    if [[ -e "${dest_dir}" && ! -d "${dest_dir}" ]]; then
      die "Destination exists but is not a directory: ${dest_dir}"
    fi
    mkdir -p "${dest_dir}"
    dest_dir="$(cd "${dest_dir}" && pwd)"
    cp -f "${binary}" "${dest_dir}/"
    info "Copied binary to: ${dest_dir}/$(basename "${binary}")"
  fi
else
  die "Expected binary missing: ${binary}"
fi
