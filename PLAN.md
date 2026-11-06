# ReelCraft Studio

**Tagline:** Make long video to short

**Created:** 2026-10-26 (Asia/Bangkok)  
**Status:** Windows v0.1 implemented in this folder. Rust/Tauri executable built; native media and UI smoke tests passed. This document retains the wider design; see README.md for current features and limitations.
**Current platform scope:** Windows only. Android, macOS and Linux are outside the current build scope.

## Product objective

Build a standalone Rust-based Windows desktop version of the existing Splitter, adding selectable Unicode fonts and correct complex-script text rendering. Preserve the useful splitter workflow: video/folder input, duration-based splitting, optional pause-aware cuts, aspect-ratio settings, text overlays, part labels, preview, export and stop/resume.

Keep the requested product spelling **ReelCraft Studio**. Do not rename it to “Long to Short Studio” without a user request.

Facebook Reels, TikTok and daily posting volume are possible uses only. There is no daily target, required clip count, posting checklist or publishing feature. The scope is the Rust splitter with Unicode font support.

## Scope and independence

- Use only the splitter functionality as the starting point. Do not migrate Gemini translation, Whisper, Edge TTS, VoxCPM, narration generation or Smart Sync video retiming.
- Preserve source audio by default. Splitting does not rewrite or translate the video.
- Build the new application inside this folder. Treat all original application code as read-only reference: do not modify, refactor, fix or migrate it in place. The new Rust app must run independently of the original Python application.
- Target Windows only, with local files and local processing. Do not add mobile layouts, cross-platform media adapters or packaging for other operating systems in this rebuild. Network accounts are not needed.
- MVP splits consecutive time ranges with optional pause-aware boundaries and manual adjustments. It does not claim to identify compelling highlights or complete sentences from audio loudness alone.
- Automatic highlight selection, transcription, animated captions, daily boards, posting schedules and social publishing are outside this plan.

## Proposed technology

| Layer | Proposed choice | Purpose |
| --- | --- | --- |
| Desktop shell | Tauri 2 | Windows application shell with a Rust backend and webview interface |
| Interface | React + TypeScript | Queue, preview, text/font editor and splitter settings |
| Processing core | Rust | Planning, validation, process supervision, caching and job state |
| Media tools | FFmpeg + ffprobe | Decode, analyze, crop, overlay, encode and validate media |
| Job storage | Atomically written versioned JSON checkpoints | Local settings and split-job recovery without a database requirement |
| Project format | Versioned JSON export | Portable settings, clip plans and text layer metadata |
| Text rendering | Shared shaping/rasterization engine behind a Rust adapter | Unicode text with the same rendered overlay in preview and export |

Tauri explicitly supports a Rust backend paired with a web frontend. This is a Rust-based application; FFmpeg remains responsible for encoding. Rewriting video codecs in Rust is outside scope. Rust alone does not guarantee faster exports: codec, resolution, GPU support and concurrent workload determine throughput.

Source: [Tauri architecture](https://v2.tauri.app/concept/architecture/).

Choose and pin specific library/tool versions during the technical prototype, after confirming Windows packaging and the Unicode rendering requirements. No untested version pins are prescribed here.

## Main user workflow

1. Add one or several long videos, or import a folder.
2. Choose a reusable output preset and destination.
3. Choose the duration of each part and optional Smart Cut extension.
4. Analyze pause locations and generate a proposed clip list.
5. Preview clips, adjust boundaries and exclude unwanted ranges.
6. Apply text and part-number templates; choose fonts and verify glyph support.
7. Add approved clips to the export queue.
8. Export while monitoring progress; stop or resume when needed.
9. Review the exported parts and open their output folder.

The number of parts follows from source length and split settings. There is no requested daily or per-job output quota.

## Workspace design

Rebuild the standalone splitter with a modern Windows desktop interface. Preserve the original splitter capabilities, while redesigning their presentation. The original recap app and its layout remain unchanged.

### Modern UI direction

- A clean dark workspace with navy/charcoal surfaces, a restrained blue accent and clear visual hierarchy.
- Compact top bar with project/source information and primary export action.
- Source list on the left, a large preview in the center and a contextual inspector on the right.
- Clip list and export queue below the preview, with clear durations, cut reasons and job status.
- Text/font controls grouped into readable inspector sections with searchable font selection and live samples.
- Consistent spacing, typography, hover/focus states, icons with labels and accessible contrast.
- Resizable panels and sensible minimum window sizes; support Windows display scaling and keyboard navigation.
- Keep analysis and rendering asynchronous so the interface remains responsive.
- Useful empty, loading, error, stopped and completed states. No decorative dashboards or posting counters.
- Build the UI around actual splitter operations; do not introduce social-account, scheduling or publishing screens.

### Main workspace components

- **Source queue:** filename, duration, dimensions, audio presence and project assignment.
- **Preview/editor:** selected frame, aspect-ratio guides, draggable text layers and source-time controls.
- **Clip list:** clip number, source start/end, length, cut reason, approval and export status.
- **Inspector:** framing, font, text appearance, part numbering and export settings.
- **Activity panel:** analysis, rendering, cancellation and failure details with retry actions.

Use clear empty states and templates to reduce repetitive setup. All editable settings apply to a captured job snapshot when export starts; later UI changes must not silently mutate running jobs.

## Splitting and framing requirements

### Clip planning

- Fixed-duration mode with explicit policy for the final short remainder: keep, merge or exclude after review.
- Pause-aware mode bounded by a user-selected extension window.
- Manual start/end editing with frame-level preview and explicit time units.
- Preserve chronological order and avoid gaps/overlaps in consecutive splitting mode.
- Show when a boundary is exact, a detected pause, a low-energy fallback or a manual edit.
- Silence and low-energy regions are suggestions, not semantic sentence detection.

### Output presets

- Initial internal preset: vertical 9:16, 1080 x 1920, H.264 video and AAC audio when source audio exists.
- Also support Original, 16:9 and 1:1 for reuse beyond short-form posts.
- Fit with blurred background, fit with solid background and crop-to-fill.
- Manual crop position so faces or subjects can remain visible. Automatic face tracking is deferred.
- Adjustable visual safe-area guides for overlay placement; they are editable guides, not guaranteed platform UI boundaries.
- Clip duration, frame rate and bitrate are configurable. Verify current platform upload limits in official documentation before describing any preset as platform compliant.
- Use the primary video duration as the planning clock, with explicit handling of audio tails, absent audio and variable frame rate.

## Unicode text and font selection

### User-facing promise

Support installed or imported fonts that contain the required glyphs and are supported by the chosen renderer. A font being called “Unicode” does not mean it contains every language. Detect missing characters and offer explicit fallback rather than promising that every font works for every script.

### Font picker

- Enumerate system fonts and show searchable family, face, weight and style.
- Import project fonts: begin with TTF and OTF. Treat TTC face selection, variable fonts and color emoji as prototype/compatibility gates.
- Show a preview using the actual user text, including Myanmar syllables, punctuation, numbers and mixed Latin names.
- Save a font identity containing file content hash, face index and variation settings where supported; do not store only the displayed family name.
- Copy imported fonts into project assets when permitted. Offer an explicit portable-project packaging action for other fonts; do not automatically redistribute system font files.
- Warn when a font is unavailable on reopening a project and let the user choose a replacement. Do not silently change the design.

### Text editing

- Multiple text layers: title/hook, persistent label, part number and optional call-to-action.
- Unicode input, paste, undo/redo and multiline editing.
- Font family, style, size, fill, stroke, shadow, opacity and alignment.
- Line spacing, maximum width, wrapping and background-box padding, color, opacity and corner radius.
- Drag position, anchor point, snap alignment and safe-area guides.
- Explicit application scope: all clips, selected clips or one clip.
- Layer order, visibility and optional start/end time within a clip.
- Placeholders such as `{n}`, `{total}`, `{mm}`, `{source}` and `{date}` with a preview of expanded values.
- Support Myanmar number formatting consistently across current-part and total-part values.

### Rendering design

Use a single shaping and rasterization implementation for both editor previews and exported overlays. Do not draw final-export text with one engine and simulate it with ordinary browser/Tk text in preview.

Prototype a Rust-accessible text stack with script shaping, bidirectional layout, glyph fallback and rasterization. Evaluate a packaged Skia-based implementation against an alternative shaping/rasterization stack before selecting it. Test Myanmar first; do not assume that a working Latin demo proves complex-script support.

For static layers, render transparent PNG overlays at final output resolution, then composite them with FFmpeg. Display those same overlay assets scaled in the editor preview. For timed static layers, enable the overlay only in the configured interval. Text changes produce a new asset hash.

FFmpeg's `drawtext` supports font selection and shaping when the required libraries are built in, but available behavior depends on the distributed FFmpeg build. A shared overlay renderer is the preferred design for matching preview/export and for Myanmar text boxes. Direct `drawtext` can remain a prototype alternative if it passes the same test suite.

Source: [FFmpeg filter documentation](https://ffmpeg.org/ffmpeg-filters.html).

### Unicode acceptance cases

- Myanmar combining marks, stacked consonants and kinzi; no detached or clipped marks.
- Myanmar plus English names, ASCII and Myanmar digits in one layer.
- Arabic mixed with English to verify bidi behavior, and Thai to verify another combining script.
- Long text wraps without breaking grapheme clusters; text selection/editing respects grapheme boundaries.
- Imported font survives project restart; missing glyphs and missing fonts produce useful messages.
- Preview and exported frames match in wrapping, placement, stroke and box size.
- Emoji and variable/color fonts are supported only after documented compatibility tests; unsupported cases must be reported.

## Reliability changes required by the existing splitter review

| Existing issue | Requirement in the new app |
| --- | --- |
| Different videos with the same stem share an output folder | Use source/project IDs and collision-safe destinations; never overwrite another source implicitly |
| A pause midpoint can exceed Max extra | Enforce a hard boundary invariant after choosing every cut |
| Steady loud audio can be treated as a pause | Require meaningful energy contrast; identify silent/constant signals separately |
| Audio analysis ignores Stop | Supervise all analysis and render children with cancellation |
| Check Cuts can finish after the source selection changes | Snapshot inputs and attach project/source/revision IDs to results; discard stale results |
| Existing output may be reused based only on existence and state | Validate output media and matching fingerprints before marking complete |
| State file writes are not atomic | Atomically write versioned checkpoints and publish finished media |

Use the old Python planner as a behavioral reference, not as a production dependency or a line-for-line port. Port its useful concepts and fix the documented edge cases first.

## Rust processing architecture

```text
Desktop interface
       | typed commands and progress events
Rust application service
       +-- Project / source / clip repositories
       +-- Media probe and audio analysis
       +-- Boundary planner
       +-- Font and text-overlay service
       +-- Export queue and child-process supervisor
                         |
                    FFmpeg / ffprobe
                         |
                Validated clip artifacts
```

- Rust owns trusted job configuration and filesystem destinations. Frontend events do not directly compose executable shell commands.
- Launch FFmpeg using argument arrays. Handle filtergraph/path escaping separately and test Unicode paths, spaces, quotes and punctuation.
- Keep preview extraction and analysis off the UI thread.
- Start with one render worker and bounded analysis concurrency; add configurable parallel exports only after CPU/GPU/memory benchmarks.
- Detect usable hardware encoders with a real smoke test, then fall back to CPU encoding with an explicit log message.
- On Windows, hide helper process windows and supervise the child process tree. Stop waits for process exit and cleans only that job's temporary artifacts.
- Report monotonic per-clip progress and overall queue progress. Audio-only, no-audio and corrupt inputs must have clear outcomes.

## Job state, resume and output safety

Suggested states: `draft`, `analyzing`, `ready`, `queued`, `rendering`, `validating`, `completed`, `cancelled`, `failed`.

Persist source identity, source metadata, clip boundaries, settings revision, font/text assets, expected output, active attempt and failure details. On restart, reconcile jobs left rendering with actual output files; never assume that a killed process completed.

Fingerprint source content/identity, clip bounds, renderer revision, output settings and overlay assets. A text or font change must invalidate affected exports. Do not rely solely on filename and second-resolution modification time.

Write each export to a unique temporary file in its destination directory, probe it, then atomically publish it. Existing final files require matching identity or an explicit conflict decision. Never delete old outputs merely because a new plan has fewer clips; expose a separate user-invoked cleanup action.

Suggested project layout:

```text
project/
  project.json
  assets/fonts/
  assets/overlays/
  cache/analysis/
  cache/previews/
  exports/<source-id>/
  logs/
```

Keep application settings in its own application data location and split checkpoints with the project. Version the JSON schema for future compatibility. Do not share or overwrite the original recap app configuration, logs or job state.

## Implementation phases

1. **Technical gates:** verify Rust/Tauri packaging on Windows, FFmpeg process cancellation, Myanmar shaping and shared preview/export overlays. Produce small executable prototypes, not the full interface first.
2. **Splitter core:** implement probing, fixed and bounded pause-aware cuts, collision-safe outputs, queue state and stop/resume. Add unit and real-media integration tests.
3. **Modern Windows UI:** implement the redesigned source queue, preview, inspector, clip list and export queue. Connect manual boundaries, output presets and progress to the Rust core. Verify scaling, keyboard navigation and responsive background work. Operate without Python installed.
4. **Text studio:** font discovery/import, glyph coverage feedback, multiple layers, templates and matching export overlays.
5. **Release hardening:** fresh Windows-machine install, tool licensing/distribution review, Unicode paths, recovery after forced close and throughput measurements.

Each phase should leave a reviewable working result. Keep AI highlight selection and direct social publishing out of the critical path.

## Acceptance and performance measurement

- Export the expected number of parts for the selected source length and duration settings; test both single-file and folder batches.
- Verify consecutive mode covers exactly the intended source range, with no unintended overlaps/gaps.
- Every pause-aware cut remains inside its permitted extension window, including long silences at the end of that window.
- Test constant loud audio, silence, music, missing audio, variable frame rate, audio tails and corrupt media.
- Stop works during analysis and export; restarting resumes matching completed clips without re-encoding.
- Same-name sources from different folders cannot overwrite one another.
- Fonts and Unicode paths work after application restart; Myanmar preview/export comparison passes.
- Verify duration within the declared frame/encoder tolerance and inspect audio boundary continuity.
- Measure representative split/export workloads, queue responsiveness, peak memory and output size on a recorded reference PC. Set performance budgets from those measurements, rather than promising an arbitrary speedup from Rust.
- Test the installed app on Windows without Python, Rust or developer tools available. Document WebView2 and FFmpeg provisioning.
- Verify the modern workspace at common Windows display scales, including 100%, 125% and 150%; controls must remain reachable and text readable.

## Proposed repository structure when implementation begins

```text
LongToShotStudio/
  PLAN.md
  src/                     # TypeScript interface
  src-tauri/               # Tauri shell, commands and permissions
  crates/
    studio-core/           # planning, validation, project models
    studio-media/          # FFmpeg supervisor and media analysis
    studio-text/           # fonts, shaping and overlay rendering
  tests/fixtures/           # small, redistributable test videos/fonts
  docs/                     # architecture decisions and packaging notes
```

The first implementation uses `src/` for the interface and `src-tauri/src/` modules for planning, text, media and export. Separate workspace crates are deferred until the codebase warrants them. The original recap application remains unchanged.
