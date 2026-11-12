#!/usr/bin/env bash
# Builds a TopCalc installer. Installs Rust when it is missing.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if ! command -v node >/dev/null 2>&1; then
  echo "Install Node.js 20 or newer, then run this script again: https://nodejs.org"
  exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs | sh -s -- -y
  # shellcheck disable=SC1091
  source "$HOME/.cargo/env"
fi

if [[ "$(uname -s)" == "Linux" ]] && command -v apt-get >/dev/null 2>&1; then
  sudo apt-get update
  sudo apt-get install -y libwebkit2gtk-4.1-dev build-essential libssl-dev \
    libayatana-appindicator3-dev librsvg2-dev patchelf libxdo-dev
fi

if [[ "$(uname -s)" == "Darwin" ]] && ! xcode-select -p >/dev/null 2>&1; then
  xcode-select --install
  echo "Finish the Xcode tools install, then run this script again."
  exit 1
fi

npm install
echo "Building TopCalc. The first build can take several minutes."
npm run tauri build
echo
echo "Done. Installers are in src-tauri/target/release/bundle"
