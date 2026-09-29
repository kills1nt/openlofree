import { mockApi } from './mock';
import { tauriApi } from './tauri';
import type { Api } from './types';

const inTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

/** The Tauri backend inside the app, an in-memory mock in a plain browser (dev and checks). */
export const api: Api = inTauri ? tauriApi : mockApi();
