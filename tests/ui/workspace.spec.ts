import { expect, test } from '@playwright/test';

test('numeric inputs can be cleared with Backspace and retyped', async ({ page }) => {
  await page.goto('/');
  const duration = page.getByLabel('Length of each part');
  await duration.focus();
  await duration.press('End');
  for (let i = 0; i < 3; i++) await duration.press('Backspace');
  await expect(duration).toHaveValue('');
  await duration.pressSequentially('300');
  await expect(duration).toHaveValue('300');
  await duration.press('Tab');
  await expect(duration).toHaveValue('300');
  await duration.fill('');
  await duration.press('Tab');
  await expect(duration).toHaveValue('300');
  await duration.fill('1');
  await expect(duration).toHaveValue('1');
  await duration.press('Tab');
  await expect(duration).toHaveValue('5');
  await page.getByRole('button', { name: 'Text studio', exact: true }).click();
  for (const name of ['Font size', 'Text horizontal position', 'Text vertical position']) {
    const input = page.getByLabel(name, { exact: true });
    await input.focus();
    await input.press('ControlOrMeta+a');
    await input.press('Backspace');
    await expect(input).toHaveValue('');
    await input.pressSequentially('25');
    await input.press('Tab');
    await expect(input).toHaveValue('25');
  }
  await page.getByRole('button', { name: 'Part label', exact: true }).click();
  await expect(page.getByLabel('Font size', { exact: true })).toHaveValue('48');
});

test('browser preview clearly separates native features and exposes text controls', async ({ page }) => {
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  await expect(page.locator('.wordmark')).toContainText('REELCRAFT');
  await expect(page).toHaveTitle('ReelCraft Studio');
  await expect(page.getByRole('img', { name: 'ReelCraft Studio logo' })).toBeVisible();
  await expect.poll(()=>page.getByRole('img', { name: 'ReelCraft Studio logo' }).evaluate(image => (image as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
  await expect(page.locator('.page-heading')).toHaveCount(0);
  await expect(page.getByText('Interface preview ·')).toBeVisible();
  await expect(page.getByRole('button', { name: 'Add videos', exact: true })).toBeDisabled();
  await page.getByRole('button', { name: 'Text studio', exact: true }).click();
  await page.getByRole('textbox', { name: 'Overlay text' }).fill('မြန်မာစာ Unicode');
  await expect(page.getByRole('textbox', { name: 'Overlay text' })).toHaveValue('မြန်မာစာ Unicode');
  await page.getByRole('button', { name: 'Part label', exact: true }).click();
  await expect(page.getByRole('textbox', { name: 'Overlay text' })).toHaveValue('Part {n} / {total}');
  await page.getByRole('button', { name: 'Split & frame', exact: true }).click();
  await page.getByRole('button', { name: '9:16', exact: true }).click();
  await expect(page.getByLabel('Background & framing')).toBeEnabled();
  await expect(page.locator('.panel-title').first()).toHaveCSS('font-size', '13px');
  await expect(page.locator('.app')).toHaveCSS('font-family', /Segoe UI/);
  await expect(page.locator('svg.app-icon').first()).toBeVisible();
  await expect(page.getByRole('switch', { name: 'Smart Cut', exact: true })).toHaveCSS('width', '37px');
  await page.screenshot({ path: 'test-results/workspace.png', fullPage: true });
});

test('larger controls fit a smaller Windows workspace', async ({ page }) => {
  await page.setViewportSize({ width: 1120, height: 800 });
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  await expect(page.getByRole('button', { name: 'Export parts', exact: true })).toBeVisible();
  const layout = await page.evaluate(() => ({ width: document.documentElement.scrollWidth, viewport: innerWidth }));
  expect(layout.width).toBeLessThanOrEqual(layout.viewport);
  await page.getByRole('button', { name: 'Text studio', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Font family', exact: true })).toHaveCSS('font-size', '14px');
});

test('desktop bridge import, plan, deselect and export workflow', async ({ page }) => {
  await page.addInitScript(() => {
    const w = window as any;
    w.isTauri = true;
    let id=0, wallpaperImports=0;
    w.__TAURI_EVENT_PLUGIN_INTERNALS__={unregisterListener:()=>{}};
    w.__TAURI_INTERNALS__={
      metadata:{currentWindow:{label:'main'},currentWebview:{label:'main'}},
      transformCallback:()=>++id,
      unregisterCallback:()=>{},
      invoke:async (cmd:string,args:any) => {
        if(cmd.startsWith('plugin:window|')){w.__windowCommands=[...(w.__windowCommands||[]),cmd];return 1;}
        if(cmd==='bootstrap')return {fonts:['Myanmar Text','Segoe UI',...Array.from({length:500},(_,i)=>`Test Font ${String(i).padStart(4,'0')}`)],tools:[['ffmpeg',true],['ffprobe',true]],settings:null};
        if(cmd==='system_info')return {cpu:'AMD Ryzen 9 7950X',gpus:['NVIDIA GeForce RTX 4080'],ram_bytes:34359738368,os:'Windows 11 Pro 24H2 Build 26100'};
        if(cmd==='plugin:dialog|open')return args.options?.title==='Choose background wallpaper'?'C:/Images/wallpaper.jpg':args.options?.directory?'C:/Exports':['C:/Videos/story.mp4'];
        if(cmd==='import_wallpaper')return `C:/AppData/wallpapers/${['imported','top','bottom'][wallpaperImports++]}.png`;
        if(cmd==='add_sources')return [{path:'C:/Videos/story.mp4',name:'story.mp4',duration:400,width:1920,height:1080,has_audio:true}];
        if(cmd==='analyze_source')return [{n:1,start:0,end:180,cut:'pause',enabled:true},{n:2,start:180,end:360,cut:'pause',enabled:true},{n:3,start:360,end:400,cut:'end',enabled:true}];
        if(cmd==='render_overlay')return {image:'',warnings:[]};
        if(cmd==='preview_frame')return '';
        if(cmd==='export_clips'){w.__lastExport=args;return ['part1.mp4','part3.mp4'];}
        return 1;
      }
    };
  });
  const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));
  await page.goto('/', { waitUntil: 'domcontentloaded' });
  await expect(page.getByRole('radio', { name: 'Fastest', exact: true })).toBeChecked();
  await expect(page.getByLabel('System information')).toContainText('AMD Ryzen 9 7950X');
  await expect(page.getByLabel('System information')).toContainText('NVIDIA GeForce RTX 4080');
  await expect(page.getByLabel('System information')).toContainText('32.0 GiB');
  await expect(page.getByLabel('System information')).toContainText('Windows 11 Pro');
  await page.getByRole('button',{name:'Minimize window',exact:true}).click();
  await page.getByRole('button',{name:'Maximize or restore window',exact:true}).click();
  await page.getByRole('button',{name:'Close window',exact:true}).click();
  expect(await page.evaluate(()=>(window as any).__windowCommands)).toEqual(['plugin:window|minimize','plugin:window|toggle_maximize','plugin:window|close']);
  await page.getByRole('radio', { name: 'Quality', exact: true }).check();
  await page.getByRole('button',{name:'Add videos',exact:true}).click();
  await expect(page.getByText('story.mp4',{exact:true})).toBeVisible();
  await page.getByLabel('Video scale percentage',{exact:true}).fill('75');
  await page.getByLabel('Video scale percentage',{exact:true}).press('Tab');
  await expect(page.getByLabel('Video scale',{exact:true})).toHaveValue('75');
  await page.getByRole('button',{name:'9:16',exact:true}).click();
  await page.getByLabel('Background & framing').selectOption('wallpaper');
  await expect(page.getByText('Wallpaper ready',{exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Split',exact:true}).click();
  await page.getByRole('button',{name:'Choose top image',exact:true}).click();
  await expect(page.getByRole('button',{name:'Change top image',exact:true})).toBeVisible();
  await page.getByLabel('Top wallpaper zoom',{exact:true}).evaluate((input,value)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value')!.set!.call(input,value);input.dispatchEvent(new Event('input',{bubbles:true}));},'180');
  await page.getByRole('button',{name:'Bottom wallpaper',exact:true}).click();
  await page.getByRole('button',{name:'Choose bottom image',exact:true}).click();
  await expect(page.getByRole('button',{name:'Change bottom image',exact:true})).toBeVisible();
  await page.getByLabel('Bottom wallpaper vertical position',{exact:true}).evaluate((input,value)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value')!.set!.call(input,value);input.dispatchEvent(new Event('input',{bubbles:true}));},'20');
  await page.getByLabel('Video vertical position',{exact:true}).evaluate((input,value)=>{Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value')!.set!.call(input,value);input.dispatchEvent(new Event('input',{bubbles:true}));},'40');
  await page.getByRole('button',{name:'Auto',exact:true}).click();
  await expect(page.getByText('Wallpaper ready',{exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Split',exact:true}).click();
  await page.getByRole('button',{name:'Analyze cuts',exact:true}).click();
  const previewCard = await page.locator('.preview-panel').boundingBox();
  const partsCard = await page.locator('.clips-panel').boundingBox();
  expect(previewCard!.width).toBeGreaterThan(partsCard!.width);
  expect(Math.abs(previewCard!.y - partsCard!.y)).toBeLessThan(2);
  expect(partsCard!.x).toBeGreaterThanOrEqual(previewCard!.x + previewCard!.width);
  expect(Math.abs(previewCard!.height - partsCard!.height)).toBeLessThan(2);
  await expect(page.getByRole('checkbox',{name:'Export part 2'})).toBeChecked();
  await page.getByRole('checkbox',{name:'Export part 2'}).uncheck();
  await page.getByRole('button',{name:'Choose an output folder',exact:true}).click();
  await page.getByRole('button',{name:'Export 2 parts',exact:true}).click();
  await expect(page.getByText('2 parts exported',{exact:true})).toBeVisible();
  const result=await page.evaluate(()=>(window as any).__lastExport);
  expect(result.sources[0].clips[1].enabled).toBe(false);
  expect(result.settings.encoding_profile).toBe('quality');
  expect(result.settings.fill).toBe('wallpaper');
  expect(result.settings.wallpaper).toBe('C:/AppData/wallpapers/imported.png');
  expect(result.settings.scale).toBe(0.75);
  expect(result.settings.wallpaper_layout).toMatchObject({mode:'split',top:{path:'C:/AppData/wallpapers/top.png',zoom:1.8},bottom:{path:'C:/AppData/wallpapers/bottom.png',y:.2},video_y:.4});
  await expect.poll(()=>page.evaluate(()=>JSON.parse(localStorage.getItem('reelcraft.project-history.v1')||'[]').length)).toBe(1);
  await page.getByRole('button',{name:'Activity log',exact:true}).click();
  await expect(page.getByRole('heading',{name:'Activity log',exact:true})).toBeVisible();
  await expect(page.getByRole('log')).toContainText('Export complete');
  await expect(page.locator('.activity-drawer')).toHaveCount(0);
  await page.getByRole('button',{name:'Project history',exact:true}).click();
  await expect(page.getByRole('heading',{name:'story.mp4',exact:true})).toBeVisible();
  await expect(page.locator('.project-state')).toHaveText('2 exported');
  await page.reload();
  await page.getByRole('button',{name:'Project history',exact:true}).click();
  await expect(page.getByRole('heading',{name:'story.mp4',exact:true})).toBeVisible();
  await page.getByRole('button',{name:'Open project',exact:true}).click();
  await expect(page.getByRole('radio',{name:'Quality',exact:true})).toBeChecked();
  await expect(page.getByRole('button',{name:'Split',exact:true})).toHaveAttribute('aria-pressed','true');
  await expect(page.getByLabel('Top wallpaper zoom',{exact:true})).toHaveValue('180');
  await expect(page.getByLabel('Video scale percentage',{exact:true})).toHaveValue('75');
  await expect(page.getByRole('checkbox',{name:'Export part 2'})).not.toBeChecked();
  expect(errors).toEqual([]);
  await page.getByRole('button',{name:'Text studio',exact:true}).click();
  await page.getByRole('button', { name: 'Font family', exact: true }).click();
  // A full Windows font catalog must not mount hundreds of font-face previews.
  expect(await page.getByRole('option').count()).toBeLessThan(20);
  await expect(page.getByRole('option').first()).toHaveAttribute('aria-setsize', '502');
  await expect(page.getByRole('option').first()).toHaveCSS('font-family', /Segoe UI/);
  const search = page.getByRole('combobox', { name: 'Search Windows fonts' });
  await search.press('End');
  await expect(page.getByRole('option', { name: 'Test Font 0499', exact: true })).toBeVisible();
  await search.press('Enter');
  await expect(page.getByRole('button', { name: 'Font family', exact: true })).toContainText('Test Font 0499');
  await page.getByRole('button', { name: 'Font family', exact: true }).click();
  await expect(page.getByRole('option', { name: 'Test Font 0499', exact: true })).toHaveAttribute('aria-selected', 'true');
  await search.fill('font that is not installed');
  await expect(page.getByText('No matching fonts installed.')).toBeVisible();
  await page.getByRole('combobox', { name: 'Search Windows fonts' }).fill('Segoe');
  await page.getByRole('option', { name: 'Segoe UI', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Font family', exact: true })).toContainText('Segoe UI');
  await page.screenshot({ path:'test-results/desktop-workflow.png',fullPage:true });
});
