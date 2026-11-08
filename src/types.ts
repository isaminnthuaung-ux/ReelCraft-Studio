export type Source = { path: string; name: string; duration: number; width: number; height: number; has_audio: boolean };
export type Clip = { n: number; start: number; end: number; cut: string; enabled: boolean };
export type SourceItem = { source: Source; clips: Clip[] };
export type TextLayer = { enabled: boolean; text: string; font: string; size: number; x: number; y: number; color: string; opacity: number; boxed: boolean };
export type WallpaperLayer = { path: string; zoom: number; x: number; y: number };
export type WallpaperLayout = { mode: 'auto' | 'split'; top: WallpaperLayer; bottom: WallpaperLayer; video_y: number };
export type EncodingProfile = 'fastest' | 'fast' | 'balanced' | 'quality';
export const encodingProfiles: { value: EncodingProfile; label: string; description: string }[] = [
  { value: 'fastest', label: 'Fastest — P1', description: 'Maximum NVIDIA encoding speed. Allows larger files to preserve detail.' },
  { value: 'fast', label: 'Fast — P3', description: 'Fast NVIDIA encoding with improved compression.' },
  { value: 'balanced', label: 'Balanced — P5', description: 'The previous NVIDIA default. Balances speed and compression.' },
  { value: 'quality', label: 'Quality — P7', description: 'Prioritizes compression quality. Takes longer to encode.' },
];
export type Settings = { seconds: number; smart: boolean; extra: number; ratio: string; fill: string; scale: number; wallpaper: string; wallpaper_layout: WallpaperLayout; encoding_profile: EncodingProfile; text: TextLayer; label: TextLayer };
export type Progress = { phase: string; progress: number; message: string; clip: number; total: number };
export const defaults: Settings = {
  seconds: 180, smart: true, extra: 20, ratio: 'Original', fill: 'blur', scale: 1, wallpaper: '', encoding_profile: 'fastest',
  wallpaper_layout: { mode: 'auto', top: { path: '', zoom: 1, x: .5, y: .5 }, bottom: { path: '', zoom: 1, x: .5, y: .5 }, video_y: .5 },
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
