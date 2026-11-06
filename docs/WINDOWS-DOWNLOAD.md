# Windows ready-to-use download / Windows အသင့်သုံး ZIP

## မြန်မာ

1. GitHub **Releases** မှ `ReelCraftStudio-<version>-Windows-x64.zip` ကို download လုပ်ပါ။
2. ZIP ကို **Extract All** လုပ်ပါ။ ZIP ထဲမှ တိုက်ရိုက် မဖွင့်ပါနှင့်။
3. ထွက်လာသည့် `ReelCraftStudio` folder ထဲက **ReelCraftStudio.exe** ကို ဖွင့်ပါ။

FFmpeg နှင့် ffprobe ကို `bin/` ထဲတွင် တစ်ပါတည်း ထည့်ထားသည်။ သီးခြား install လုပ်ရန်၊ PATH ပြင်ရန်၊ Node.js/Rust install လုပ်ရန် မလိုပါ။ `bin/` နှင့် `licenses/` folder များကို app နှင့်အတူ ထားပါ။

Windows 10/11 **x64** အတွက် ဖြစ်သည်။ WebView2 Runtime မရှိ၍ app မပွင့်ပါက `prerequisites/Install-WebView2.exe` ကို ဖွင့်ပြီး install လုပ်ပါ။ ဒီအဆင့်မှာ internet လိုအပ်သည်။ ထို့နောက် app ကို ပြန်ဖွင့်ပါ။ ZIP ထဲတွင် WebView2 runtime အပြည့်မပါဘဲ Microsoft ၏ installer အသေးသာ ပါသည်။

မြန်မာစာအတွက် Text studio မှ မြန်မာ Unicode font ကို ရွေးပါ။ ZIP သည် code-sign လုပ်မထားသော build ဖြစ်သည်။

## English

1. Download `ReelCraftStudio-<version>-Windows-x64.zip` from GitHub **Releases**.
2. Use **Extract All**. Do not run the app inside the ZIP.
3. Open **ReelCraftStudio.exe** inside the extracted `ReelCraftStudio` folder.

FFmpeg and ffprobe are included in `bin/`. No separate media-tool install, PATH change, Node.js, or Rust is needed. Keep `bin/` and `licenses/` beside the app.

Requires Windows 10/11 **x64** and Microsoft WebView2 Runtime. If WebView2 is missing, run `prerequisites/Install-WebView2.exe`, then open the app again. That installer needs an internet connection; the full offline WebView2 runtime is not included. These builds are unsigned.

## Maintainers: GitHub Actions

- A push to `main` or **Actions → Windows ready-to-use ZIP → Run workflow** builds a tested download artifact (retained for 30 days). Download the artifact, extract it, then extract the application ZIP inside.
- Push a version tag such as `v0.1.0` to publish the ZIP, source ZIP, and `SHA256SUMS.txt` under **Releases**. Release asset links are suitable for users who are not signed in to GitHub when the repository is public.
- The workflow uses Windows x64, a release Rust build, and static CRT linking. FFmpeg is pinned in `scripts/ffmpeg-windows.json` and verified with SHA-256 before use.
- The packaging test extracts the actual ZIP, checks required files and app identity, runs a synthetic H.264/AAC encode with bundled FFmpeg, probes it with bundled ffprobe, and verifies archive checksums. It does not automate GUI interaction.
- FFmpeg upstream notices, build information, version output, frontend/native dependency notices, and VidCoder asset notices are included. Keep these notices when redistributing. The application source ZIP does not replace the corresponding source of FFmpeg or its external libraries; consult the retained upstream build/source information for those components.