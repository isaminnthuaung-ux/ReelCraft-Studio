import { useEffect, useState } from 'react';
import { CircuitBoard, Cpu, MemoryStick, Monitor } from 'lucide-react';
import { call, desktop } from './bridge';

type Hardware = { cpu: string; gpus: string[]; ram_bytes: number | null; os: string };
export default function SystemInfo() {
  const [info,setInfo]=useState<Hardware | null>(null);
  const [failed,setFailed]=useState(false);
  useEffect(()=>{
    if(!desktop) return;
    let disposed=false;
    call<Hardware>('system_info').then(value=>{if(!disposed)setInfo(value);}).catch(()=>{if(!disposed)setFailed(true);});
    return ()=>{disposed=true;};
  },[]);
  const fallback=!desktop?'Desktop app required':failed?'Unavailable':'Reading…';
  const entries=[
    {name:'CPU',icon:Cpu,value:info?.cpu,kind:'cpu'},
    {name:'GPU',icon:CircuitBoard,value:info?.gpus?.join(' + ')|| (info?'Unavailable':undefined),kind:'gpu'},
    {name:'RAM',icon:MemoryStick,value:info?(info.ram_bytes?`${(info.ram_bytes/1024**3).toFixed(1)} GiB`:'Unavailable'):undefined,kind:'ram'},
    {name:'OS',icon:Monitor,value:info?.os,kind:'os'},
  ];
  return <dl className="system-info" aria-label="System information">{entries.map(({name,icon:Icon,value,kind})=><div key={name} className={`system-spec system-${kind}`} title={`${name}: ${value||fallback}`}>
    <span className="system-spec-icon"><Icon size={17}/></span><div><dt>{name}</dt><dd>{value||fallback}</dd></div>
  </div>)}</dl>;
}
