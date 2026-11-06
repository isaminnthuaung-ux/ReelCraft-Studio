import type { CSSProperties } from 'react';

type IconProps = { size?: number | string; className?: string; strokeWidth?: number; style?: CSSProperties };
function asset(file: string) {
  return function VidCoderIcon({ size = 20, className = '', style }: IconProps) {
    return <img className={`vidcoder-icon ${className}`} src={`/icons/vidcoder/${file}`} width={size} height={size} style={style} alt="" aria-hidden="true" draggable={false}/>;
  };
}
export const Film = asset('video-file.png');
export const Folder = asset('folder.png');
export const FolderOpen = asset('add_folder.png');
export const ListVideo = asset('AddToQueue.png');
export const Clapperboard = asset('Preview64.png');
export const Settings2 = asset('Options.png');
export const Columns3 = asset('EncodingSettings.png');
export const Volume2 = asset('Audio.png');
export const Type = asset('text_file_16x16.png');
export const Upload = asset('ImportExport.png');
export const Square = asset('Stop.png');
export const CircleHelp = asset('Help.png');
export const Monitor = asset('Windows.png');
export const CheckCircle2 = asset('succeeded.png');
export const ArrowDownToLine = asset('Encode.png');
