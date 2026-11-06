# ReelCraft Studio

[မြန်မာ (မူလ)](README.md) | **English**

**Make long video to short.** A standalone Windows video splitter rebuilt with Rust, Tauri and a modern desktop interface. No original recap application files are changed or required at runtime.

## Run the final app

| OS | Open the final build |
| --- | --- |
| Windows | Double-click `FinalBuild/Windows/ReelCraftStudio.exe`. |
| Linux | Run `bash FinalBuild/Linux/START.sh`. |
| macOS | Open `FinalBuild/macOS/START.command`, or run `bash FinalBuild/macOS/START.sh`. |

The destination machine needs **FFmpeg and ffprobe** installed and available on PATH. Windows also needs **Microsoft Edge WebView2 Runtime**. Linux needs WebKitGTK runtime libraries. Install and select a Myanmar Unicode font for Myanmar text overlays.

### Build from source

Run only the script for your OS from the project directory. It runs `npm ci` and creates a native release executable.

| Host OS | Command | Output folder |
| --- | --- | --- |
| Windows | `.\BUILD-WINDOWS.bat` | `FinalBuild/Windows/` |
| Linux | `bash BUILD-LINUX.sh` | `FinalBuild/Linux/` |
| macOS | `bash BUILD-MAC.sh` | `FinalBuild/macOS/` |

Install prerequisites first:

- All platforms: Node.js/npm, Rust/Cargo, FFmpeg, and ffprobe.
- Windows: Microsoft C++ Build Tools with Windows SDK, plus Edge WebView2.
- Linux: Tauri 2 development dependencies for your distribution and system fonts.
- macOS: Xcode Command Line Tools (`xcode-select --install`). With Homebrew installed, use `brew install ffmpeg` for media tools.

Ubuntu/Debian dependency example:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev librsvg2-dev libayatana-appindicator3-dev ffmpeg fontconfig fonts-dejavu fonts-noto-core
```

See [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for setup. Build each platform on its own OS, for the host CPU architecture. These scripts produce portable executables, not installers, macOS `.app` bundles, or DMGs. Linux/macOS native builds and runtime have not been tested on this Windows machine.

Older `BUILD.bat`, `BUILD-UI.bat`, and `START*.bat` remain for existing development workflows. Use the scripts above for final output.

## Workflow

The workspace shows Preview and Your parts as side-by-side cards, with a wider preview and a narrow vertically scrolling parts list.

1. Add videos, add a folder, or drag files/folders into the window.
2. Set part length in seconds. Enable Smart Cut to search a bounded extension window for a pause.
3. Select Original, 9:16, 16:9 or 1:1 framing. Non-original ratios support blur, black bars and center crop.
4. In **Text studio**, select the title or part-label layer. Choose from the searchable Windows fonts list (system and current-user installed fonts), or optionally import TTF/OTF/TTC files. Change text, size, color, opacity, background box and position. Drag the text handle in the preview to position it.
5. Analyze cuts and review the parts. Deselect parts you do not want to export.
6. Choose an output folder and export. Open Activity for processing details.
7. Stop safely if needed. Run the same source/settings/output again to reuse verified completed parts.

The first version provides frame previews with a seek slider, not real-time playback. Settings and imported fonts persist in this app's own Windows application-data folder. Source lists and manual part selections are session-only; reimport and analyze after reopening. Matching disk exports remain reusable.

## Fonts and Unicode

Both text preview and export use the same Rust `cosmic-text` shaping/rasterization engine. The overlay is rendered at export resolution, then displayed scaled in the preview. Myanmar complex-script text and mixed Latin names are included in native integration tests.

- Imported fonts are copied into this app's private font library.
- The Windows font picker renders only visible rows and uses Segoe UI for names, avoiding loading every installed font on open. Search and keyboard navigation are supported. The video overlay still renders in the selected font.

- Font coverage varies. The app reports fallback font use and missing glyphs; export refuses unresolved missing-glyph output.
- Family selection is supported in v0.1; separate face/weight selection, stroke/shadow, variable-axis controls and color emoji are not promised in this release.
- Part labels support `{n}`, `{total}` and `{mm}`. Using `{mm}` also formats the total with Myanmar digits.
- The new app does not strip English words or translate narration.

## Output safety

Each source has a path-hash output folder. Each rendered part has a settings/content fingerprint suffix. Changed settings generate new artifacts instead of overwriting earlier exports. A checkpoint records completed outputs, which are probed before reuse. Old exports are never automatically deleted.

Export automatically probes NVIDIA NVENC with a real encode and uses it when available (p5, VBR, CQ 21). If hardware initialization or encoding fails, it falls back to CPU H.264 (`libx264`, veryfast, CRF 20). Source audio is encoded as AAC. Processing remains local and sequential. Progress updates during each part, with encoder and speed shown; cancellation interrupts the active analysis/render process. The preview and export share the optimized background-filter path. Completed parts from the earlier release remain reusable when the source and settings match.

Smart Cut uses audio energy, not speech recognition. It cannot guarantee sentence completion. Constant loud audio uses exact cuts; silent audio uses the requested boundary. A final short remainder may merge into the previous part, matching the existing splitter concept.

## Development / Tests

Final build: see the OS-specific scripts above. Development: `npm run tauri -- dev`.

```text
cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1
npm run build
npm run test:ui
```

## Current release scope

Windows is the currently validated platform; Linux/macOS validation is pending. No Gemini/Whisper/TTS, social accounts, publishing, posting quotas or modifications to the Python recap application. The larger design is in `PLAN.md`; features such as manual boundary editing, advanced typography, project persistence and manual encoder selection remain future work; automatic NVIDIA/CPU selection is implemented.
