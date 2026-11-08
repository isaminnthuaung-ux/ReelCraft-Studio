import { CheckCircle2, FolderOpen } from './icons';
import type { WallpaperLayout } from './types';

export default function WallpaperControls({ layout, side, available, desktop, busy, onChange, onSide, onPick }: {
  layout: WallpaperLayout; side: 'top' | 'bottom'; available: boolean; desktop: boolean; busy: boolean;
  onChange: (layout: WallpaperLayout) => void; onSide: (side: 'top' | 'bottom') => void;
  onPick: (side: 'auto' | 'top' | 'bottom') => void;
}) {
  const current = layout[side];
  const name = side === 'top' ? 'Top' : 'Bottom';
  return <div className="wallpaper-controls">
    <div className="layer-tabs" aria-label="Wallpaper mode">
      <button type="button" aria-pressed={layout.mode==='auto'} className={layout.mode==='auto'?'chosen':''} onClick={()=>onChange({...layout,mode:'auto'})}>Auto</button>
      <button type="button" aria-pressed={layout.mode==='split'} className={layout.mode==='split'?'chosen':''} onClick={()=>onChange({...layout,mode:'split'})}>Split</button>
    </div>
    {layout.mode==='auto'?<div className="wallpaper-control"><span><CheckCircle2 size={14}/> {available?'Wallpaper ready':'Choose a wallpaper'}</span><button className="page-button" disabled={!desktop||busy} onClick={()=>onPick('auto')}><FolderOpen size={14}/> Change image</button><p className="field-help">One wallpaper automatically fills the output frame behind your video.</p></div>:<>
      <p className="field-help">Separate images above and below the video. Select an area, then drag it in Preview or adjust below.</p>
      <div className="wallpaper-side-tabs layer-tabs" aria-label="Wallpaper area">
        {(['top','bottom'] as const).map(area=><button type="button" key={area} aria-pressed={side===area} className={side===area?'chosen':''} onClick={()=>onSide(area)}>{area==='top'?'Top wallpaper':'Bottom wallpaper'}</button>)}
      </div>
      <button className="page-button wallpaper-image-button" disabled={!desktop||busy} onClick={()=>onPick(side)}><FolderOpen size={14}/> {current.path?'Change':'Choose'} {side} image</button>
      {!current.path&&<p className="field-help">Using your Auto wallpaper until you choose another image.</p>}
      {([
        ['zoom','Zoom',100,300], ['x','Horizontal position',0,100], ['y','Vertical position',0,100],
      ] as const).map(([key,label,min,max])=><label className="range-setting" key={key}><span>{label}<strong>{Math.round(current[key]*100)}%</strong></span><input type="range" aria-label={`${name} wallpaper ${label.toLowerCase()}`} min={min} max={max} step={1} value={Math.round(current[key]*100)} onChange={e=>onChange({...layout,[side]:{...current,[key]:Number(e.target.value)/100}})}/></label>)}
      <button className="scale-reset wallpaper-reset" onClick={()=>onChange({...layout,[side]:{...current,zoom:1,x:.5,y:.5}})}>Reset {side} position</button>
      <label className="range-setting video-position"><span>Video vertical position<strong>{Math.round(layout.video_y*100)}%</strong></span><input type="range" aria-label="Video vertical position" min={0} max={100} step={1} value={Math.round(layout.video_y*100)} onChange={e=>onChange({...layout,video_y:Number(e.target.value)/100})}/></label>
      <button className="scale-reset" onClick={()=>onChange({...layout,video_y:.5})}>Center video</button>
      <p className="field-help">Shrink the video or select a taller ratio to create room for both wallpapers. Each image stays inside its own area.</p>
    </>}
  </div>;
}
