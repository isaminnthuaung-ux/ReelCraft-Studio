import { useEffect, useMemo, useRef, useState } from 'react';
import { Check, ChevronDown, Search } from 'lucide-react';

const ROW_HEIGHT = 36;
const LIST_HEIGHT = 240;
const OVERSCAN = 3;

export default function FontPicker({ fonts, value, onChange }: { fonts: string[]; value: string; onChange: (font: string) => void }) {
  const [expanded, setExpanded] = useState(false);
  const [query, setQuery] = useState('');
  const [scrollTop, setScrollTop] = useState(0);
  const [active, setActive] = useState(0);
  const root = useRef<HTMLDivElement>(null);
  const search = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLDivElement>(null);
  const trigger = useRef<HTMLButtonElement>(null);
  const indexed = useMemo(() => fonts.map(name => ({ name, key: name.toLocaleLowerCase() })), [fonts]);
  const filtered = useMemo(() => {
    const key = query.toLocaleLowerCase();
    return indexed.filter(font => font.key.includes(key)).map(font => font.name);
  }, [indexed, query]);
  const start = Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - OVERSCAN);
  const end = Math.min(filtered.length, Math.ceil((scrollTop + LIST_HEIGHT) / ROW_HEIGHT) + OVERSCAN);
  useEffect(() => {
    if (!expanded) return;
    search.current?.focus();
    const close = (event: PointerEvent) => { if (!root.current?.contains(event.target as Node)) setExpanded(false); };
    document.addEventListener('pointerdown', close);
    return () => document.removeEventListener('pointerdown', close);
  }, [expanded]);
  useEffect(() => {
    if (!expanded || !list.current) return;
    const top = active * ROW_HEIGHT;
    if (top < list.current.scrollTop) list.current.scrollTop = top;
    else if (top + ROW_HEIGHT > list.current.scrollTop + LIST_HEIGHT) list.current.scrollTop = top + ROW_HEIGHT - LIST_HEIGHT;
    setScrollTop(list.current.scrollTop);
  }, [active, expanded]);
  const choose = (font: string) => {
    setExpanded(false); onChange(font); trigger.current?.focus();
  };
  return <div className="font-picker" ref={root} onKeyDown={e => {
    if (e.key === 'Escape') { setExpanded(false); trigger.current?.focus(); }
  }}>
    <button ref={trigger} type="button" aria-label="Font family" aria-haspopup="listbox" aria-expanded={expanded} disabled={!fonts.length} className="font-picker-trigger" onClick={() => {
      setQuery(''); const index = Math.max(0, fonts.indexOf(value)); setActive(index);
      setScrollTop(Math.max(0, index * ROW_HEIGHT - LIST_HEIGHT + ROW_HEIGHT)); setExpanded(!expanded);
    }}><span>{value || 'Choose a Windows font'}</span><ChevronDown size={14}/></button>
    {expanded && <div className="font-picker-popup">
      <div className="font-search"><Search size={13}/><input ref={search} type="search" role="combobox" aria-label="Search Windows fonts" aria-expanded="true" aria-controls="installed-font-list" aria-autocomplete="list" aria-activedescendant={filtered[active] ? `font-option-${active}` : undefined}
        placeholder="Search Windows fonts…" value={query} onChange={e => {
          setQuery(e.target.value); setActive(0); setScrollTop(0); if (list.current) list.current.scrollTop = 0;
        }} onKeyDown={e => {
          if (['ArrowDown', 'ArrowUp', 'Home', 'End'].includes(e.key)) {
            e.preventDefault(); setActive(index => e.key === 'Home' ? 0 : e.key === 'End' ? Math.max(0, filtered.length - 1)
              : Math.max(0, Math.min(filtered.length - 1, index + (e.key === 'ArrowDown' ? 1 : -1))));
          } else if (e.key === 'Enter' && filtered[active]) { e.preventDefault(); choose(filtered[active]); }
        }}/></div>
      <div ref={list} className="font-options virtual-font-options" role="listbox" id="installed-font-list" aria-label="Installed fonts" onScroll={e => setScrollTop(e.currentTarget.scrollTop)}>
        <div className="font-options-spacer" style={{ height: filtered.length * ROW_HEIGHT }}>
          {filtered.slice(start, end).map((font, offset) => {
            const index = start + offset;
            // Keep names in the UI font. The video overlay previews the actual
            // selection without loading hundreds of font faces into WebView.
            return <button type="button" key={font} id={`font-option-${index}`} role="option" aria-selected={font === value} aria-posinset={index + 1} aria-setsize={filtered.length}
              className={`font-option ${active === index ? 'keyboard-active' : ''}`} style={{ top: index * ROW_HEIGHT, height: ROW_HEIGHT }} tabIndex={-1}
              onClick={() => choose(font)}><span>{font}</span>{font === value && <Check size={13}/>}</button>;
          })}
        </div>
        {!filtered.length && <p>No matching fonts installed.</p>}
      </div>
    </div>}
  </div>;
}
