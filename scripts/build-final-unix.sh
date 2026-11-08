#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")/.."
platform="${1:?Use BUILD-LINUX.sh or BUILD-MAC.sh}"
case "$platform:$(uname -s)" in Linux:Linux|macOS:Darwin) ;; *) echo 'Platform mismatch.' >&2; exit 1 ;; esac
for tool in npm cargo ffmpeg ffprobe; do
    command -v "$tool" >/dev/null 2>&1 || { echo "Missing $tool. See README build requirements." >&2; exit 1; }
done
# Use the same lockfile and release profile on each host. No cross-compilation.
npm ci
npm run tauri -- build --no-bundle -- --locked
# Respect an explicitly configured Cargo output directory.
target_dir="$(cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps --format-version 1 | node -e 'let s="";process.stdin.on("data",d=>s+=d);process.stdin.on("end",()=>process.stdout.write(JSON.parse(s).target_directory))')"
output="FinalBuild/$platform"
mkdir -p "$output"
cp "$target_dir/release/long-to-shot-studio" "$output/ReelCraftStudio"
chmod +x "$output/ReelCraftStudio"
cat > "$output/START.sh" <<'LAUNCHER'
#!/usr/bin/env bash
set -euo pipefail
cd -- "$(dirname -- "$0")"
# Include usual Homebrew locations when started outside a login shell.
export PATH="/opt/homebrew/bin:/usr/local/bin:$PATH"
exec ./ReelCraftStudio
LAUNCHER
chmod +x "$output/START.sh"
if [[ "$platform" == macOS ]]; then cp "$output/START.sh" "$output/START.command"; chmod +x "$output/START.command"; fi
printf '\nFinal app: %s/ReelCraftStudio\nLaunch: bash %s/START.sh\n' "$output" "$output"
echo 'FFmpeg, ffprobe, system fonts and platform runtime libraries are required on the destination machine.'
echo 'This is a native executable, not an installer or a signed macOS .app.'
