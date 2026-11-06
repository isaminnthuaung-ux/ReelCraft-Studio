# v0.1 validation — 2026-10-26

Built and checked on Windows x64 with a project-local Rust MSVC toolchain, Node.js, Edge WebView2, and the available FFmpeg distribution.

- TypeScript check and Vite production build: passed.
- Rust compiler check: passed.
- Rust tests: **15 passed**, including actual video encoding/decoding.
- Playwright interface tests: **3 passed**, using Microsoft Edge; browser-preview, smaller-window readability, and mocked desktop import/plan/select/export workflows.
- Actual packaged Tauri smoke test: desktop file-drop event, real native probing, frame extraction, Myanmar overlay rendering, split planning, and three MP4 exports passed.
- The standalone text smoke sample produced no missing-glyph/fallback warnings with the installed Myanmar Text font. The resulting PNG was visually inspected. This is not certification for every font/script.

Native coverage includes source audio preservation, no-audio portrait/blur export, Unicode source filenames, Myanmar/Latin overlays, consistent Myanmar part numbering, completed-file reuse, cancellation of active FFmpeg work, source-folder collision avoidance, constant/silent audio and cut-window bounds.

Tests use temporary generated footage; the native desktop smoke script stores development artifacts under ignored `.tools/native-smoke/`. Remote debugging is enabled only by the developer's smoke-test launch command; START.bat does not enable it. Native OS picker interaction is covered through the UI bridge mock, not OS-level dialog automation.

Current limits: frame preview rather than video playback; source list and part selection are session-only; manual boundary editing and advanced text effects are not implemented. Settings/imported fonts persist separately from the original app.

No source files in the original Python recap application were edited during this rebuild. The new app is entirely under `LongToShotStudio/`.

## Export performance update

- Native integration tests used NVIDIA NVENC successfully on the RTX 3050. CPU fallback policy and streaming cancellation are covered.
- Blur is computed at one quarter of each output dimension and skipped when the source aspect ratio already fits. Empty overlays are omitted; visible overlay images are cropped before compositing.
- In-part progress is duration-weighted, monotonic, and retained across a GPU-to-CPU retry. High-frequency progress events do not flood Activity.
- Resume tests cover both the new cache and the previous release's fingerprint/record format.
- Single-run synthetic benchmark: 8 seconds of 1280x720, 30 fps footage exported as 1080x1920 with blurred background and text disabled. Previous Rust path: **2.702 s**; optimized CPU: **0.797 s**; optimized NVIDIA: **1.491 s**. This short fixture favors low CPU startup overhead; quality modes differ, and these measurements do not guarantee speed on other videos.
- Reproduce with `node scripts/benchmark-export.mjs`. Benchmarks write only ignored synthetic fixtures under `.tools/export-benchmark/`.

The final release executable also passed a native three-part export using NVIDIA NVENC with live in-part progress events. Both START.bat and START-UI.bat select the newest compiled executable.

## Font picker responsiveness ? 2026-10-27

Opening the old picker mounted every font row and applied its own font family, potentially forcing WebView to load hundreds of faces. The picker now virtualizes 36 px rows with a small overscan and renders names in Segoe UI. Search indexing is memoized. Tests exercise a 502-family catalog, bounded rendered row count, the last option, reopening the selection, empty search results and normal font selection. Actual video text rendering and export are unchanged.
