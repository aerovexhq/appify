#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${ROOT_DIR}"
npm --prefix docs install
npm --prefix docs run docs:build
touch docs/.vitepress/dist/.nojekyll

echo "Appify documentation assembled in docs/.vitepress/dist"
