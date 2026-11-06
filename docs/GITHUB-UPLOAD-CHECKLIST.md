# GitHub upload checklist

Reviewed 2026-11-06. This checklist covers the project files and upload scope, not a full application or security audit.

## Repository boundary

The initial review found a parent Git repository at `E:\AIPROJECT`. A separate repository has now been initialized in this project directory and linked to `https://github.com/Minnthuaung/ReelCraftStudio`, preserving its existing main history. Unrelated parent projects are outside this repository.

Run Git commands from this project directory when updating this repository.

## Upload as source

| Files or folders | Purpose |
| --- | --- |
| `src/` | React frontend |
| `src-tauri/src/`, `src-tauri/capabilities/`, `src-tauri/icons/` | Native backend, permissions, application icons |
| `src-tauri/Cargo.toml`, `Cargo.lock`, `build.rs`, `tauri.conf.json` | Native dependencies and build configuration |
| `public/`, `branding/`, `ReelCraft Studio Neon Logo.png` | Application assets and branding |
| `scripts/`, `tests/` | Build support, packaging, and test source |
| `docs/`, `README.md` | Documentation; manually review screenshots for personal paths or media before publishing |
| `package.json`, `package-lock.json`, `bun.lock` | JavaScript dependencies and lockfiles; npm is the documented workflow |
| `index.html`, `tsconfig.json`, `vite.config.ts`, `playwright.config.ts` | Frontend and test configuration |
| Root `BUILD*`, `DEV*`, `START*` scripts | Windows build and launch helpers |
| `.gitignore` | Upload exclusions |
| `PLAN.md`, `VALIDATION.md` | Optional development notes; read before making public |

Keep `public/icons/vidcoder/NOTICE.md` and `LICENSE.txt` with the toolbar assets. The existing GitHub repository MIT LICENSE has been preserved at the project root. Third-party assets retain their own license notices.

## Keep out of source control

- `node_modules/`: installed dependencies.
- `src-tauri/target/`, `dist/`, `src-tauri/gen/`: compiled output and generated schemas.
- `.tools/`, `.npm-cache/`: local toolchains, synthetic test media, caches, and benchmark output.
- `test-results/`, `playwright-report/`, `*.log`: local test results and logs.
- `Production/`: approximately 244 MB of release binaries, archives, and duplicated source at review time. Share selected release archives and checksums as release attachments separately.
- `.env*` except example templates, `.npmrc`, signing keys, credential directories, and local editor/agent state.
- Personal videos, exports, imported fonts, and application settings, if copied here later. The current source tree does not need these to be uploaded.

The ignore file has been updated for these generated outputs and common credentials. Ignore rules do not protect files already committed, or a manual browser upload that bypasses Git.

## Review findings

- A pattern scan of upload-candidate text files found no recognizable private keys or common access-token formats. This is not a guarantee that every possible secret is absent.
- The packaging helper `scripts/package-public.ps1` refers to a missing `Copy/` folder and a fixed release date. Do not assume it can recreate the existing release unchanged on a fresh checkout.
- Build helpers support this machine's local toolchain setup. A clean-machine build was not performed for this file review.
- Screenshots and any existing release binaries were not inspected for private content or provenance.

## Before the first push

In the standalone repository, inspect `git status --short` and `git diff --cached --stat` after staging. Confirm that only this project's source and approved assets appear, and that `Production/`, dependencies, build output, credentials, and unrelated projects are absent.
