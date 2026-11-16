# UI Tests

> [!NOTE]
> ဒီ folder ထဲမှာ ReelCraft Studio ရဲ့ UI နဲ့ workflow မှန်မမှန် အလိုအလျောက် စစ်ဆေးတဲ့ **Playwright test code** တွေ ပါဝင်ပါတယ်။
> App အသုံးပြုသူတွေအတွက် ဖွင့်သုံးဖို့ မလိုပါ။ Developer တွေ code ပြင်တဲ့အခါ ရှိပြီးသားလုပ်ဆောင်ချက်တွေ မပျက်ကြောင်း ပြန်စစ်ဖို့ GitHub မှာ သိမ်းထားတာပါ။

Tests cover UI layout, export progress, source import, clip selection, and export workflows.

Run from the project root after installing development dependencies:

```sh
npm run test:ui
```

The current test configuration uses Microsoft Edge. Generated screenshots and
reports are excluded from Git through `test-results/` and `playwright-report/`.
