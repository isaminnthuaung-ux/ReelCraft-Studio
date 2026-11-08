# Linux and macOS downloads

GitHub Actions builds Linux x64 on Ubuntu 24.04 and macOS arm64 for Apple Silicon.
Download the matching artifact, then extract the included `.tar.gz` archive.
Run `bash START.sh` from the extracted platform folder. On macOS you can also
open `START.command`.

These archives contain native executables and launch scripts, not installers.
Keep all extracted files together. FFmpeg and ffprobe must be installed separately.

## Linux

Use Ubuntu 24.04 or a compatible system with WebKitGTK 4.1 and GTK 3:

```sh
sudo apt-get update
sudo apt-get install ffmpeg libwebkit2gtk-4.1-0 libayatana-appindicator3-1
bash START.sh
```

## macOS

This build targets Apple Silicon, not Intel Macs. Install FFmpeg using Homebrew:

```sh
brew install ffmpeg
bash START.sh
```

The executable is unsigned and not notarized. macOS may require approval in
System Settings > Privacy & Security before it can run.

## Workflow

`.github/workflows/windows-portable.yml` builds all three platforms on pushes
to `main`, pull requests targeting `main`, version tags (`v*`), and manual runs.
Download packages from the run's Artifacts section. Version tags publish all
packages to a GitHub Release only after every platform succeeds.

Windows includes FFmpeg and ffprobe. Linux and macOS use the tools installed
on the destination computer. Each platform download includes SHA-256 checksums.
