import { invoke, isTauri } from '@tauri-apps/api/core';
export const desktop = isTauri();
export async function call<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  if (!desktop) throw new Error('Open the Windows desktop app to process videos. This browser view previews the interface only.');
  return invoke<T>(name, args);
}
