# ReelCraft Studio

[မြန်မာ (မူလ)](README.md) | **English**

**Make long video to short.** A standalone Windows video splitter rebuilt with Rust, Tauri and a modern desktop interface. No original recap application files are changed or required at runtime.

## Run

The app is now named **ReelCraft Studio** and uses the supplied `reelcraft.ico`. The existing project directory and internal settings identifier are retained so saved fonts/settings and export checkpoints remain available.

**Larger-font UI update:** run **START-UI.bat** after your current export finishes and you close the old window. The updated executable uses readable Windows/Segoe UI typography and the requested VidCoder toolbar icons. Both launchers now select the newest build. It does not replace or stop an already-running export. Build it with **BUILD-UI.bat**; icon attribution and license are in `public/icons/vidcoder/`.

Double-click **START.bat** after building. The executable is:

`src-tauri/target/release/ReelCraftStudio.exe`

Microsoft Edge WebView2 Runtime is required. FFmpeg and ffprobe must be in `bin/` beside the executable or on PATH. This build also recognizes the standard Chocolatey FFmpeg location. `BUILD.bat` copies available FFmpeg tools into the release folder for this machine; review the tool's license/distribution requirements before redistribution.

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

## Build from source

Requirements: Node.js/npm, Rust MSVC toolchain, Microsoft C++ Build Tools with Windows SDK, Edge WebView2, and FFmpeg/ffprobe.

```powershell
npm.cmd install
.\BUILD.bat
```

For development, run `DEV.bat`. The repository supports a project-local Rust installation in `.tools/`; no system PATH change is needed. `scripts/env.ps1` prepares the build environment for the current process only. It includes MSVC/SDK discovery for this machine's incomplete installer metadata; a normal developer shell also works.

```powershell
# In a Rust-enabled developer shell:
cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1
npm.cmd run build
npm.cmd run test:ui
```

UI tests use the installed Microsoft Edge browser. They cover interface behavior and a mocked desktop bridge; native Rust tests cover actual FFmpeg rendering, Unicode overlays, output reuse and cancellation. `npm run dev` in a normal browser intentionally disables native video operations.

## Current release scope

Windows only. No Gemini/Whisper/TTS, social accounts, publishing, posting quotas or modifications to the Python recap application. The larger design is in `PLAN.md`; features such as manual boundary editing, advanced typography, project persistence and manual encoder selection remain future work; automatic NVIDIA/CPU selection is implemented.
