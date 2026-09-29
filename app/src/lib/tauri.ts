import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { open, save } from '@tauri-apps/plugin-dialog';
import type { Api, AppError } from './types';

/** Tauri rejects with the serialized Rust `AppError`, anything else becomes an internal error. */
export function toAppError(e: unknown): AppError {
  if (e && typeof e === 'object' && 'kind' in e && 'message' in e) return e as AppError;
  return { kind: 'internal', message: e instanceof Error ? e.message : String(e) };
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (e) {
    throw toAppError(e);
  }
}

const jsonFilter = [{ name: 'openlofree profile', extensions: ['json'] }];

export const tauriApi: Api = {
  connect: (demo) => call('connect', { demo }),
  disconnect: () => call('disconnect'),
  readState: () => call('read_state'),
  setKey: (layer, row, col, code) => call('set_key', { layer, row, col, code }),
  applyKeymap: (layers) => call('apply_keymap', { layers }),
  previewBacklight: (b) => call('preview_backlight', { mode: b.mode, brightness: b.brightness }),
  applyBacklight: (b) => call('apply_backlight', { mode: b.mode, brightness: b.brightness }),
  listProfiles: () => call('list_profiles'),
  saveProfile: (name, layers, backlight) => call('save_profile', { name, layers, backlight }),
  applyProfile: (name) => call('apply_profile', { name }),
  duplicateProfile: (name, newName) => call('duplicate_profile', { name, newName }),
  deleteProfile: (name) => call('delete_profile', { name }),
  async exportProfile(name) {
    const path = await save({ defaultPath: `${name}.json`, filters: jsonFilter });
    if (!path) return false;
    await call('export_profile', { name, path });
    return true;
  },
  async importProfile() {
    const path = await open({ multiple: false, directory: false, filters: jsonFilter });
    if (typeof path !== 'string') return null;
    return call('import_profile', { path });
  },
  battery: () => call('battery'),
  catalog: () => call('catalog'),
  legends: (codes) => call('legends', { codes }),
  factoryBackupInfo: () => call('factory_backup_info'),
  onProfileApplied: (handler) =>
    listen<{ name: string; changed: number }>('profile-applied', (e) =>
      handler(e.payload.name, e.payload.changed),
    ),
  onTrayError: (handler) => listen<string>('tray-error', (e) => handler(e.payload)),
};
