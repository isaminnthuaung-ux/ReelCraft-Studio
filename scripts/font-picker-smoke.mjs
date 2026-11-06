// Development-only: launch the app with WebView2 debugging on localhost:9239.
// This checks the real installed font catalog without changing the selected font.
import { chromium } from '@playwright/test';

let browser;
for (let attempt = 0; attempt < 25; attempt++) {
  try { browser = await chromium.connectOverCDP('http://127.0.0.1:9239'); break; }
  catch (error) { if (attempt === 24) throw error; await new Promise(resolve => setTimeout(resolve, 200)); }
}
try {
  const page = browser.contexts()[0].pages()[0];
  await page.getByRole('button', { name: 'Text studio', exact: true }).click();
  await page.waitForFunction(() => document.querySelector('.font-picker-trigger')?.disabled === false);
  await page.getByRole('button', { name: 'Font family', exact: true }).click();
  await page.getByRole('listbox', { name: 'Installed fonts' }).waitFor();
  const result = await page.getByRole('option').evaluateAll(rows => ({
    total: Number(rows[0]?.getAttribute('aria-setsize')),
    rendered: rows.length,
    listFont: getComputedStyle(rows[0]).fontFamily,
  }));
  if (result.total < 2 || result.rendered > 20 || !result.listFont.includes('Segoe UI')) throw Error(JSON.stringify(result));
  await page.getByRole('combobox', { name: 'Search Windows fonts' }).fill('Myanmar Text');
  await page.getByRole('option', { name: 'Myanmar Text', exact: true }).waitFor();
  await page.getByRole('combobox', { name: 'Search Windows fonts' }).press('Escape');
  console.log('Native font picker passed:', result);
} finally { await browser?.close(); }
