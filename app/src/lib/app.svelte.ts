import { api } from './api';
import { changeSummary, cloneLayers, keyIndex, withKey } from './draft';
import { keyId } from './geometry';
import { t } from './strings';
import { toAppError } from './tauri';
import type {
  AppError,
  Backlight,
  BackupInfo,
  CatalogGroup,
  ConnectInfo,
  DeviceState,
  KeyDef,
  Layers,
  ProfileInfo,
} from './types';

export type Tab = 'keys' | 'lighting' | 'profiles' | 'device';
export type Phase = 'searching' | 'loading' | 'ready' | 'error';
export type Busy =
  | { kind: 'idle' }
  | { kind: 'writing' }
  | { kind: 'done'; message: string }
  | { kind: 'error'; message: string };
export type Note = { kind: 'ok' | 'error'; text: string } | null;

/** Errors that mean "the keyboard is not there right now", worth retrying quietly. */
const RETRYABLE = new Set(['not_found', 'timeout', 'io', 'no_session']);
const POLL_MS = 4000;
const PREVIEW_DEBOUNCE_MS = 80;

class AppStore {
  tab = $state<Tab>('keys');
  phase = $state<Phase>('searching');
  error = $state<AppError | null>(null);
  info = $state<ConnectInfo | null>(null);
  saved = $state<DeviceState | null>(null);
  draftLayers = $state<Layers>([]);
  draftBacklight = $state<Backlight>({ mode: 'steady', brightness: 255 });
  layer = $state(0);
  selected = $state<string | null>(null);
  legends = $state<Record<number, string>>({});
  catalog = $state<CatalogGroup[]>([]);
  profiles = $state<ProfileInfo[]>([]);
  profilesPhase = $state<'loading' | 'ready' | 'error'>('loading');
  profilesError = $state('');
  battery = $state<number | null>(null);
  backup = $state<BackupInfo | null>(null);
  keysBusy = $state<Busy>({ kind: 'idle' });
  lightBusy = $state<Busy>({ kind: 'idle' });
  profileNote = $state<Note>(null);
  announcement = $state('');

  #poll: ReturnType<typeof setInterval> | undefined;
  #previewTimer: ReturnType<typeof setTimeout> | undefined;
  #stayOffline = false;

  changes = $derived(
    this.saved ? changeSummary(this.saved.layers, this.draftLayers) : { keys: 0, layers: 0 },
  );
  lightChanged = $derived(
    !!this.saved &&
      (this.saved.backlight.mode !== this.draftBacklight.mode ||
        this.saved.backlight.brightness !== this.draftBacklight.brightness),
  );
  selectedKey = $derived<KeyDef | null>(
    this.info?.keys.find((k) => keyId(k) === this.selected) ?? null,
  );
  selectedCode = $derived.by(() => {
    const k = this.selectedKey;
    if (!k || !this.info) return null;
    return this.draftLayers[this.layer]?.[keyIndex(this.info.cols, k.row, k.col)] ?? null;
  });

  say(message: string) {
    this.announcement = '';
    queueMicrotask(() => (this.announcement = message));
  }

  legend(code: number): string {
    return this.legends[code] ?? `0x${code.toString(16).toUpperCase().padStart(4, '0')}`;
  }

  codeAt(layer: number, key: KeyDef): number {
    return this.draftLayers[layer]?.[keyIndex(this.info?.cols ?? 0, key.row, key.col)] ?? 0;
  }

  async start() {
    void this.refreshProfiles();
    void api.onProfileApplied(async (name, changed) => {
      this.profileNote = { kind: 'ok', text: t.profiles.applied(name, changed) };
      if (this.phase === 'ready') await this.load();
    });
    void api.onTrayError((message) => (this.profileNote = { kind: 'error', text: message }));
    await this.connect(false);
  }

  // ---- connection ----------------------------------------------------------------------

  async connect(demo: boolean, silent = false) {
    this.#stayOffline = false;
    if (!silent) {
      this.error = null;
      this.phase = 'searching';
    }
    try {
      this.info = await api.connect(demo);
      this.error = null;
      this.phase = 'loading';
      this.stopPolling();
      await this.load();
    } catch (e) {
      this.fail(toAppError(e));
    }
  }

  async disconnect(thenSearch: boolean) {
    this.stopPolling();
    await api.disconnect().catch(() => {});
    this.info = null;
    this.saved = null;
    this.draftLayers = [];
    this.selected = null;
    this.keysBusy = { kind: 'idle' };
    this.lightBusy = { kind: 'idle' };
    if (thenSearch) {
      await this.connect(false);
    } else {
      this.#stayOffline = true;
      this.error = { kind: 'not_found', message: t.device.offline };
      this.phase = 'searching';
    }
  }

  private fail(e: AppError) {
    this.error = e;
    if (RETRYABLE.has(e.kind)) {
      this.phase = 'searching';
      if (e.kind === 'io' || e.kind === 'no_session') this.info = null;
      this.startPolling();
    } else {
      this.phase = 'error';
    }
  }

  private startPolling() {
    if (this.#poll || this.#stayOffline) return;
    this.#poll = setInterval(() => {
      if (this.phase === 'searching') void this.connect(false, true);
    }, POLL_MS);
  }

  private stopPolling() {
    clearInterval(this.#poll);
    this.#poll = undefined;
  }

  /** Reads keymap, backlight, key catalog and legends. Also the "Read again" action. */
  async load() {
    if (!this.info) return;
    try {
      const [state, catalog, backup] = await Promise.all([
        api.readState(),
        api.catalog(),
        api.factoryBackupInfo(),
      ]);
      this.saved = state;
      this.draftLayers = cloneLayers(state.layers);
      this.draftBacklight = { ...state.backlight };
      this.catalog = catalog;
      this.backup = backup;
      if (this.layer >= this.info.layers) this.layer = 0;
      await this.ensureLegends([
        ...state.layers.flat(),
        ...catalog.flatMap((g) => g.items.map((i) => i.code)),
      ]);
      this.battery = await api.battery().catch(() => null);
      this.phase = 'ready';
    } catch (e) {
      this.fail(toAppError(e));
    }
  }

  private async ensureLegends(codes: number[]) {
    const missing = [...new Set(codes)].filter((c) => !(c in this.legends));
    if (missing.length === 0) return;
    const labels = await api.legends(missing);
    const next = { ...this.legends };
    missing.forEach((c, i) => (next[c] = labels[i] ?? ''));
    this.legends = next;
  }

  // ---- keys ----------------------------------------------------------------------------

  selectKey(id: string | null) {
    this.selected = id;
  }

  setLayer(n: number) {
    this.layer = n;
    this.say(t.keys.layer(n));
  }

  async assign(code: number) {
    const k = this.selectedKey;
    if (!k || !this.info) return;
    this.draftLayers = withKey(this.draftLayers, this.layer, keyIndex(this.info.cols, k.row, k.col), code);
    this.keysBusy = { kind: 'idle' };
    await this.ensureLegends([code]).catch(() => {});
    this.say(t.keys.selected(this.legend(code), `0x${code.toString(16).toUpperCase().padStart(4, '0')}`));
  }

  discard() {
    if (!this.saved) return;
    this.draftLayers = cloneLayers(this.saved.layers);
    this.keysBusy = { kind: 'idle' };
  }

  async applyKeymap() {
    if (!this.saved || this.keysBusy.kind === 'writing') return;
    this.keysBusy = { kind: 'writing' };
    try {
      const n = await api.applyKeymap(this.draftLayers);
      this.saved = { ...this.saved, layers: cloneLayers(this.draftLayers) };
      const message = t.bar.done(n);
      this.keysBusy = { kind: 'done', message };
      this.say(message);
      this.backup = await api.factoryBackupInfo().catch(() => this.backup);
    } catch (e) {
      const err = toAppError(e);
      this.keysBusy = { kind: 'error', message: err.message };
      this.say(`${t.bar.failed}. ${err.message}`);
      // A write may have landed before the failure, so show what the keyboard really holds now.
      api.readState().then((s) => (this.saved = s)).catch(() => {});
      if (err.kind === 'io') this.fail(err);
    }
  }

  // ---- lighting ------------------------------------------------------------------------

  previewBacklight(b: Backlight) {
    this.draftBacklight = b;
    this.lightBusy = { kind: 'idle' };
    clearTimeout(this.#previewTimer);
    this.#previewTimer = setTimeout(() => {
      api.previewBacklight(b).catch((e) => {
        this.lightBusy = { kind: 'error', message: `${t.lighting.previewFailed}. ${toAppError(e).message}` };
      });
    }, PREVIEW_DEBOUNCE_MS);
  }

  async applyBacklight() {
    if (!this.saved) return;
    clearTimeout(this.#previewTimer);
    this.lightBusy = { kind: 'writing' };
    try {
      await api.applyBacklight(this.draftBacklight);
      this.saved = { ...this.saved, backlight: { ...this.draftBacklight } };
      this.lightBusy = { kind: 'done', message: t.lighting.applied };
      this.say(t.lighting.applied);
    } catch (e) {
      this.lightBusy = { kind: 'error', message: toAppError(e).message };
    }
  }

  resetBacklight() {
    if (this.saved) this.previewBacklight({ ...this.saved.backlight });
  }

  // ---- profiles ------------------------------------------------------------------------

  async refreshProfiles() {
    this.profilesPhase = 'loading';
    try {
      this.profiles = await api.listProfiles();
      this.profilesPhase = 'ready';
    } catch (e) {
      this.profilesError = toAppError(e).message;
      this.profilesPhase = 'error';
    }
  }

  private async profileAction(run: () => Promise<string | null>) {
    this.profileNote = null;
    try {
      const text = await run();
      if (text) {
        this.profileNote = { kind: 'ok', text };
        this.say(text);
      }
    } catch (e) {
      this.profileNote = { kind: 'error', text: toAppError(e).message };
    }
  }

  saveProfile(name: string) {
    return this.profileAction(async () => {
      if (!this.saved) throw { kind: 'no_session', message: t.device.offline };
      this.profiles = await api.saveProfile(name.trim(), this.draftLayers, this.draftBacklight);
      return t.profiles.saved(name.trim());
    });
  }

  applyProfile(name: string) {
    return this.profileAction(async () => {
      const n = await api.applyProfile(name);
      await this.load();
      return t.profiles.applied(name, n);
    });
  }

  duplicateProfile(name: string, newName: string) {
    return this.profileAction(async () => {
      this.profiles = await api.duplicateProfile(name, newName.trim());
      return t.profiles.saved(newName.trim());
    });
  }

  deleteProfile(name: string) {
    return this.profileAction(async () => {
      this.profiles = await api.deleteProfile(name);
      return t.profiles.deleted(name);
    });
  }

  exportProfile(name: string) {
    return this.profileAction(async () => ((await api.exportProfile(name)) ? t.profiles.exported(name) : null));
  }

  importProfile() {
    return this.profileAction(async () => {
      const list = await api.importProfile();
      if (!list) return null;
      this.profiles = list;
      return t.profiles.imported;
    });
  }
}

export const app = new AppStore();
