#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")"
if [[ "$(uname -s)" != Darwin ]]; then echo 'Run this script on macOS.' >&2; exit 1; fi
exec bash scripts/build-final-unix.sh macOS
