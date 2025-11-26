#!/usr/bin/env bash
set -euo pipefail

NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
nvm use 22.16.0

WEB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$WEB_DIR/dist"
RELEASE_DIR="$WEB_DIR/release"
VERSION="$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"[:space:]]*\)".*/\1/p' "$WEB_DIR/package.json" | head -n 1)"
if [ -z "$VERSION" ]; then VERSION="0.0.0"; fi
OUT_GZ="$RELEASE_DIR/slam_web-v$VERSION.tar.gz"

if ! command -v pnpm >/dev/null 2>&1; then
  command -v corepack >/dev/null 2>&1 && corepack enable || true
  PM_SPEC="$(sed -n 's/.*"packageManager"[[:space:]]*:[[:space:]]*"\([^"[:space:]]*\)".*/\1/p' "$WEB_DIR/package.json" | head -n 1)"
  if [ -n "$PM_SPEC" ]; then
    corepack prepare "$PM_SPEC" --activate || true
  else
    corepack prepare pnpm@10.23.0 --activate || true
  fi
fi

cd "$WEB_DIR"
pnpm install --frozen-lockfile
pnpm build

if [ ! -d "$DIST_DIR" ]; then
  echo "dist目录不存在"
  exit 1
fi

mkdir -p "$RELEASE_DIR"
tar -C "$WEB_DIR" -czf "$OUT_GZ" dist
echo "$OUT_GZ"
