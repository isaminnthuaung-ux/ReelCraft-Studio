// Small synthetic benchmark of the previous and optimized FFmpeg paths.
// Never uses, replaces, or deletes a user's video/export.
import { spawnSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { performance } from 'node:perf_hooks';

const root=resolve('.tools/export-benchmark');mkdirSync(root,{recursive:true});
const ffmpeg=resolve('src-tauri/target/release/bin/ffmpeg.exe');
const source=resolve(root,'source.mp4'), overlay=resolve(root,'transparent.png');
function run(args) {const result=spawnSync(ffmpeg,args,{windowsHide:true});if(result.status!==0)throw Error(String(result.stderr));}
run(['-y','-v','error','-f','lavfi','-i','testsrc2=s=1280x720:r=30:d=8','-c:v','libx264','-preset','ultrafast',source]);
run(['-y','-v','error','-f','lavfi','-i','color=s=1080x1920,format=rgba,colorchannelmixer=aa=0','-frames:v','1',overlay]);
const baseline='[0:v]split=2[back][front];[back]scale=1080:1920:force_original_aspect_ratio=increase,crop=1080:1920,boxblur=20:2[bg];[front]scale=1080:1920:force_original_aspect_ratio=decrease[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2[frame];[frame][1:v]overlay=0:0:eof_action=repeat,format=yuv420p[out]';
const optimized='[0:v]split=2[back][front];[back]scale=270:480:force_original_aspect_ratio=increase,crop=270:480,boxblur=luma_radius=14:luma_power=2:chroma_radius=7:chroma_power=1,scale=1080:1920,eq=brightness=-0.06:saturation=0.85[bg];[front]scale=1080:1920:force_original_aspect_ratio=decrease[fg];[bg][fg]overlay=(W-w)/2:(H-h)/2,setsar=1,format=yuv420p[out]';
const cpu=['-c:v','libx264','-preset','veryfast','-crf','20'];
const gpu=['-c:v','h264_nvenc','-preset','p5','-rc','vbr','-cq','21','-b:v','0'];
const results=[];
for(const [name,graph,codec,extra]of[['Previous Rust path',baseline,cpu,['-i',overlay]],['Optimized CPU path',optimized,cpu,[]],['Optimized NVIDIA path',optimized,gpu,[]]]){
  const start=performance.now();
  run(['-y','-v','error','-i',source,...extra,'-filter_complex',graph,'-map','[out]',...codec,'-an','-movflags','+faststart',resolve(root,`${results.length}.mp4`)]);
  const seconds=(performance.now()-start)/1000;
  results.push({name,seconds:Number(seconds.toFixed(3))});console.log(`${name}: ${seconds.toFixed(2)}s`);
}
writeFileSync(resolve(root,'results.json'),JSON.stringify({fixture:'8-second synthetic 1280x720 30fps source, 1080x1920 blurred portrait export, text disabled',note:'Single-run local measurement, not a general speed guarantee; encoder quality modes differ.',results},null,2));
