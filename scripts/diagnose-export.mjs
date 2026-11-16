// Controlled synthetic diagnosis. Does not read or modify user media.
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { performance } from 'node:perf_hooks';

const root = resolve('.tools/export-diagnosis');
mkdirSync(root, { recursive: true });
const ffmpeg = resolve('src-tauri/target/release/bin/ffmpeg.exe');
const duration = 8;
const source = resolve(root, 'synthetic-av1.mkv');
const wall = resolve(root, 'wallpaper.png');
const title = resolve(root, 'title.png');
function run(args) {
  const start = performance.now();
  const result = spawnSync(ffmpeg, ['-hide_banner', '-nostdin', '-y', ...args], { windowsHide: true, encoding: 'utf8', maxBuffer: 8e6 });
  return { seconds: (performance.now() - start) / 1000, status: result.status, stderr: result.stderr, args };
}
function fixture(args) {
  const result = run(['-v', 'error', ...args]);
  if (result.status !== 0) throw Error(result.stderr);
}
console.log('Preparing 8s 1920x1080 30fps AV1 fixture...');
fixture(['-f', 'lavfi', '-i', `testsrc2=s=1920x1080:r=30:d=${duration}`, '-c:v', 'libaom-av1', '-cpu-used', '8', '-crf', '38', '-row-mt', '1', source]);
fixture(['-f', 'lavfi', '-i', 'testsrc2=s=1920x1080', '-frames:v', '1', wall]);
fixture(['-f', 'lavfi', '-i', 'color=white:s=700x150,format=rgba,colorchannelmixer=aa=0.7', '-frames:v', '1', title]);
const nvenc = preset => ['-c:v', 'h264_nvenc', '-preset', preset, '-tune', 'hq', '-rc', 'vbr', '-cq', '18', '-b:v', '0', '-multipass', 'disabled', '-rc-lookahead', '0', '-profile:v', 'high', '-pix_fmt', 'yuv420p'];
const blur = '[0:v:0]split=2[back][front];[back]scale=270:480:force_original_aspect_ratio=increase,crop=270:480,boxblur=luma_radius=14:luma_power=2:chroma_radius=7:chroma_power=1,scale=1080:1920,eq=brightness=-0.06:saturation=0.85[bg];[front]scale=1080:1920:force_original_aspect_ratio=decrease[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2,setsar=1[frame];[frame][1:v]overlay=190:1500:eof_action=repeat,format=yuv420p[out]';
// Exact split-wallpaper structure from framing_filter: scale=.85, zoom=1.5.
const wallpaper = '[0:v:0]split=2[front][clock];[clock]scale=2:2,scale=1080:1920,drawbox=color=black:t=fill,setsar=1[base0];[1:v:0]scale=1080:702:force_original_aspect_ratio=increase:force_divisible_by=2,scale=w=trunc(iw*1.5/2)*2:h=trunc(ih*1.5/2)*2,crop=1080:702,setsar=1[wall1];[base0][wall1]overlay=0:0:eof_action=repeat[base1];[2:v:0]scale=1080:702:force_original_aspect_ratio=increase:force_divisible_by=2,scale=w=trunc(iw*1.5/2)*2:h=trunc(ih*1.5/2)*2,crop=1080:702,setsar=1[wall2];[base1][wall2]overlay=0:1218:eof_action=repeat[base2];[front]scale=918:516,setsar=1[fg];[base2][fg]overlay=(W-w)/2:702:eof_action=repeat,setsar=1[frame];[frame][3:v]overlay=190:1500:eof_action=repeat,format=yuv420p[out]';
const input = ['-i', source];
const blurArgs = [...input, '-i', title, '-filter_complex', blur, '-map', '[out]'];
const wallArgs = [...input, '-i', wall, '-i', wall, '-i', title, '-filter_complex', wallpaper, '-map', '[out]'];
const cases = [
  ['AV1 CPU decode only', input],
  ['AV1 NVDEC decode only', ['-hwaccel', 'cuda', '-hwaccel_output_format', 'cuda', ...input]],
  ['AV1 CPU decode + NVENC p1 (1080p landscape)', [...input, ...nvenc('p1')]],
  ['Blur + title filters, no encoding', blurArgs],
  ['Blur + title + NVENC p1', [...blurArgs, ...nvenc('p1')]],
  ['Split wallpaper + zoom + title filters, no encoding', wallArgs],
  ['Split wallpaper + zoom + title + NVENC p1', [...wallArgs, ...nvenc('p1')]],
  ['Split wallpaper + zoom + title + NVENC p5', [...wallArgs, ...nvenc('p5')]],
];
const results = cases.map(([name]) => ({ name, runs: [] }));
for (let repeat = 0; repeat < 3; repeat++) {
  for (let i = 0; i < cases.length; i++) {
    const [name, args] = cases[i];
    const result = run(['-benchmark', ...args, '-an', '-t', String(duration), '-f', 'null', '-']);
    writeFileSync(resolve(root, `case-${i}-run-${repeat}.log`), result.stderr);
    results[i].runs.push({ seconds: result.seconds, status: result.status, args: result.args });
    console.log(`${repeat + 1}/3 ${name}: ${result.seconds.toFixed(3)}s, exit ${result.status}`);
  }
}
for (const result of results) {
  const times = result.runs.filter(r => r.status === 0).map(r => r.seconds).sort((a,b) => a-b);
  result.medianSeconds = times.length ? times[Math.floor(times.length / 2)] : null;
  result.speed = result.medianSeconds ? duration / result.medianSeconds : null;
}
writeFileSync(resolve(root, 'results.json'), JSON.stringify({ fixture: 'Synthetic AV1 1920x1080 30fps 8s; portrait output 1080x1920; no audio; null muxer eliminates output disk I/O; repeated sequentially three times; wall timing includes process startup; not actual user source/settings', results }, null, 2));
console.table(results.map(({ name, medianSeconds, speed }) => ({ name, medianSeconds: medianSeconds?.toFixed(3), speed: speed?.toFixed(2) })));
