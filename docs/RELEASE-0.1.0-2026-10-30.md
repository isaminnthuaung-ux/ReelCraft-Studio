# ReelCraft Studio 0.1.0 — 2026-10-30

Windows x64 portable production release. The executable embeds the production frontend and does not require a development server.

Changes:

- Numeric settings can be fully cleared with Backspace and replaced.
- Incomplete numeric edits remain local; persisted settings retain valid numeric values.
- Leaving a numeric field restores the last valid value or applies its bounds.
- Title and part-label numeric fields maintain separate editing state.

Validation:

- TypeScript and Vite production build passed.
- All 4 Playwright UI tests passed, including numeric input regression coverage.
- All 15 Rust native tests passed, including actual video export, Unicode overlays, resume, cancellation, and encoder fallback.

Distribution:

- Portable executable with FFmpeg/ffprobe 7.1.1 and third-party notices.
- App source ZIP includes lockfiles, build inputs, UI tests, native tests, and original toolbar PNG assets.
- SHA-256 checksums cover the portable and source ZIP files.
- No code-signing certificate or bundled WebView2 installer. WebView2 Runtime must be available on the recipient computer.
- Personal media, imported fonts, preferences, caches, development tools, and credentials are excluded from the package.
- Third-party corresponding-source requirements are described in the public share guide; the app source ZIP does not contain the full FFmpeg dependency source tree.
