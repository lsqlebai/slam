#!/usr/bin/env bash
set -euo pipefail

WEB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$WEB_DIR/dist"
RELEASE_DIR="$WEB_DIR/release"
VERSION="$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"[:space:]]*\)".*/\1/p' "$WEB_DIR/package.json" | head -n 1)"
if [ -z "$VERSION" ]; then VERSION="0.0.0"; fi
OUT_GZ="$RELEASE_DIR/slam_web-v$VERSION.tar.gz"

if [ ! -d "$DIST_DIR" ]; then
  echo "dist目录不存在"
  exit 1
fi

mkdir -p "$RELEASE_DIR"
tar -C "$WEB_DIR" -czf "$OUT_GZ" dist
echo "$OUT_GZ"
