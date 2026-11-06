export type Source = { path: string; name: string; duration: number; width: number; height: number; has_audio: boolean };
export type Clip = { n: number; start: number; end: number; cut: string; enabled: boolean };
export type SourceItem = { source: Source; clips: Clip[] };
export type TextLayer = { enabled: boolean; text: string; font: string; size: number; x: number; y: number; color: string; opacity: number; boxed: boolean };
export type Settings = { seconds: number; smart: boolean; extra: number; ratio: string; fill: string; text: TextLayer; label: TextLayer };
export type Progress = { phase: string; progress: number; message: string; clip: number; total: number };
export const defaults: Settings = {
  seconds: 180, smart: true, extra: 20, ratio: 'Original', fill: 'blur',
  text: { enabled: true, text: '', font: 'Myanmar Text', size: 54, x: .5, y: .82, color: '#ffffff', opacity: 1, boxed: true },
  label: { enabled: true, text: 'Part {n} / {total}', font: 'Segoe UI', size: 48, x: .5, y: .12, color: '#ffffff', opacity: 1, boxed: true },
};
export function clock(value: number) {
  if (!Number.isFinite(value)) return '00:00';
  const v = Math.max(0, Math.round(value));
  return v >= 3600 ? `${Math.floor(v / 3600)}:${String(Math.floor(v / 60) % 60).padStart(2, '0')}:${String(v % 60).padStart(2, '0')}` : `${String(Math.floor(v / 60)).padStart(2, '0')}:${String(v % 60).padStart(2, '0')}`;
}
export function dimensions(s: Source, ratio: string): [number, number] {
  return ratio === '9:16' ? [1080, 1920] : ratio === '16:9' ? [1920, 1080] : ratio === '1:1' ? [1080, 1080] : [Math.floor(s.width / 2) * 2, Math.floor(s.height / 2) * 2];
}
