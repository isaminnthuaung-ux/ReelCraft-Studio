import { useCallback, useEffect, useRef, useState } from 'react';
import type { CSSProperties, ReactNode } from 'react';
import { open } from '@tauri-apps/plugin-dialog';
import { listen } from '@tauri-apps/api/event';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { ArrowLeft, ArrowRight, Check, ChevronDown, ChevronRight, LayoutGrid, LoaderCircle, Plus, Scissors, ShieldCheck, Sparkles, X } from 'lucide-react';
import { ArrowDownToLine, CheckCircle2, CircleHelp, Clapperboard, Columns3, Film, Folder, FolderOpen, ListVideo, Monitor, Settings2, Square, Type, Upload, Volume2 } from './icons';
import { call, desktop } from './bridge';
import { defaults, clock, dimensions } from './types';
import FontPicker from './FontPicker';
import NumberInput from './NumberInput';
import type { Clip, Progress, Settings, Source, SourceItem, TextLayer } from './types';

type Log = { time: string; message: string; error?: boolean };
const iconSize = 16;

function Section({ title, icon, children, trailing }: { title: string; icon?: ReactNode; children: ReactNode; trailing?: ReactNode }) {
  return <section className="inspector-section"><div className="section-heading"><span>{icon}{title}</span>{trailing}</div>{children}</section>;
}
function Toggle({ checked, onChange, label, disabled }: { checked: boolean; onChange: (b: boolean) => void; label: string; disabled?: boolean }) {
  return <button type="button" role="switch" aria-checked={checked} aria-label={label} disabled={disabled} className={`toggle ${checked ? 'on' : ''}`} onClick={() => onChange(!checked)}><span /></button>;
}

export default function App() {
  const [sources, setSources] = useState<SourceItem[]>([]);
  const [selected, setSelected] = useState(0);
  const [settings, setSettings] = useState<Settings>(defaults);
  const [fonts, setFonts] = useState<string[]>([]);
  const [tools, setTools] = useState<[string, boolean][]>([]);
  const [busy, setBusy] = useState('');
  const [ready, setReady] = useState(false);
  const [output, setOutput] = useState('');
  const [frame, setFrame] = useState('');
  const [overlay, setOverlay] = useState('');
  const [warnings, setWarnings] = useState<string[]>([]);
  const [time, setTime] = useState(0);
  const [frameLoading, setFrameLoading] = useState(false);
  const [progress, setProgress] = useState<Progress | null>(null);
  const [logs, setLogs] = useState<Log[]>([]);
  const [showLogs, setShowLogs] = useState(false);
  const [activeLayer, setActiveLayer] = useState<'text' | 'label'>('text');
  const [activeClip, setActiveClip] = useState(0);
  const [tab, setTab] = useState<'split' | 'text'>('split');
  const [dragging, setDragging] = useState(false);
  const [notice, setNotice] = useState('');
  const [exported, setExported] = useState(0);
  const addLog = useCallback((message: string, error = false) => {
    setLogs(l => [...l.slice(-199), { time: new Date().toLocaleTimeString([], { hour12: false }), message, error }]);
    if (error) setNotice(message);
  }, []);
  const previewSeq = useRef(0), overlaySeq = useRef(0);
  const item = sources[selected], source = item?.source;
  const clips = item?.clips || [];
  const selectedCount = sources.reduce((sum, s) => sum + s.clips.filter(c => c.enabled).length, 0);
  const totalDuration = sources.reduce((sum, s) => sum + s.source.duration, 0);
  const layer = settings[activeLayer];
  const update = (patch: Partial<Settings>) => {
    setSettings(s => ({ ...s, ...patch }));
    if (patch.seconds !== undefined || patch.extra !== undefined || patch.smart !== undefined) setSources(s => s.map(e => ({ ...e, clips: [] })));
    setExported(0);
  };
  const updateLayer = (patch: Partial<TextLayer>) => update({ [activeLayer]: { ...layer, ...patch } });

  useEffect(() => {
    if (!desktop) { setReady(true); return; }
    let disposed = false;
    call<{ fonts: string[]; tools: [string, boolean][]; settings: Settings | null }>('bootstrap').then(data => {
      if (disposed) return;
      setFonts(data.fonts); setTools(data.tools);
      if (data.settings) setSettings(data.settings);
      else setSettings(s => ({ ...s, text: { ...s.text, font: data.fonts.includes('Myanmar Text') ? 'Myanmar Text' : data.fonts[0] || '' }, label: { ...s.label, font: data.fonts.includes('Segoe UI') ? 'Segoe UI' : data.fonts[0] || '' } }));
      data.tools.filter(([, ok]) => !ok).forEach(([name]) => addLog(`${name} was not found. Install FFmpeg and add it to PATH, then restart.`, true));
      setReady(true); addLog('Workspace ready. All video processing stays on this computer.');
    }).catch(e => { if (!disposed) { addLog(String(e), true); setReady(true); } });
    return () => { disposed = true; };
  }, [addLog]);

  useEffect(() => {
    if (!desktop || !ready) return;
    const t = setTimeout(() => { call('save_settings', { settings }).catch(e => addLog(`Could not save settings: ${e}`, true)); }, 600);
    return () => clearTimeout(t);
  }, [settings, ready, addLog]);

  const addPaths = useCallback(async (paths: string[]) => {
    if (!paths.length || busy) return;
    setBusy('Reading videos'); setNotice('');
    try {
      const added = await call<Source[]>('add_sources', { paths });
      setSources(prev => [...prev, ...added.filter(a => !prev.some(p => p.source.path.toLowerCase() === a.path.toLowerCase())).map(source => ({ source, clips: [] }))]);
      addLog(`${added.length} video${added.length === 1 ? '' : 's'} read.`); setExported(0);
    } catch (e) { addLog(String(e), true); } finally { setBusy(''); }
  }, [busy, addLog]);

  useEffect(() => {
    if (!desktop) return;
    const unlisten = getCurrentWebview().onDragDropEvent(event => {
      if (event.payload.type === 'over') setDragging(true);
      else { setDragging(false); if (event.payload.type === 'drop') void addPaths(event.payload.paths); }
    });
    return () => { void unlisten.then(fn => fn()); };
  }, [addPaths]);

  useEffect(() => {
    if (!desktop) return;
    const unlisten = listen<Progress>('studio-progress', e => { setProgress(e.payload); if (e.payload.phase !== 'rendering') addLog(e.payload.message); });
    return () => { void unlisten.then(fn => fn()); };
  }, [addLog]);

  useEffect(() => { setTime(0); setFrame(''); setOverlay(''); setActiveClip(0); }, [source?.path]);
  useEffect(() => {
    const seq = ++previewSeq.current;
    if (!source || !desktop || !ready) return;
    const timeout = setTimeout(() => {
      setFrameLoading(true);
      call<string>('preview_frame', { source, settings, time }).then(img => { if (seq === previewSeq.current) setFrame(img); }).catch(e => { if (seq === previewSeq.current) addLog(`Preview: ${e}`, true); }).finally(() => { if (seq === previewSeq.current) setFrameLoading(false); });
    }, 240);
    return () => clearTimeout(timeout);
    // Text edits only rerender the overlay, never decode the source again.
  }, [source, settings.ratio, settings.fill, time, ready, addLog]);
  useEffect(() => {
    const seq = ++overlaySeq.current;
    if (!source || !desktop || !ready) return;
    const [width, height] = dimensions(source, settings.ratio);
    const timeout = setTimeout(() => {
      call<{ image: string; warnings: string[] }>('render_overlay', { settings, width, height, n: clips[activeClip]?.n || 1, total: clips.length || 1 }).then(data => {
        if (seq === overlaySeq.current) { setOverlay(data.image); setWarnings(data.warnings); }
      }).catch(e => { if (seq === overlaySeq.current) { setOverlay(''); setWarnings([String(e)]); } });
    }, 180);
    return () => clearTimeout(timeout);
  }, [source, settings, clips, activeClip, ready]);

  async function pickVideos(folder = false) {
    try { const result = await open(folder ? { directory: true, title: 'Add a folder of videos' } : { multiple: true, title: 'Add videos', filters: [{ name: 'Video', extensions: ['mp4', 'mkv', 'mov', 'webm', 'avi', 'm4v'] }] }); if (result) await addPaths(Array.isArray(result) ? result : [result]); }
    catch (e) { addLog(String(e), true); }
  }
  async function pickOutput() {
    try { const path = await open({ directory: true, title: 'Choose output folder' }); if (typeof path === 'string') setOutput(path); }
    catch (e) { addLog(String(e), true); }
  }
  async function analyze() {
    if (!sources.length || busy) return;
    setBusy('Analyzing cuts'); setNotice(''); setProgress(null); setExported(0);
    try {
      for (let i = 0; i < sources.length; i++) {
        addLog(`Analyzing ${sources[i].source.name}${settings.smart ? ' · finding pauses' : ' · exact cuts'}`);
        const plan = await call<Clip[]>('analyze_source', { source: sources[i].source, settings });
        setSources(prev => prev.map(p => p.source.path === sources[i].source.path ? { ...p, clips: plan } : p));
      }
      addLog('Cut plans ready. Review parts, then export.');
    } catch (e) { addLog(String(e), !String(e).startsWith('Stopped')); } finally { setBusy(''); }
  }
  async function exportAll() {
    if (busy || !selectedCount || !output) return;
    setBusy('Exporting'); setNotice(''); setProgress(null); setExported(0);
    try {
      const files = await call<string[]>('export_clips', { sources: sources.filter(s => s.clips.length), settings, output });
      setExported(files.length); addLog(`Export complete · ${files.length} parts saved.`);
    } catch (e) { addLog(String(e), !String(e).startsWith('Stopped')); } finally { setBusy(''); }
  }
  async function importFont() {
    try {
      const path = await open({ title: 'Import a Unicode font', filters: [{ name: 'Font', extensions: ['ttf', 'otf', 'ttc'] }] });
      if (typeof path === 'string') { const list = await call<string[]>('import_font', { path }); const added = list.filter(f => !fonts.includes(f)); setFonts(list); if (added[0]) updateLayer({ font: added[0] }); addLog('Font imported into this app’s font library.'); }
    } catch (e) { addLog(String(e), true); }
  }
  const mediaAvailable = tools.length === 2 && tools.every(([, ok]) => ok);
  const canWork = desktop && ready && !busy && mediaAvailable;
  const [pw, ph] = source ? dimensions(source, settings.ratio) : [16, 9];
  const moveText = (event: React.PointerEvent<HTMLButtonElement>) => {
    event.currentTarget.setPointerCapture(event.pointerId);
    const parent = event.currentTarget.parentElement!;
    const bounds = parent.getBoundingClientRect();
    const move = (e: PointerEvent) => updateLayer({ x: Math.min(.98, Math.max(.02, (e.clientX - bounds.left) / bounds.width)), y: Math.min(.98, Math.max(.02, (e.clientY - bounds.top) / bounds.height)) });
    const end = () => { event.currentTarget?.removeEventListener('pointermove', move); };
    const target = event.currentTarget; target.addEventListener('pointermove', move); target.addEventListener('pointerup', () => { target.removeEventListener('pointermove', move); end(); }, { once: true });
  };

  return <div className={`app ${dragging ? 'drop-active' : ''}`}>
    <aside className="rail">
      <div className="brand-mark" title="ReelCraft Studio"><img src="/branding/reelcraft.ico" alt="ReelCraft Studio logo"/></div>
      <div className="rail-divider" />
      <button className="rail-button active" aria-label="Splitter workspace" title="Splitter workspace"><Columns3 size={20} /></button>
      <button className={`rail-button ${showLogs ? 'active' : ''}`} aria-label="Activity log" title="Activity log" onClick={() => setShowLogs(!showLogs)}><ListVideo size={20} /></button>
      <div className="rail-bottom"><span className="local-dot" /><span className="vertical-label">LOCAL WORKSPACE</span><button className="rail-button" aria-label="About this app" title="About this app" onClick={() => setNotice('ReelCraft Studio · v0.1.0. Standalone Windows splitter. Rust + FFmpeg. Original recap app remains unchanged.')}><CircleHelp size={19} /></button></div>
    </aside>
    <div className="workspace">
      <header className="topbar"><div><div className="wordmark"><img className="header-logo" src="/branding/reelcraft.ico" alt="" aria-hidden="true"/>REELCRAFT <span>STUDIO</span><span className="version">BETA</span></div><p>Make long video to short</p></div><div className="header-right"><span className="private-badge"><ShieldCheck size={14} /> On-device processing</span><span className="header-separator"/><Monitor size={16}/><span>Windows</span></div></header>
      {!desktop && <div className="preview-banner"><Monitor size={15}/> Interface preview · Launch the Windows app to import and process local videos.</div>}
      {notice && <div role="alert" className="notice"><span>{notice}</span><button aria-label="Dismiss message" onClick={() => setNotice('')}><X size={16}/></button></div>}
      <main className="studio-grid">
        <section className="panel source-panel">
          <div className="panel-title"><span><Film size={16}/> Source videos</span><span className="count">{sources.length}</span></div>
          <div className="source-actions"><button disabled={!canWork} onClick={() => pickVideos()}><Plus size={15}/> Add videos</button><button className="icon-button" disabled={!canWork} title="Add folder" aria-label="Add folder" onClick={() => pickVideos(true)}><Folder size={16}/></button></div>
          <div className="source-list">{!sources.length ? <div className="source-empty"><div className="empty-mini"><Film size={25}/></div><strong>Your story starts here</strong><p>Add a video or a folder.<br/>We’ll take it from there.</p><span>MP4 · MKV · MOV · WEBM · AVI</span></div> : sources.map((s, i) => <button key={s.source.path} className={`source-item ${i === selected ? 'selected' : ''}`} disabled={!!busy} onClick={() => setSelected(i)}><div className="source-thumb"><Film size={19}/><span>{String(i+1).padStart(2,'0')}</span></div><div className="source-copy"><strong title={s.source.name}>{s.source.name}</strong><span>{clock(s.source.duration)} <b>·</b> {s.source.width} × {s.source.height}</span><small>{s.clips.length ? `${s.clips.length} parts planned` : 'Ready to analyze'}</small></div><span role="button" tabIndex={0} aria-label={`Remove ${s.source.name}`} className="remove-source" onClick={e => { e.stopPropagation(); setSources(p => p.filter((_, j) => j !== i)); setSelected(0); }} onKeyDown={e => { if(e.key==='Enter'){e.stopPropagation();setSources(p=>p.filter((_,j)=>j!==i));setSelected(0);} }}><X size={12}/></span></button>)}</div>
          <div className="source-summary"><div><span>Total footage</span><strong>{clock(totalDuration)}</strong></div><div><span>Planned parts</span><strong>{sources.reduce((s,e)=>s+e.clips.length,0)}</strong></div></div>
          <div className="workflow-tip"><Sparkles size={17}/><p><strong>A cleaner cut</strong>Smart Cut looks for a nearby pause. Review the boundaries before you export.</p></div>
        </section>
        <section className="center-column">
          <div className="panel preview-panel"><div className="panel-title"><span><Clapperboard size={16}/> Preview</span><div className="preview-meta"><span className="tiny-dot"/>{source ? `${pw} × ${ph}` : 'No video selected'}<span className="pill">{settings.ratio}</span></div></div>
            <div className="preview-stage">{source ? <div className="frame-container" style={{ '--aspect': `${pw} / ${ph}`, aspectRatio: `${pw}/${ph}` } as CSSProperties}>{frame ? <img className="video-frame" src={frame} alt="Source video frame"/> : <div className="frame-placeholder"><LoaderCircle className="spin" size={28}/><span>Preparing frame</span></div>}{overlay && <img className="text-overlay" src={overlay} alt="Rendered text overlay"/>}{tab==='text' && layer.enabled && <button className="text-handle" aria-label="Drag selected text layer" title="Drag to position text" style={{left:`${layer.x*100}%`,top:`${layer.y*100}%`}} disabled={!!busy} onPointerDown={moveText}><Type size={12}/><span>{activeLayer==='text'?'Title':'Part label'}</span></button>}{frameLoading && frame && <span className="loading-chip"><LoaderCircle className="spin" size={12}/> Updating</span>}</div> : <div className="preview-empty"><div className="empty-frame-icon"><div/><Scissors size={32}/><div/></div><h2>A little less long.<br/><span>A lot more yours.</span></h2><p>Your video preview will appear here.<br/>Add footage to start shaping your next clip.</p><button className="subtle-button" disabled={!canWork} onClick={() => pickVideos()}><Plus size={15}/> Choose a video</button></div>}</div>
            <div className="preview-controls"><button className="icon-button" disabled={!source || !!busy} title="Back 5 seconds" aria-label="Back 5 seconds" onClick={()=>setTime(Math.max(0,time-5))}><ArrowLeft size={16}/></button><span className="timecode">{clock(time)}</span><input aria-label="Preview time" type="range" min="0" max={source?.duration||1} step="0.1" value={time} disabled={!source || !!busy} onChange={e=>setTime(+e.target.value)}/><span className="timecode muted">{clock(source?.duration||0)}</span><button className="icon-button" disabled={!source || !!busy} title="Forward 5 seconds" aria-label="Forward 5 seconds" onClick={()=>setTime(Math.min(source?.duration||0,time+5))}><ArrowRight size={16}/></button><span className="control-divider"/><span className="frame-label">FRAME PREVIEW</span></div>
          </div>
          <div className="panel clips-panel"><div className="panel-title"><span><LayoutGrid size={16}/> Your parts <span className="count">{clips.length}</span></span></div>
            {!clips.length ? <div className="clips-empty"><div className="clip-outlines"><span/><span/><span/></div><div><strong>One video. Many possibilities.</strong><p>Set a part duration, then analyze your cuts.</p></div></div> : <div className="clip-table"><div className="clip-row table-heading"><span/><span>PART</span><span>SOURCE RANGE</span><span>LENGTH</span><span>CUT</span></div>{clips.map((c,i)=><div key={c.n} className={`clip-row ${activeClip===i?'current':''}`}><input aria-label={`Export part ${c.n}`} type="checkbox" disabled={!!busy} checked={c.enabled} onChange={e=>{setSources(prev=>prev.map((s,j)=>j===selected?{...s,clips:s.clips.map((c,k)=>k===i?{...c,enabled:e.target.checked}:c)}:s));setExported(0);}}/><button className="part-select" onClick={()=>{setActiveClip(i);setTime(c.start);}}><span className="part-number">{String(c.n).padStart(2,'0')}</span><span>Part {c.n}</span></button><span className="range-label">{clock(c.start)} <ChevronRight size={11}/> {clock(c.end)}</span><span className="clip-duration"><span>Length</span>{clock(c.end-c.start)}</span><span className={`cut-badge ${c.cut==='pause'?'good':''}`}>{c.cut}</span></div>)}</div>}
            <div className="parts-actions"><button className="text-button" disabled={!canWork||!sources.length} onClick={analyze}>{busy==='Analyzing cuts'?<LoaderCircle className="spin" size={14}/>:<Scissors size={14}/>} Analyze cuts</button></div>
          </div>
        </section>
        <aside className="panel inspector"><div className="inspector-tabs"><button className={tab==='split'?'selected':''} onClick={()=>setTab('split')}><Settings2 size={15}/> Split & frame</button><button className={tab==='text'?'selected':''} onClick={()=>setTab('text')}><Type size={16}/> Text studio</button></div><div className="inspector-body"><fieldset disabled={!!busy}>
          {tab==='split'?<>
            <Section title="PART DURATION" icon={<Scissors size={14}/>}><label className="field-label" htmlFor="part-duration">Length of each part</label><div className="number-unit"><NumberInput id="part-duration" min={5} max={36000} value={settings.seconds} onChange={value=>update({seconds:value})}/><span>seconds</span></div><div className="quick-options">{[30,60,180,300].map(s=><button className={settings.seconds===s?'chosen':''} key={s} onClick={()=>update({seconds:s})}>{s<60?`${s}s`:`${s/60} min`}</button>)}</div><div className="toggle-row"><div><strong>Smart Cut</strong><p>Find a nearby pause in the audio</p></div><Toggle label="Smart Cut" checked={settings.smart} onChange={smart=>update({smart})}/></div>{settings.smart&&<label className="inline-setting">Wait up to <NumberInput aria-label="Maximum pause extension" min={0} max={120} value={settings.extra} onChange={value=>update({extra:value})}/><span>extra seconds</span></label>}</Section>
            <Section title="OUTPUT FRAME" icon={<Monitor size={14}/>}><div className="ratio-grid">{['Original','9:16','16:9','1:1'].map(r=><button key={r} className={settings.ratio===r?'chosen':''} onClick={()=>update({ratio:r})}><span className={`ratio-icon ratio-${r.replace(':','-')}`}/><span>{r}</span>{settings.ratio===r&&<Check size={11}/>}</button>)}</div><label className="field-label" htmlFor="frame-fill">Background & framing</label><div className="select-wrap"><select id="frame-fill" disabled={settings.ratio==='Original'} value={settings.fill} onChange={e=>update({fill:e.target.value})}><option value="blur">Blurred background</option><option value="black">Black bars</option><option value="crop">Crop to fill</option></select><ChevronDown size={14}/></div><p className="field-help">{settings.ratio==='Original'?'Keep the source video’s original dimensions.':'Fit your footage to the selected frame.'}</p></Section>
            <Section title="TEXT OVERLAYS" icon={<Type size={14}/>}><button className="text-layer-shortcut" onClick={()=>{setTab('text');setActiveLayer('text');}}><span className="layer-icon"><Type size={18}/></span><span><strong>Make it your own</strong><small>Fonts, titles & part numbers</small></span><ChevronRight size={16}/></button></Section>
            <Section title="AUDIO" icon={<Volume2 size={14}/>}><div className="audio-info"><span className="tiny-dot"/><span>{source ? source.has_audio?'Source audio preserved':'No source audio track':'Original audio preserved'}</span></div><p className="field-help">No narration or translation. Just your video.</p></Section>
          </>:<>
            <Section title="TEXT LAYER" icon={<Type size={14}/>} trailing={<Toggle label="Enable selected text layer" checked={layer.enabled} onChange={enabled=>updateLayer({enabled})}/>}><div className="layer-tabs"><button className={activeLayer==='text'?'chosen':''} onClick={()=>setActiveLayer('text')}>Title / text</button><button className={activeLayer==='label'?'chosen':''} onClick={()=>setActiveLayer('label')}>Part label</button></div><textarea aria-label="Overlay text" rows={3} placeholder={activeLayer==='text'?'Write your title…':'Part {n} / {total}'} value={layer.text} onChange={e=>updateLayer({text:e.target.value})}/><p className="field-help">{activeLayer==='label'?'Use {n}, {total} or {mm} for Myanmar numbers.':'Unicode text, including Myanmar. Line breaks supported.'}</p></Section>
            <Section title="FONT & APPEARANCE"><div className="font-picker-label"><span className="field-label">Windows fonts</span><span>{fonts.length} available</span></div><FontPicker fonts={fonts} value={layer.font} onChange={font=>updateLayer({font})}/><p className="field-help">Uses fonts installed in Windows. Import is optional.</p><button className="import-font" disabled={!desktop} onClick={importFont}><Upload size={14}/> Import TTF / OTF / TTC</button><div className="two-fields"><label>Size <div className="number-unit"><NumberInput key={activeLayer + "-size"} aria-label="Font size" min={12} max={200} value={layer.size} onChange={value=>updateLayer({size:value})}/><span>px</span></div></label><label>Color <div className="color-field"><input aria-label="Text color" type="color" value={layer.color} onChange={e=>updateLayer({color:e.target.value})}/><span>{layer.color.toUpperCase()}</span></div></label></div><label className="range-setting"><span>Opacity <strong>{Math.round(layer.opacity*100)}%</strong></span><input aria-label="Text opacity" type="range" min={0} max={1} step={.01} value={layer.opacity} onChange={e=>updateLayer({opacity:+e.target.value})}/></label><div className="toggle-row"><strong>Background box</strong><Toggle label="Text background box" checked={layer.boxed} onChange={boxed=>updateLayer({boxed})}/></div></Section>
            <Section title="POSITION"><p className="field-help">Drag the text handle in the preview, or adjust below.</p><div className="two-fields"><label>Horizontal <NumberInput key={activeLayer + "-x"} aria-label="Text horizontal position" min={2} max={98} value={Math.round(layer.x*100)} onChange={value=>updateLayer({x:value/100})}/></label><label>Vertical <NumberInput key={activeLayer + "-y"} aria-label="Text vertical position" min={2} max={98} value={Math.round(layer.y*100)} onChange={value=>updateLayer({y:value/100})}/></label></div><div className="position-presets">{[.12,.5,.85].map((y,i)=><button key={y} onClick={()=>updateLayer({x:.5,y})}>{['Top','Center','Bottom'][i]}</button>)}</div></Section>
            <div className="font-note"><ShieldCheck size={14}/><p>Preview and export use the same Rust text renderer. Missing glyphs use available fallback fonts.</p></div>{warnings.map(w=><div className="font-warning" key={w}>{w}</div>)}
          </>}
        </fieldset></div></aside>
      </main>
      <section className="export-bar"><div className="destination"><span className="destination-icon"><FolderOpen size={20}/></span><div><span className="eyebrow">SAVE YOUR PARTS</span><button className="destination-picker" disabled={!desktop||!!busy} onClick={pickOutput} title={output||'Choose output folder'}>{output||'Choose an output folder'}<ChevronDown size={13}/></button></div><button className="icon-button" aria-label="Open output folder" title="Open output folder" disabled={!output} onClick={()=>call('open_output',{path:output}).catch(e=>addLog(String(e),true))}><FolderOpen size={16}/></button></div><div className="export-status">{busy?<><LoaderCircle className="spin" size={16}/><div><strong>{busy}</strong><span>{progress?.message||'Working locally…'}</span></div></>:exported?<><CheckCircle2 size={18}/><div><strong>{exported} parts exported</strong><span>Your files are ready</span></div></>:<><span className="status-dot"/><div><strong>{selectedCount?`${selectedCount} parts selected`:'Ready when you are'}</strong><span>{selectedCount?'H.264 · AAC · MP4':'Add footage and analyze your cuts'}</span></div></>}</div>{busy?<button className="stop-button" onClick={()=>call('cancel_job').catch(e=>addLog(String(e),true))}><Square size={13}/> Stop safely</button>:<button className="primary-button" disabled={!canWork||!selectedCount||!output} onClick={exportAll}><ArrowDownToLine size={17}/> Export {selectedCount?`${selectedCount} parts`:'parts'}<ArrowRight size={16}/></button>}</section>
      {busy&&<div className="export-progress"><span style={{width:`${Math.max(3,(progress?.progress||0)*100)}%`}}/></div>}
      <footer><span><span className="tiny-dot"/> {ready?'Local workspace':'Starting workspace…'} <span className="footer-dot">·</span> {mediaAvailable?'FFmpeg ready':desktop?'Checking media tools':'Desktop app required'}</span><button onClick={()=>setShowLogs(!showLogs)}><ListVideo size={13}/> Activity {logs.length>0&&<span>{logs.length}</span>}<ChevronDown size={12} className={showLogs?'flipped':''}/></button></footer>
      {showLogs&&<div className="activity-drawer"><div className="panel-title"><span><ListVideo size={15}/> Activity log</span><button className="icon-button" aria-label="Close activity" onClick={()=>setShowLogs(false)}><X size={16}/></button></div><div className="log-lines">{logs.length?logs.map((l,i)=><div className={l.error?'error':''} key={i}><time>{l.time}</time><span>{l.message}</span></div>):<p>No activity yet.</p>}</div></div>}
    </div>{dragging&&<div className="drop-overlay"><Upload size={40}/><h2>Drop your videos here</h2><p>Files and folders are welcome.</p></div>}
  </div>;
}
