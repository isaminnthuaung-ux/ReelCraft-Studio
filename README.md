# ReelCraft Studio

**မြန်မာ (မူလ)** | [English](README.en.md)

**ဗီဒီယိုရှည်များကို အပိုင်းတိုများအဖြစ် ခွဲထုတ်ပါ။** ReelCraft Studio သည် Rust၊ Tauri နှင့် React ကို အသုံးပြုထားသော Windows အတွက် ဗီဒီယိုခွဲထုတ်သည့် desktop app ဖြစ်သည်။ ဗီဒီယိုများကို မိမိစက်ပေါ်တွင်သာ လုပ်ဆောင်ပြီး မြန်မာစာခေါင်းစဉ်နှင့် အပိုင်းနံပါတ်များ ထည့်သွင်းနိုင်သည်။ မူလ recap application ၏ ဖိုင်များကို ပြောင်းလဲရန် သို့မဟုတ် အသုံးပြုရန် မလိုအပ်ပါ။

## အဓိကလုပ်ဆောင်ချက်များ

- သတ်မှတ်ထားသော စက္ကန့်အလိုက် ဗီဒီယိုကို အပိုင်းများ ခွဲထုတ်ခြင်း။
- **Smart Cut** ဖြင့် သတ်မှတ်ချိန်အနီးရှိ အသံငြိမ်သည့်နေရာကို ရှာဖွေဖြတ်တောက်ခြင်း။
- Original၊ 9:16၊ 16:9 နှင့် 1:1 အချိုးများ ရွေးချယ်ခြင်း။ အချိုးပြောင်းရာတွင် နောက်ခံဝါးခြင်း၊ အမည်းဘောင်ထည့်ခြင်း သို့မဟုတ် အလယ်ဗဟိုမှ ဖြတ်ယူခြင်း။
- မြန်မာ Unicode စာသား၊ ခေါင်းစဉ်နှင့် အပိုင်းနံပါတ်များ ထည့်သွင်းခြင်း။
- Windows font များကို ရှာဖွေရွေးချယ်ခြင်းနှင့် TTF၊ OTF၊ TTC font များ ထည့်သွင်းခြင်း။
- ရရှိနိုင်ပါက NVIDIA NVENC ကို အလိုအလျောက်အသုံးပြုခြင်း၊ မရရှိပါက CPU ဖြင့် ပြောင်းလဲထုတ်လုပ်ခြင်း။
- ပြီးစီးပြီးသော အပိုင်းများကို စစ်ဆေးပြီး ပြန်အသုံးပြုနိုင်သည့် export checkpoint စနစ်။

## App ကို ဖွင့်ရန်

### Final Build ရှိပြီးသားဆိုရင်

| OS | App ဖွင့်ရန် |
| --- | --- |
| Windows | `FinalBuild/Windows/ReelCraftStudio.exe` ကို နှစ်ချက်နှိပ်ပါ။ |
| Linux | Terminal တွင် `bash FinalBuild/Linux/START.sh` ကို run ပါ။ |
| macOS | `FinalBuild/macOS/START.command` ကို ဖွင့်ပါ။ သို့မဟုတ် `bash FinalBuild/macOS/START.sh` ကို run ပါ။ |

အသုံးပြုမည့်စက်တွင် **FFmpeg နှင့် ffprobe** ကို install လုပ်ထားပြီး PATH မှ ခေါ်ယူနိုင်ရမည်။ Windows တွင် **Microsoft Edge WebView2 Runtime** လည်း လိုအပ်သည်။ Linux တွင် WebKitGTK runtime library များ လိုအပ်သည်။ မြန်မာစာအတွက် Myanmar Unicode font ကို install လုပ်ပြီး app ထဲတွင် ရွေးပါ။

### Source မှ Final Build ထုတ်ရန်

မိမိ OS အတွက် script **တစ်ခုကိုသာ** project folder ထဲမှ run ပါ။ Script က dependency များကို `npm ci` ဖြင့် install လုပ်ပြီး final executable ထုတ်ပေးမည်။

| OS | Run ရမည့် command | ထွက်လာမည့် folder |
| --- | --- | --- |
| Windows | `.\BUILD-WINDOWS.bat` | `FinalBuild/Windows/` |
| Linux | `bash BUILD-LINUX.sh` | `FinalBuild/Linux/` |
| macOS | `bash BUILD-MAC.sh` | `FinalBuild/macOS/` |

**အရင် install လုပ်ထားရန်**

- OS အားလုံး: Node.js/npm၊ Rust/Cargo၊ FFmpeg နှင့် ffprobe။
- Windows: Windows SDK ပါသော Microsoft C++ Build Tools နှင့် Edge WebView2။
- Linux: မိမိ distribution အတွက် Tauri 2 build dependency များနှင့် system font များ။
- macOS: Xcode Command Line Tools (`xcode-select --install`)။ Homebrew ရှိပါက `brew install ffmpeg` ဖြင့် media tool များကို install လုပ်နိုင်သည်။

Ubuntu/Debian အတွက် build dependency ဥပမာ:

```bash
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev librsvg2-dev libayatana-appindicator3-dev ffmpeg fontconfig fonts-dejavu fonts-noto-core
```

အသေးစိတ်ကို [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) တွင် ကြည့်ပါ။ Windows အတွက် Windows ပေါ်တွင်၊ Linux အတွက် Linux ပေါ်တွင်၊ macOS အတွက် Mac ပေါ်တွင် build လုပ်ရမည်။ မိမိစက်၏ CPU architecture အတွက် ထုတ်ပေးသည်။ ဤ script များသည် portable executable ထုတ်ပေးပြီး installer၊ macOS `.app` သို့မဟုတ် DMG မထုတ်ပေးပါ။ Linux/macOS native build နှင့် runtime ကို ဒီ Windows စက်ပေါ်တွင် မစမ်းသပ်ရသေးပါ။

ယခင် `BUILD.bat`၊ `BUILD-UI.bat` နှင့် `START*.bat` များကို development အဟောင်းအတွက် ဆက်ထားသည်။ Final build အတွက် အပေါ်က script များကိုသာ သုံးပါ။

## အသုံးပြုပုံ

1. **Add videos**၊ folder ထည့်သည့်ခလုတ် သို့မဟုတ် drag-and-drop ဖြင့် ဗီဒီယိုများ ထည့်ပါ။
2. အပိုင်းတစ်ခုစီ၏ ကြာချိန်ကို စက္ကန့်ဖြင့် သတ်မှတ်ပါ။ လိုအပ်ပါက **Smart Cut** ကို ဖွင့်ပါ။
3. လိုအပ်သည့် frame အချိုးနှင့် နောက်ခံပုံစံကို ရွေးပါ။
4. **Text studio** တွင် ခေါင်းစဉ် သို့မဟုတ် အပိုင်းနံပါတ် layer ကို ရွေးပြီး စာသား၊ font၊ အရွယ်အစား၊ အရောင်၊ အလင်းဖောက်နှုန်း၊ နောက်ခံအကွက်နှင့် နေရာကို ပြင်ဆင်ပါ။ Preview ပေါ်ရှိ စာသားနေရာရွှေ့သည့် handle ကို ဆွဲရွှေ့နိုင်သည်။
5. **Analyze cuts** ကို နှိပ်ပြီး အပိုင်းများကို စစ်ဆေးပါ။ မထုတ်လိုသော အပိုင်းများကို အမှန်ခြစ်ဖြုတ်ပါ။
6. Output folder ရွေးပြီး **Export** ကို နှိပ်ပါ။ လုပ်ဆောင်မှုအသေးစိတ်ကို **Activity** တွင် ကြည့်နိုင်သည်။
7. လိုအပ်ပါက ရပ်တန့်နိုင်သည်။ တူညီသော source၊ settings နှင့် output folder ဖြင့် ပြန်စတင်လျှင် စစ်ဆေးအတည်ပြုပြီးသော အပိုင်းများကို ပြန်အသုံးပြုနိုင်သည်။

Preview သည် seek slider ပါသော frame preview ဖြစ်ပြီး အချိန်နှင့်တစ်ပြေးညီ ဗီဒီယိုဖွင့်ပြခြင်း မပါဝင်သေးပါ။ Settings နှင့် ထည့်သွင်းထားသော font များကို app ၏ Windows application-data folder တွင် သိမ်းဆည်းသည်။ Source စာရင်းနှင့် ကိုယ်တိုင်ရွေးထားသော အပိုင်းများကို session အတွင်းသာ ထိန်းသိမ်းထားသဖြင့် app ပြန်ဖွင့်ချိန်တွင် source များကို ပြန်ထည့်ပြီး analyze လုပ်ရန် လိုအပ်သည်။

## မြန်မာစာနှင့် Font များ

စာသား preview နှင့် export နှစ်ခုလုံးသည် Rust `cosmic-text` engine ကို အသုံးပြုသည်။ စာသားကို export resolution ဖြင့် ရေးဆွဲပြီး preview တွင် အရွယ်ချုံ့ပြသသည်။ မြန်မာစာနှင့် အင်္ဂလိပ်စာ ရောနှောအသုံးပြုမှုကို native integration test များတွင် ထည့်သွင်းစစ်ဆေးထားသည်။

- ထည့်သွင်းသော font များကို app ၏ သီးသန့် font library ထဲသို့ ကူးယူသည်။
- Font picker တွင် ရှာဖွေခြင်းနှင့် keyboard ဖြင့် ရွေးချယ်ခြင်းကို အသုံးပြုနိုင်သည်။ Font အမည်များကို Segoe UI ဖြင့် ပြသပြီး ဗီဒီယိုပေါ်ရှိစာသားကို ရွေးချယ်ထားသော font ဖြင့် ရေးဆွဲသည်။
- Font တစ်ခုစီတွင် ပါဝင်သော စာလုံးများ ကွာခြားနိုင်သည်။ အစားထိုး font အသုံးပြုမှုနှင့် ပျောက်နေသော စာလုံးများကို app က အသိပေးပြီး မဖြေရှင်းရသေးသော စာလုံးပျောက်မှုရှိပါက export ကို ရပ်တန့်ပေးသည်။
- အပိုင်းနံပါတ်တွင် `{n}`၊ `{total}` နှင့် `{mm}` ကို အသုံးပြုနိုင်သည်။ `{mm}` သုံးလျှင် စုစုပေါင်းအရေအတွက်ကိုလည်း မြန်မာဂဏန်းဖြင့် ပြသည်။
- ဤဗားရှင်းတွင် font family ရွေးချယ်နိုင်သည်။ သီးခြား weight/face၊ stroke/shadow၊ variable font axis နှင့် color emoji အတွက် အပြည့်အဝ ပံ့ပိုးမှု မပါဝင်သေးပါ။
- အင်္ဂလိပ်စကားလုံးများ ဖယ်ရှားခြင်း သို့မဟုတ် အသံဇာတ်ကြောင်းကို ဘာသာပြန်ခြင်း မလုပ်ဆောင်ပါ။

## Export လုပ်ဆောင်ပုံ

Source တစ်ခုစီအတွက် လမ်းကြောင်း hash ပါသော output folder ကို အသုံးပြုသည်။ ထုတ်လုပ်သော အပိုင်းဖိုင်အမည်တွင် settings နှင့် content ကို ခွဲခြားသိနိုင်သည့် fingerprint ပါရှိသည်။ Settings ပြောင်းလဲလျှင် ဖိုင်အသစ်များ ထုတ်လုပ်ပြီး ယခင် export များကို အလိုအလျောက် မဖျက်ပါ။ Checkpoint တွင် ပြီးစီးသော ဖိုင်များကို မှတ်တမ်းတင်ပြီး ပြန်အသုံးမပြုမီ စစ်ဆေးသည်။

NVIDIA NVENC ကို အမှန်တကယ် encode စမ်းသပ်ပြီး ရရှိလျှင် p5၊ VBR၊ CQ 21 ဖြင့် အသုံးပြုသည်။ Hardware စတင်ခြင်း သို့မဟုတ် encoding မအောင်မြင်လျှင် CPU H.264 (`libx264`၊ veryfast၊ CRF 20) သို့ အလိုအလျောက် ပြောင်းသည်။ အသံကို AAC ဖြင့် ထုတ်လုပ်သည်။ လုပ်ဆောင်မှုများသည် မိမိစက်ပေါ်တွင် အစဉ်လိုက် ဆောင်ရွက်ပြီး encoder၊ အမြန်နှုန်းနှင့် တိုးတက်မှုကို ပြသသည်။ Cancel လုပ်လျှင် လက်ရှိ analysis/render process ကို ရပ်တန့်သည်။

**Smart Cut** သည် အသံစွမ်းအင်ကိုသာ တိုင်းတာပြီး စကားသံကို အဓိပ္ပာယ်ဖော်ခြင်း မလုပ်ပါ။ ထို့ကြောင့် စာကြောင်းဆုံးချိန်တွင် အတိအကျ ဖြတ်နိုင်မည်ဟု အာမမခံနိုင်ပါ။ အသံအဆက်မပြတ် ကျယ်နေပါက သတ်မှတ်ချိန်အတိုင်း ဖြတ်ပြီး အသံတိတ်နေပါက တောင်းဆိုထားသော နယ်နိမိတ်ကို သုံးသည်။ နောက်ဆုံးကျန်သည့် အပိုင်းတိုကို ယခင်အပိုင်းနှင့် ပေါင်းစည်းနိုင်သည်။

## Development / Tests

Final build: see the OS-specific scripts above. Development: `npm run tauri -- dev`.

```text
cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1
npm run build
npm run test:ui
```

## လက်ရှိဗားရှင်း၏ အကန့်အသတ်များ

Windows တွင် စမ်းသပ်ထားပြီး Linux/macOS အတွက် စမ်းသပ်အတည်ပြုရန် ကျန်ရှိသည်။ Gemini၊ Whisper၊ TTS၊ social account ချိတ်ဆက်ခြင်းနှင့် online publishing လုပ်ဆောင်ချက်များ မပါဝင်ပါ။ ကိုယ်တိုင် cut boundary ပြင်ဆင်ခြင်း၊ ပိုမိုစုံလင်သော စာသားအလှဆင်မှု၊ project သိမ်းဆည်းခြင်းနှင့် encoder ကို ကိုယ်တိုင်ရွေးချယ်ခြင်းတို့သည် နောင်လုပ်ဆောင်ရန် ကျန်ရှိသည်။ NVIDIA/CPU အလိုအလျောက် ရွေးချယ်မှုကို အသုံးပြုနိုင်ပြီးဖြစ်သည်။ အစီအစဉ်အသေးစိတ်ကို [PLAN.md](PLAN.md) တွင် ဖတ်ရှုနိုင်သည်။

## လိုင်စင်နှင့် အသိအမှတ်ပြုချက်များ

Project ၏ MIT လိုင်စင်ကို [LICENSE](LICENSE) တွင် ကြည့်ပါ။ Third-party toolbar icon များ၏ အသိအမှတ်ပြုချက်နှင့် သက်ဆိုင်ရာလိုင်စင်ကို [NOTICE.md](public/icons/vidcoder/NOTICE.md) နှင့် [LICENSE.txt](public/icons/vidcoder/LICENSE.txt) တွင် ကြည့်ပါ။ FFmpeg နှင့် အခြား third-party ဖိုင်များကို ပြန်လည်ဖြန့်ဝေရာတွင် ၎င်းတို့နှင့် သက်ဆိုင်သော လိုင်စင်သတ်မှတ်ချက်များကို စစ်ဆေးလိုက်နာပါ။
