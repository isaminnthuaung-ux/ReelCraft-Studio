// Development-only test. Launch the app with WebView2 remote debugging enabled
// on localhost:9237 before running. Final builds do not enable remote debugging.
import { chromium } from '@playwright/test';
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { defaults } from '../src/types.ts';

const root = resolve('.tools/native-smoke');
mkdirSync(root, { recursive: true });
const input = resolve(root, 'sample.mp4');
const ffmpeg = resolve('src-tauri/target/release/bin/ffmpeg.exe');
const fixture = spawnSync(ffmpeg, ['-y', '-v', 'error', '-f', 'lavfi', '-i', 'testsrc2=s=640x360:r=30:d=12', '-f', 'lavfi', '-i', 'sine=frequency=440:duration=12', '-c:v', 'libx264', '-preset', 'ultrafast', '-c:a', 'aac', '-shortest', input], { windowsHide: true });
if (fixture.status !== 0) throw new Error(String(fixture.stderr));
const browser = await chromium.connectOverCDP('http://127.0.0.1:9237');
let savedSettings = defaults;
try {
  const page = browser.contexts()[0].pages()[0];
  if (!page) throw new Error('Native WebView window not found');
  await page.waitForSelector('.source-actions button:not([disabled])', { timeout: 30000 });
  savedSettings = await page.evaluate(async fallback => (await window.__TAURI_INTERNALS__.invoke('bootstrap')).settings || fallback, defaults);
  // Exercise the actual desktop drop event; Tauri's production IPC is immutable.
  await page.evaluate(input => window.__TAURI_INTERNALS__.invoke('plugin:event|emit', {
    event: 'tauri://drag-drop', payload: { paths: [input], position: { x: 300, y: 300 } }
  }), input);
  await page.getByText('sample.mp4', { exact: true }).waitFor();
  await page.waitForSelector('img.video-frame', { timeout: 30000 });
  await page.getByRole('button', { name: 'Text studio', exact: true }).click();
  await page.getByRole('textbox', { name: 'Overlay text' }).fill('မြန်မာစာ · Long to Shot');
  await page.waitForTimeout(1000);
  await page.waitForSelector('img.text-overlay', { timeout: 15000 });
  const overlay = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke('render_overlay', {
    settings: { seconds: 5, smart: false, extra: 0, ratio: 'Original', fill: 'black',
      text: { enabled: true, text: 'မြန်မာစာ John ၁၂၃', font: 'Myanmar Text', size: 42, x: .5, y: .5, color: '#ffffff', opacity: 1, boxed: true },
      label: { enabled: false, text: '', font: 'Segoe UI', size: 24, x: .5, y: .1, color: '#ffffff', opacity: 1, boxed: true } },
    width: 640, height: 360, n: 1, total: 3
  }));
  writeFileSync(resolve(root, 'unicode-overlay.png'), Buffer.from(overlay.image.split(',')[1], 'base64'));
  await page.getByRole('button', { name: 'Split & frame', exact: true }).click();
  await page.getByRole('button', { name: 'Original', exact: true }).click();
  await page.getByLabel('Length of each part').fill('5');
  if (await page.getByRole('switch', { name: 'Smart Cut', exact: true }).getAttribute('aria-checked') === 'true') await page.getByRole('switch', { name: 'Smart Cut', exact: true }).click();
  await page.getByRole('button', { name: 'Analyze cuts', exact: true }).click();
  await page.getByRole('checkbox', { name: 'Export part 3' }).waitFor({ timeout: 30000 });
  // OS file dialogs are covered by the mocked UI workflow. Here, export through
  // the real native command to verify packaged media tools and renderer access.
  await page.evaluate(async () => {
    window.__smokeProgress = [];
    const handler = window.__TAURI_INTERNALS__.transformCallback(event => window.__smokeProgress.push(event.payload));
    await window.__TAURI_INTERNALS__.invoke('plugin:event|listen', { event: 'studio-progress', target: { kind: 'Any' }, handler });
  });
  const files = await page.evaluate(async ({ input, root, defaults }) => {
    const invoke = window.__TAURI_INTERNALS__.invoke;
    const [source] = await invoke('add_sources', { paths: [input] });
    const settings = { ...defaults, seconds: 5, smart: false, text: { ...defaults.text, text: 'မြန်မာစာ · Long to Shot' } };
    const clips = await invoke('analyze_source', { source, settings });
    return invoke('export_clips', { sources: [{ source, clips }], settings, output: root });
  }, { input, root, defaults });
  if (files.length !== 3) throw new Error(`Expected 3 native exports, got ${files.length}`);
  const progress = await page.evaluate(() => window.__smokeProgress);
  if (!progress.some(event => event.phase === 'rendering')) throw new Error('Native export did not report in-part progress');
  await page.screenshot({ path: resolve(root, 'native-workspace.png'), fullPage: true });
  console.log('Native Tauri import, preview, Myanmar overlay, split planning and three-part export passed.');
  console.log('Overlay warnings:', overlay.warnings);
  console.log('Native render:', progress.find(event => event.message.includes('rendering'))?.message);
  console.log('In-part progress updates:', progress.filter(event => event.phase === 'rendering').length);
} finally {
  const page = browser.contexts()[0].pages()[0];
  if (page) await page.evaluate(settings => window.__TAURI_INTERNALS__.invoke('save_settings', { settings }), savedSettings);
  await browser.close();
}
