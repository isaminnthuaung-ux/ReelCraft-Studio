import type { Settings, SourceItem } from './types';

export type ProjectLog = { time: string; message: string; error?: boolean };
export type Project = {
  id: string;
  name: string;
  updatedAt: string;
  sources: SourceItem[];
  settings: Settings;
  output: string;
  exported: number;
  logs: ProjectLog[];
};
export const historyKey = 'reelcraft.project-history.v1';

export function readProjects(): Project[] {
  const value: unknown = JSON.parse(localStorage.getItem(historyKey) || '[]');
  if (!Array.isArray(value)) throw new Error('Invalid project history');
  return value.filter((p): p is Project => p && typeof p.id === 'string'
    && typeof p.name === 'string' && typeof p.updatedAt === 'string'
    && Array.isArray(p.sources) && p.sources.every((s: SourceItem) => s?.source && typeof s.source.path === 'string' && Array.isArray(s.clips))
    && p.settings && typeof p.output === 'string' && typeof p.exported === 'number'
    && Array.isArray(p.logs)).slice(0, 30);
}

export function storeProject(project: Project): Project[] {
  const projects = [project, ...readProjects().filter(p => p.id !== project.id)].slice(0, 30);
  localStorage.setItem(historyKey, JSON.stringify(projects));
  return projects;
}
