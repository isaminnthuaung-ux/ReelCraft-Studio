import { useRef } from 'react';
import type { PointerEvent } from 'react';
import { Move } from 'lucide-react';
import { dimensions } from './types';
import type { Settings, Source, WallpaperLayout } from './types';

export default function WallpaperHandles({ source, settings, side, disabled, onSide, onChange }: {
  source: Source; settings: Settings; side: 'top' | 'bottom'; disabled: boolean;
  onSide: (side: 'top' | 'bottom') => void; onChange: (layout: WallpaperLayout) => void;
}) {
  const drag = useRef<{ side: 'top' | 'bottom'; pointer: number; x: number; y: number; width: number; height: number; layout: WallpaperLayout } | null>(null);
  const [w,h] = dimensions(source,settings.ratio);
  const fit = Math.min(w/source.width,h/source.height);
  const videoHeight = Math.max(2,Math.round(source.height*fit*settings.scale/2)*2);
  const offset = (h-videoHeight)*settings.wallpaper_layout.video_y;
  const videoY = Math.sign(offset)*Math.round(Math.abs(offset)/2)*2;
  const top = Math.max(0,Math.min(h,videoY));
  const bottomY = Math.max(0,Math.min(h,videoY+videoHeight));
  const clamp = (n: number) => Math.max(0,Math.min(1,n));
  function move(event: PointerEvent<HTMLButtonElement>) {
    const start = drag.current;
    if(!start || start.pointer!==event.pointerId) return;
    const layer = start.layout[start.side];
    onChange({...start.layout,[start.side]:{...layer,x:clamp(layer.x-(event.clientX-start.x)/start.width),y:clamp(layer.y-(event.clientY-start.y)/start.height)}});
  }
  return <>{([{side:'top',y:0,height:top},{side:'bottom',y:bottomY,height:h-bottomY}] as const).filter(region=>region.height>=2).map(region=><button
    type="button" key={region.side} className={`wallpaper-handle ${side===region.side?'selected':''}`}
    style={{top:`${region.y/h*100}%`,height:`${region.height/h*100}%`}}
    aria-label={`Move ${region.side} wallpaper`} title="Drag to pan; arrow keys adjust position" disabled={disabled}
    onPointerDown={event=>{
      onSide(region.side); event.currentTarget.setPointerCapture(event.pointerId);
      const bounds=event.currentTarget.getBoundingClientRect();
      drag.current={side:region.side,pointer:event.pointerId,x:event.clientX,y:event.clientY,width:bounds.width,height:bounds.height,layout:settings.wallpaper_layout};
    }} onPointerMove={move} onPointerUp={()=>{drag.current=null;}} onPointerCancel={()=>{drag.current=null;}}
    onFocus={()=>onSide(region.side)} onKeyDown={event=>{
      if(!['ArrowLeft','ArrowRight','ArrowUp','ArrowDown'].includes(event.key)) return;
      event.preventDefault();
      const layer=settings.wallpaper_layout[region.side];
      onChange({...settings.wallpaper_layout,[region.side]:{...layer,
        x:clamp(layer.x+(event.key==='ArrowLeft'?-.02:event.key==='ArrowRight'?.02:0)),
        y:clamp(layer.y+(event.key==='ArrowUp'?-.02:event.key==='ArrowDown'?.02:0))}});
    }}><span><Move size={12}/>{region.side==='top'?'Top wallpaper':'Bottom wallpaper'}</span>
  </button>)}</>;
}
