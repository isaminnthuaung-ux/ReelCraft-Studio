# ReelCraft Studio — Public Share Guide

Version: 0.1.0 · Release date: 2026-10-30 · Windows x64 portable

## စတင်အသုံးပြုရန်

1. `ReelCraftStudio-0.1.0-Windows-x64.zip` ကို download လုပ်ပါ။
2. ZIP ကို **Extract All** လုပ်ပါ။ ZIP အတွင်းမှ တိုက်ရိုက်မဖွင့်ပါနှင့်။
3. Extract လုပ်ထားသော folder ထဲမှ `ReelCraftStudio.exe` ကို ဖွင့်ပါ။
4. `bin` နှင့် `licenses` folder များကို EXE နှင့်အတူ ထားပါ။ EXE တစ်ခုတည်း မကူးပါနှင့်။

Windows 10/11 x64 နှင့် Microsoft Edge WebView2 Runtime လိုအပ်သည်။ Runtime မရှိပါက Microsoft official page မှ install လုပ်ပါ:
https://developer.microsoft.com/microsoft-edge/webview2/

Node.js, Python, Rust နှင့် FFmpeg ကို သီးခြား install လုပ်ရန် မလိုပါ။ FFmpeg နှင့် ffprobe ကို `bin` ထဲတွင် ထည့်ထားသည်။ NVIDIA encoder ရရှိလျှင် အသုံးပြုပြီး မရရှိလျှင် CPU encoder သုံးသည်။

## Video ဖြတ်ထုတ်ရန်

1. **Add videos** ဖြင့် video ထည့်ပါ၊ သို့မဟုတ် video/folder ကို window ထဲ ဆွဲထည့်ပါ။
2. **Length of each part** မှာ လိုချင်သော seconds သတ်မှတ်ပါ။ Backspace ဖြင့် ဂဏန်းအားလုံးဖျက်ပြီး ပြန်ရိုက်နိုင်သည်။ Input မှထွက်သောအခါ သတ်မှတ် range အတွင်း ပြန်ချိန်ပေးပြီး အလွတ်ဖြစ်လျှင် နောက်ဆုံး valid value ပြန်ပြသည်။
3. လိုအပ်ပါက **Smart Cut** နှင့် output frame ratio ကို ရွေးပါ။
4. **Text studio** မှ title / part label, font, size နှင့် position ကို ပြင်ပါ။ `{n}`, `{total}`, `{mm}` placeholders အသုံးပြုနိုင်သည်။ Font များသည် အသုံးပြုသူ၏ Windows တွင်ရှိသော fonts ကို သုံးသည်။
5. **Analyze cuts** ကို နှိပ်ပြီး မလိုသော parts ကို အမှန်ခြစ်ဖြုတ်ပါ။
6. Output folder ရွေးပြီး **Export** နှိပ်ပါ။ Video processing သည် computer ပေါ်မှာပင် လုပ်ဆောင်သည်။

Preview သည် frame preview ဖြစ်သည်။ Settings နှင့် imported fonts ကို Windows app-data folder တွင် သိမ်းသည်။ App ပြန်ဖွင့်လျှင် source videos ကို ပြန်ထည့်ရမည်။

## Public ကို share ပေးရန်

1. Release folder ထဲရှိ portable ZIP, `SHA256SUMS.txt`, ဒီ guide နှင့် `ReelCraftStudio-0.1.0-Source.zip` ကို အတူ upload လုပ်ပါ။ GitHub Releases သို့မဟုတ် file hosting service အသုံးပြုနိုင်သည်။ ယခု build လုပ်ခြင်းက online publish လုပ်ပြီးသား မဟုတ်ပါ။
2. Download link ကို **Anyone with the link** / public download အဖြစ် ရွေးပါ။ Account မဝင်ထားသော browser ဖြင့် link ကို စမ်းပါ။
3. ZIP တစ်ခုလုံးကို share ပါ။ `licenses` notices များ မဖယ်ရှားပါနှင့်။ Source archive တွင် app source နှင့် original toolbar PNG assets ပါသည်။ Third-party source/version information ကို `licenses/SOURCE-INFORMATION.txt` တွင် ကြည့်ပါ။ FFmpeg အပါအဝင် third-party binaries ကို ဆက်လက်ဖြန့်ဝေရာတွင် သက်ဆိုင်ရာ license နှင့် corresponding-source requirements ကိုလည်း လိုက်နာရမည်။ App source ZIP သည် FFmpeg နှင့် ၎င်း၏ libraries အားလုံး၏ corresponding source အစားထိုး မဟုတ်ပါ။
4. အောက်ပါ share စာသားကို အသုံးပြုနိုင်သည်:

> ReelCraft Studio 0.1.0 — Windows x64 portable video splitter. Video ကို parts များခွဲထုတ်နိုင်ပြီး Myanmar text/title နှင့် part labels ထည့်နိုင်သည်။ ZIP ကို Extract All လုပ်ပြီး ReelCraftStudio.exe ဖွင့်ပါ။ WebView2 Runtime လိုအပ်သည်။ Download: [သင့် download link]

## Download စစ်ဆေးရန်

PowerShell မှ ZIP ရှိသော folder တွင် အောက်ပါ command ကို run ပါ:

```powershell
Get-FileHash .\ReelCraftStudio-0.1.0-Windows-x64.zip -Algorithm SHA256
```

ရလာသော hash ကို `SHA256SUMS.txt` နှင့် နှိုင်းယှဉ်ပါ။ ဒီ release ကို digital code signing မလုပ်ထားပါ။ Windows က publisher မသိကြောင်း ပြနိုင်သည်။ Download source နှင့် hash ကို စစ်ပြီးမှ ဖွင့်ပါ။

## အဆင်မပြေလျှင်

- App မပွင့်လျှင် WebView2 Runtime ရှိမရှိ စစ်ပါ။
- FFmpeg မတွေ့လျှင် `bin/ffmpeg.exe`, `bin/ffprobe.exe` ကို EXE ဘေးတွင် ထားပါ။
- Myanmar စာမမှန်လျှင် **Text studio** မှ Myanmar glyphs ပါသော font ကို ရွေးပါ။
- Export error ဖြစ်လျှင် **Activity** log ကို ကြည့်ပြီး source file နှင့် output folder ရေးသားခွင့်ကို စစ်ပါ။
- Update လုပ်ရာတွင် လက်ရှိ export ပြီးဆုံးအောင်စောင့်၊ app ပိတ်ပြီး release အသစ်ကို သီးခြား folder တွင် extract လုပ်ပါ။
