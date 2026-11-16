# Export performance plan and validation — 1.1.0 Build 3

Priority: minimize export time. Larger output files are acceptable. Keep frame
geometry, source cadence, Unicode titles, transparency, cancellation, and resume.

User-facing output names are `Part {n} - {source stem}.mp4`, inside a folder
named after the source stem. Hashes remain in private resume records and temporary
filenames only. Verified exports from older hashed folders can be copied to the
clean names on resume without deleting the old files. Conflicting source names
or changed settings never silently overwrite existing output videos.

Each project folder also contains `part-titles.txt`: one exact completed video
filename per line, including `.mp4`, sorted by part number. It is independent of
the text overlay and is refreshed after each completed/reused part. Resume
recreates a missing list and retains names of earlier completed parts even when
only a subset is selected.

## Implemented

1. Inspect codec, pixel format and frame rate on import. Validate NVIDIA decoding
   with three real input frames and a hardware download, so a silent software
   decoder cannot be mistaken for NVDEC. Recheck the source before exporting.
2. Prepare static backgrounds once per source with the existing framing recipe.
   This preserves split-wallpaper pan/zoom and video positioning. Render titles
   once per part with the existing Unicode text engine.
3. Keep video frames on CUDA for scaling and overlays, then send hardware frames
   directly to NVENC. Upload static artwork once per FFmpeg process. Cache CUDA
   kernels across processes in the system temporary directory.
4. Retry failed hardware decoding with software decoding and GPU filters; retry
   failed GPU filters with CPU filters while retaining NVENC; finally fall back
   to ultrafast x264. Stop requests never trigger retries. Remember the working
   mode for subsequent parts of a source.
5. Keep NVENC P1 as the default. Preserve explicit user profile choices. Record
   the processing mode in export records and show it in progress.
6. Validate output duration AND exact dimensions. Preserve completed outputs from
   earlier renderer versions for resume. Bump package, Cargo, Tauri, window title,
   and changelog to 1.1.0 Build 3.

## Scope and remaining work

- Wallpaper, black and crop layouts use GPU scaling/compositing. The static
  background preparation and title rasterization still run once on the CPU.
- Dynamic blur still uses the existing CPU box blur. Supported 8-bit inputs use
  NVDEC followed by a download for that CPU graph. A matching-ratio blur layout
  at 100% scale does not need dynamic blur and can use the GPU graph.
- A fully GPU dynamic blur preserving the current appearance requires a new
  compatible GPU blur implementation/build and separate visual validation.
- Audio encoding, container muxing and file I/O remain ordinary CPU/I/O work.
- No claim of the globally fastest setting is made for every GPU/source. Cold
  CUDA kernel compilation can make a first short export slower; subsequent
  processes reuse the kernel cache. More concurrent exports remain unmeasured.

## Validation

- Rust: 23 tests, including real exports, Unicode text, resume, cancellation,
  split-wallpaper pixels, 60 FPS preservation, software-decoded CUDA uploads,
  hardware-decoded CUDA processing, alpha blending, and exact output dimensions.
- Frontend: TypeScript/Vite production build and four Playwright workflow/layout
  tests. Codec metadata remains optional when loading old project history.
- Local benchmark: RTX 3050, synthetic 1920×1080 AV1, 30 FPS, 8 seconds,
  1080×1920 split wallpaper with zoom and title. Three repetitions per mode,
  sequential execution, null muxer, no audio, warm CUDA cache. Separate real MP4
  validation checks 240 frames, 8 seconds, and 1080×1920 dimensions.
- The final `crop` + `pad_cuda` copy is intentional: writable overlay backing
  buffers can be aligned to 32 pixels. CUDA padding copies the visible area
  without scaling it, preventing NVENC from encoding allocation dimensions such
  as 1088 instead of 1080. This path requires `pad_cuda` in the bundled FFmpeg
  9.0.2; older builds without it fall back to the CPU filter path.
- Warm-cache median on the bundled 9.0.2 build: CPU filters + NVENC **2.462 s**;
  NVDEC + CUDA filters + NVENC **0.906 s** (about **2.72×**); CPU decode + CUDA
  filters + NVENC **1.757 s**. Static preparation took **0.163 s** once per source.
  These exclude import/probe time, audio and MP4 writes. First-use kernel
  compilation is additional and is not represented by the warm-cache speedup.

Reproduce from the repository root:

```text
node scripts/diagnose-export.mjs
node scripts/diagnose-export-hybrid.mjs
node scripts/diagnose-export-cuda.mjs
```

Exact commands, timings, decoder mappings and FFmpeg benchmark logs are saved
under `.tools/export-diagnosis/`. Synthetic results do not establish performance
for the user's original footage and settings. The CUDA comparison reruns its CPU
baseline with the same FFmpeg binary; do not mix results from different builds.
