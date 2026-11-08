import type { LucideIcon, LucideProps } from 'lucide-react';
import {
  FileVideo2, Folder as FolderIcon, FolderOpen as FolderOpenIcon,
  ListVideo as ListVideoIcon, Clapperboard as ClapperboardIcon,
  Settings2 as SettingsIcon, Settings as GearIcon, PanelLeft, Volume2 as VolumeIcon,
  Type as TypeIcon, Upload as UploadIcon, Square as StopIcon,
  CircleHelp as HelpIcon, Monitor as MonitorIcon,
  CircleCheck, Download,
} from 'lucide-react';

// Current-color outline symbols follow the Big Sur kit's toolbar treatment.
function symbol(Icon: LucideIcon) {
  return function ToolbarIcon({ size = 20, className = '', strokeWidth = 1.6, ...props }: LucideProps) {
    return <Icon {...props} size={size} strokeWidth={strokeWidth} className={`app-icon ${className}`} aria-hidden="true" />;
  };
}
export const Film = symbol(FileVideo2);
export const Folder = symbol(FolderIcon);
export const FolderOpen = symbol(FolderOpenIcon);
export const ListVideo = symbol(ListVideoIcon);
export const Clapperboard = symbol(ClapperboardIcon);
export const Settings2 = symbol(SettingsIcon);
export const Settings = symbol(GearIcon);
export const Columns3 = symbol(PanelLeft);
export const Volume2 = symbol(VolumeIcon);
export const Type = symbol(TypeIcon);
export const Upload = symbol(UploadIcon);
export const Square = symbol(StopIcon);
export const CircleHelp = symbol(HelpIcon);
export const Monitor = symbol(MonitorIcon);
export const CheckCircle2 = symbol(CircleCheck);
export const ArrowDownToLine = symbol(Download);
