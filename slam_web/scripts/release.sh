#!/usr/bin/env bash
set -euo pipefail

NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
[ -s "$NVM_DIR/nvm.sh" ] && . "$NVM_DIR/nvm.sh"
nvm use 22.16.0

WEB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
RELEASE_DIR="$WEB_DIR/release"
VERSION="$(sed -n 's/.*"version"[[:space:]]*:[[:space:]]*"\([^"[:space:]]*\)".*/\1/p' "$WEB_DIR/package.json" | head -n 1)"
if [ -z "$VERSION" ]; then VERSION="0.0.0"; fi
OUT_GZ="$RELEASE_DIR/slam_web-v$VERSION.tar.gz"
OUT_ZIP="$RELEASE_DIR/slam_web-v$VERSION.zip"

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

# Build releases in an isolated directory so a running dev server cannot write
# source maps or other development-only files into the release archive.
RELEASE_WORK_DIR="$(mktemp -d "$WEB_DIR/.release-build.XXXXXX")"
trap 'rm -rf "$RELEASE_WORK_DIR"' EXIT
RELEASE_DIST_DIR="$RELEASE_WORK_DIR/dist"
SLAM_RELEASE_DIST="$RELEASE_DIST_DIR" pnpm build

if [ ! -d "$RELEASE_DIST_DIR" ]; then
  echo "dist目录不存在"
  exit 1
fi

if find "$RELEASE_DIST_DIR" -type f -name '*.map' -print -quit | grep -q .; then
  echo "生产构建包含source map，终止发布" >&2
  exit 1
fi

mkdir -p "$RELEASE_DIR"
rm -f "$OUT_GZ" "$OUT_ZIP"
tar -C "$RELEASE_WORK_DIR" -czf "$OUT_GZ" dist
if command -v zip >/dev/null 2>&1; then
  (cd "$RELEASE_WORK_DIR" && zip -r -q "$OUT_ZIP" dist)
elif command -v ditto >/dev/null 2>&1; then
  # macOS fallback
  (cd "$RELEASE_WORK_DIR" && ditto -c -k --sequesterRsrc --keepParent dist "$OUT_ZIP")
else
  echo "zip工具未找到，跳过zip生成" >&2
fi

echo "$OUT_GZ"
echo "$OUT_ZIP"
