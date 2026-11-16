// Run diagnose-export.mjs first. Reuse identical fixture and recorded commands.
import { readFileSync, writeFileSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
const root = resolve('.tools/export-diagnosis');
const baseline = JSON.parse(readFileSync(resolve(root, 'results.json'), 'utf8'));
const results = [];
for (const index of [4, 6]) {
  const original = baseline.results[index];
  const args = [...original.runs[0].args];
  args.splice(args.indexOf('-i'), 0, '-hwaccel', 'cuda', '-hwaccel_output_format', 'cuda');
  const graphIndex = args.indexOf('-filter_complex') + 1;
  args[graphIndex] = args[graphIndex].replace('[0:v:0]', '[0:v:0]hwdownload,format=nv12[decoded];[decoded]');
  const runs = [];
  for (let repeat = 0; repeat < 3; repeat++) {
    const start = performance.now();
    const result = spawnSync(resolve('src-tauri/target/release/bin/ffmpeg.exe'), ['-hide_banner', '-nostdin', '-y', ...args], { windowsHide: true, encoding: 'utf8', maxBuffer: 8e6 });
    const seconds = (performance.now() - start) / 1000;
    writeFileSync(resolve(root, `hybrid-${index}-${repeat}.log`), result.stderr);
    runs.push({ seconds, status: result.status });
    console.log(`${original.name} with NVDEC + download: ${seconds.toFixed(3)}s, exit ${result.status}`);
  }
  const times = runs.filter(r => r.status === 0).map(r => r.seconds).sort((a,b) => a-b);
  results.push({ name: original.name, args, runs, medianSeconds: times.length ? times[Math.floor(times.length / 2)] : null });
}
writeFileSync(resolve(root, 'hybrid-results.json'), JSON.stringify({ note: 'NVDEC -> hwdownload -> unchanged CPU filters -> NVENC; same fixture as baseline', results }, null, 2));
