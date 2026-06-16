#!/usr/bin/env bash
# package-portable.sh
# Compile portable version and package as tar.gz on Linux
# Usage: chmod +x package-portable.sh && ./package-portable.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"
RELEASE_DIR="$PROJECT_ROOT/src-tauri/target/release"
EXE_NAME='keyboard-locker'
PUBLISH_DIR="$PROJECT_ROOT/publish/KeyboardLocker-Portable"

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

info() {
    echo -e "${CYAN}$1${NC}"
}

success() {
    echo -e "${GREEN}$1${NC}"
}

warn() {
    echo -e "${YELLOW}$1${NC}"
}

error() {
    echo -e "${RED}$1${NC}" >&2
}

info '=== 1/4 Install npm dependencies ==='
npm install

info '=== 2/4 Build frontend (Vue + Vite) ==='
npm run build

info '=== 3/4 Build Tauri app (Rust release) ==='
cd "$PROJECT_ROOT/src-tauri"
cargo build --release

if [ ! -f "$RELEASE_DIR/$EXE_NAME" ]; then
    error "Cannot find $RELEASE_DIR/$EXE_NAME, build may have failed"
    exit 1
fi

info '=== 4/4 Package portable tar.gz ==='
rm -rf "$PUBLISH_DIR"
mkdir -p "$PUBLISH_DIR"

cp "$RELEASE_DIR/$EXE_NAME" "$PUBLISH_DIR/"
if [ -f "$PROJECT_ROOT/README.md" ]; then
    cp "$PROJECT_ROOT/README.md" "$PUBLISH_DIR/"
fi

VERSION="0.1.0"
TAR_PATH="$PROJECT_ROOT/publish/KeyboardLocker-Portable-v${VERSION}-linux-x64.tar.gz"
rm -f "$TAR_PATH"

cd "$PROJECT_ROOT/publish"
tar -czf "$TAR_PATH" -C "$PROJECT_ROOT/publish" "$(basename "$PUBLISH_DIR")"

SIZE_KB=$(du -k "$TAR_PATH" | cut -f1)

echo ''
success '=== Done! ==='
success "Portable tar.gz: $TAR_PATH"
success "Size: ${SIZE_KB} KB"