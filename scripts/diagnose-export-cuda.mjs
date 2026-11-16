// Reuse the baseline AV1 fixture. Compare identical split-wallpaper layouts.
// Run diagnose-export.mjs first. All artifacts stay in .tools/export-diagnosis.
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';
import { performance } from 'node:perf_hooks';
const root = resolve('.tools/export-diagnosis');
const ffmpeg = resolve('src-tauri/target/release/bin/ffmpeg.exe');
const cache = resolve(root, 'cuda-cache');
mkdirSync(cache, {recursive: true});
const original = JSON.parse(readFileSync(resolve(root, 'results.json'), 'utf8')).results[6];
function run(args, log) {
  const start = performance.now();
  const result = spawnSync(ffmpeg, ['-hide_banner', '-nostdin', '-y', ...args], { env: {...process.env, CUDA_CACHE_PATH:cache}, timeout:120000, windowsHide: true, encoding: 'utf8', maxBuffer: 8e6 });
  const seconds = (performance.now() - start) / 1000;
  writeFileSync(resolve(root, log), result.stderr);
  if (result.status !== 0) throw Error(result.stderr);
  return seconds;
}
const background = resolve(root, 'cuda-background.png');
const cpuGraph = original.runs[0].args[original.runs[0].args.indexOf('-filter_complex') + 1];
const preparationSeconds = run(['-f', 'lavfi', '-i', 'color=c=black:s=1920x1080:r=1', '-i', resolve(root, 'wallpaper.png'), '-i', resolve(root, 'wallpaper.png'), '-filter_complex', cpuGraph.split(';[frame][3:v]')[0], '-map', '[frame]', '-frames:v', '1', background], 'cuda-prepare.log');
const nvenc = ['-c:v','h264_nvenc','-preset','p1','-tune','hq','-rc','vbr','-cq','18','-b:v','0','-multipass','disabled','-rc-lookahead','0','-profile:v','high'];
const cases = [
  ['CPU filters + NVENC', original.runs[0].args],
  ...[true, false].map(decode => {
    const graph = `[0:v:0]${decode ? '' : 'format=yuv420p,hwupload_cuda,'}scale_cuda=format=yuv420p,split=2[clock][front];[clock]scale_cuda=1080:1920[base];[1:v]format=yuv420p,hwupload_cuda[wall];[base][wall]overlay_cuda=0:0:eof_action=repeat[bg];[front]scale_cuda=918:516[fg];[bg][fg]overlay_cuda=x=80:y=702:eof_action=repeat[frame];[2:v]format=yuva420p,hwupload_cuda[text];[frame][text]overlay_cuda=190:1500:eof_action=repeat,crop=1080:1920:0:0,pad_cuda=1080:1920:0:0,setsar=1[out]`;
    return [decode ? 'NVDEC + CUDA filters + NVENC' : 'CPU decode + CUDA filters + NVENC', ['-benchmark', '-init_hw_device', 'cuda=render:0', '-filter_hw_device', 'render', ...(decode ? ['-hwaccel','cuda','-hwaccel_output_format','cuda','-hwaccel_device','render'] : []), '-i', resolve(root,'synthetic-av1.mkv'), '-i', background, '-i', resolve(root,'title.png'), '-filter_complex', graph, '-map','[out]', ...nvenc, '-an','-t','8','-f','null','-']];
  }),
];
const results = [];
for (let index = 0; index < cases.length; index++) {
  const [name,args] = cases[index];
  const seconds = [];
  for (let repeat = 0; repeat < 3; repeat++) {
    seconds.push(run(args, `cuda-case-${index}-${repeat}.log`));
    console.log(`${name}: ${seconds.at(-1).toFixed(3)}s`);
  }
  seconds.sort((a,b) => a-b);
  results.push({ name, args, seconds, medianSeconds: seconds[1] });
}
// Also write a real container; the null-output benchmark alone cannot check MP4 timing.
const encoded = resolve(root,'cuda-validation.mp4');
const args = [...cases[1][1]];
args.splice(-3, 3, encoded);
run(args, 'cuda-validation.log');
const probe = spawnSync(resolve('src-tauri/target/release/bin/ffprobe.exe'), ['-v','error','-count_frames','-show_streams','-show_format','-of','json',encoded], { encoding:'utf8',windowsHide:true });
if(probe.status !== 0) throw Error(probe.stderr);
const metadata = JSON.parse(probe.stdout);
const video = metadata.streams.find(s => s.codec_type === 'video');
if(video.width !== 1080 || video.height !== 1920 || Number(video.nb_read_frames) !== 240 || Math.abs(Number(metadata.format.duration)-8)>0.05) throw Error('CUDA output geometry, duration or cadence changed');
writeFileSync(resolve(root,'cuda-results.json'),JSON.stringify({preparationSeconds, results, validation:metadata},null,2));
console.log(`Static preparation ${preparationSeconds.toFixed(3)}s; MP4 validation passed (240 frames, 8s, 1080x1920).`);
