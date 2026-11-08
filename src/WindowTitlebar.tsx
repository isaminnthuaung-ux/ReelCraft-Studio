import { getCurrentWindow } from '@tauri-apps/api/window';
import { Maximize2, Minus, X } from 'lucide-react';
import { desktop } from './bridge';
import { version, buildNumber } from '../package.json';

export default function WindowTitlebar({ onError }: { onError: (message: string) => void }) {
  async function act(action: 'close' | 'minimize' | 'maximize') {
    if (!desktop) return;
    try {
      const window = getCurrentWindow();
      if (action==='close') await window.close();
      else if (action==='minimize') await window.minimize();
      else await window.toggleMaximize();
    } catch (error) { onError(`Window control: ${error}`); }
  }
  return <div className="mac-titlebar" data-tauri-drag-region>
    <div className="traffic-lights" role="toolbar" aria-label="Window controls">
      <button className="traffic-button traffic-close" aria-label="Close window" title="Close window" disabled={!desktop} onClick={()=>void act('close')}><span className="traffic-circle"><X size={9}/></span></button>
      <button className="traffic-button traffic-minimize" aria-label="Minimize window" title="Minimize window" disabled={!desktop} onClick={()=>void act('minimize')}><span className="traffic-circle"><Minus size={9}/></span></button>
      <button className="traffic-button traffic-maximize" aria-label="Maximize or restore window" title="Maximize or restore window" disabled={!desktop} onClick={()=>void act('maximize')}><span className="traffic-circle"><Maximize2 size={8}/></span></button>
    </div>
    <span className="mac-window-title" data-tauri-drag-region>ReelCraftStudio-{version}-Build-{buildNumber}</span>
  </div>;
}
