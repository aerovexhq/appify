#!/usr/bin/env bash
set -euo pipefail

# appify unified release script
# Builds and stages Linux release assets:
# - appify_<version>_amd64.AppImage
# - appify_<version>_amd64.deb
# - appify_linux_x64
# - appify-<version>-linux-x86_64.tar.gz

REPO="aerovexhq/appify"
SOURCE="${BASH_SOURCE[0]}"
while [ -h "$SOURCE" ]; do
  DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
  SOURCE="$(readlink "$SOURCE")"
  [[ $SOURCE != /* ]] && SOURCE="$DIR/$SOURCE"
done
SCRIPT_DIR="$(cd -P "$(dirname "$SOURCE")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"

# Determine target release tag / version
TAG="${1:-}"
if [ -z "${TAG}" ]; then
  TAG=$(gh release view -R "${REPO}" --json tagName -q .tagName 2>/dev/null || true)
  if [ -z "${TAG}" ]; then
    TAG=$(grep -m1 '^version' Cargo.toml | awk -F '"' '{print $2}')
  fi
fi

VERSION="${TAG#v}"
echo "==> Preparing release for appify version ${VERSION} (tag: ${TAG})..."

command -v gh >/dev/null 2>&1 || {
  echo "Error: gh CLI not found in PATH." >&2
  exit 1
}

DIST_DIR="${ROOT_DIR}/dist-release"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

echo "==> Building Appify Linux packages via Tauri bundler..."
npx -y @tauri-apps/cli build --bundles deb,appimage

# Copy Deb and AppImage
DEB_SRC=$(find target/release/bundle/deb -name "*.deb" | head -n 1)
APPIMAGE_SRC=$(find target/release/bundle/appimage -name "*.AppImage" | head -n 1)

if [ -f "${DEB_SRC}" ]; then
  cp "${DEB_SRC}" "${DIST_DIR}/appify_${VERSION}_amd64.deb"
fi

if [ -f "${APPIMAGE_SRC}" ]; then
  cp "${APPIMAGE_SRC}" "${DIST_DIR}/appify_${VERSION}_amd64.AppImage"
fi

# Standalone binary
cp target/release/appify "${DIST_DIR}/appify_linux_x64"
strip "${DIST_DIR}/appify_linux_x64" || true

# Portable Tarball
TAR_DIR="${ROOT_DIR}/target/tarball_staging"
rm -rf "${TAR_DIR}"
mkdir -p "${TAR_DIR}/icons"

cp target/release/appify "${TAR_DIR}/appify"
chmod +x "${TAR_DIR}/appify"
cp install.sh "${TAR_DIR}/install.sh" 2>/dev/null || true
cp README.md LICENSE "${TAR_DIR}/" 2>/dev/null || true
cp -r icons/* "${TAR_DIR}/icons/" 2>/dev/null || true

tar -czf "${DIST_DIR}/appify-${VERSION}-linux-x86_64.tar.gz" -C "${TAR_DIR}" .

echo "==> Staged release assets in dist-release/:"
ls -lh "${DIST_DIR}"

# Check upload flag (default: true)
UPLOAD="${UPLOAD:-true}"
if [ "${UPLOAD}" = "true" ]; then
  echo "==> Uploading assets to GitHub release ${TAG} in ${REPO}..."
  if gh release view "${TAG}" -R "${REPO}" >/dev/null 2>&1; then
    gh release upload "${TAG}" "${DIST_DIR}"/* --clobber -R "${REPO}"
  else
    echo "Creating new release ${TAG}..."
    gh release create "${TAG}" "${DIST_DIR}"/* \
      --title "Appify ${VERSION}" \
      --notes "Turn web apps into lightweight desktop apps powered by Tauri." \
      -R "${REPO}"
  fi
  echo "==> Release assets successfully uploaded to ${REPO} @ ${TAG}!"
else
  echo "==> UPLOAD=false specified; skipping upload."
fi
