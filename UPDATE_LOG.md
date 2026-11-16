# ReelCraft Studio update log

## 1.1.0 · Build 3 · Beta — 2026-11-16

- **Export speed:** inspect each input's codec, pixel format, and frame rate; test NVIDIA decoding on the actual video and recommend Fastest for minimum render time.
- **GPU processing:** use CUDA scaling/compositing for static wallpaper, black, and crop layouts, with NVENC P1 encoding. Prepare static backgrounds once per source and preserve Unicode title rendering and transparency.
- **Fallbacks:** retry compatible GPU paths automatically and keep NVENC when GPU decoding or filters fail; use the ultrafast x264 preset as the final CPU fallback. Dynamic blur still uses CPU filters.
- **Export status:** show overall percentage, current part, processing mode, and full progress detail while exporting.
- **Output names:** save clips as `Part {n} - {source name}.mp4`; omit hashes from visible filenames and project folder names. Keep `hash.json` as background resume data and write all verified output filenames to `part-titles.txt`.
- **Resume safety:** reuse verified earlier exports, retain completed files when resuming, and prevent same-name source videos from silently sharing a project folder.
- **Workspace UI:** move encoding profiles beneath Source, simplify the empty source panel, and make output folder selection and export progress easier to read.
- **Build and release:** bump the app to 1.1.0 Build 3, document `FinalBuild/` as the single current build/package output location, and keep the old `Production/` directory ignored as legacy output.
- **Developer checks:** add UI workflow coverage for export progress and responsive layout, plus CUDA/NVENC diagnosis scripts and export performance notes.

## 1.0.1 Build 2 Beta — 2026-11-08

- Windows, Linux, and macOS GitHub Actions builds.
- Sidebar tagline, centered branding, and an About gear icon.
- CPU, GPU, RAM, and OS information in the header.
- macOS-style title bar with window controls on the left.
- Auto mode fits one wallpaper automatically.
- Split mode supports separate top and bottom wallpapers.
- Independent wallpaper zoom, positioning, and preview dragging.
- Adjustable video position and a highlighted Output Frame panel.

## 1.0.1 Beta — 2026-11-08

- Big Sur design with colorful icons and rounded logos.
- Four encoding profiles: Fastest, Fast, Balanced, and Quality.
- Dedicated Activity page and local project history.
- Custom wallpapers and 25%–200% video scaling.
- One UTF-8 title list for all exported parts.
