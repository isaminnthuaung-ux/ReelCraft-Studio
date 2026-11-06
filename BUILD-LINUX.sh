#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")"
if [[ "$(uname -s)" != Linux ]]; then echo 'Run this script on Linux.' >&2; exit 1; fi
exec bash scripts/build-final-unix.sh Linux
