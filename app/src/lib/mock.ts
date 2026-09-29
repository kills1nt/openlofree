// In-memory stand-in for the Rust backend, used when the UI runs in a plain browser.
// URL switches: ?mock=connected (a "real" keyboard is found), ?mock=timeout (keyboard asleep for
// the first two tries), ?mock=writefail (writes fail verification). Without one, no keyboard is found.
import layout from '../../../layouts/flow2-mac-84.json';
import { catalog, legend } from './legends';
import { parseDefinition } from './parseLayout';
import type { Api, AppError, Backlight, ConnectInfo, DeviceState, Layers, ProfileInfo } from './types';

const TRNS = 1;
const LAYERS = 6;

// Layer 0 of a real 84-key Flow 2 Mac, as (row, col, code). Same table as src-tauri/src/demo.rs.
const LAYER0: [number, number, number][] = [
  [0,0,0x29],[0,1,0xbe],[0,2,0xbd],[0,3,0x7e0b],[0,4,0x7e0f],[0,5,0x7803],[0,6,0x7804],[0,7,0xac],[0,8,0xae],[0,9,0xab],[0,10,0xa8],[0,11,0xaa],[0,12,0xa9],[0,13,0x7e0d],[0,14,0x4c],
  [1,0,0x35],[1,1,0x1e],[1,2,0x1f],[1,3,0x20],[1,4,0x21],[1,5,0x22],[1,6,0x23],[1,7,0x24],[1,8,0x25],[1,9,0x26],[1,10,0x27],[1,11,0x2d],[1,12,0x2e],[1,13,0x2a],[1,14,0x4a],
  [2,0,0x2b],[2,1,0x14],[2,2,0x1a],[2,3,0x08],[2,4,0x15],[2,5,0x17],[2,6,0x1c],[2,7,0x18],[2,8,0x0c],[2,9,0x12],[2,10,0x13],[2,11,0x2f],[2,12,0x30],[2,13,0x31],[2,14,0x4d],
  [3,0,0x39],[3,1,0x04],[3,2,0x16],[3,3,0x07],[3,4,0x09],[3,5,0x0a],[3,6,0x0b],[3,7,0x0d],[3,8,0x0e],[3,9,0x0f],[3,10,0x33],[3,11,0x34],[3,13,0x28],[3,14,0x4b],
  [4,0,0xe1],[4,2,0x1d],[4,3,0x1b],[4,4,0x06],[4,5,0x19],[4,6,0x05],[4,7,0x11],[4,8,0x10],[4,9,0x36],[4,10,0x37],[4,11,0x38],[4,12,0xe5],[4,13,0x52],[4,14,0x4e],
  [5,0,0x5221],[5,1,0xe0],[5,2,0xe2],[5,3,0xe3],[5,6,0x2c],[5,9,0xe7],[5,10,0xe6],[5,11,0xe4],[5,12,0x50],[5,13,0x51],[5,14,0x4f],
];
const LAYER1: [number, number, number][] = [
  [1,1,0x3a],[1,2,0x3b],[1,3,0x3c],[1,4,0x3d],[1,5,0x3e],[1,6,0x3f],[1,7,0x40],[1,8,0x41],[1,9,0x42],[1,10,0x43],[1,11,0x44],[1,12,0x45],
  [2,1,0x7793],[2,2,0x7794],[2,3,0x7795],[2,4,0x7785],
];

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));
const err = (kind: string, message: string): AppError => ({ kind, message });

export function mockApi(): Api {
  const def = parseDefinition(layout);
  const flag = typeof location === 'undefined' ? '' : (new URLSearchParams(location.search).get('mock') ?? '');
  const cols = def.cols;

  const seed = (): Layers => {
    const layers: Layers = Array.from({ length: LAYERS }, () => new Array<number>(def.rows * cols).fill(TRNS));
    for (const [r, c, code] of LAYER0) layers[0]![r * cols + c] = code;
    for (const [r, c, code] of LAYER1) layers[1]![r * cols + c] = code;
    return layers;
  };

  let connected = false;
  let demo = false;
  let tries = 0;
  let device: DeviceState = { layers: seed(), backlight: { mode: 'steady', brightness: 255 } };
  let savedBacklight: Backlight = { ...device.backlight };
  let profiles: (ProfileInfo & { layersData: Layers; backlight: Backlight })[] = [];

  const info = (): ConnectInfo => ({
    modelId: 'flow2-mac-84',
    label: 'Flow 2 Mac 84',
    verified: true,
    demo,
    mac: true,
    product: demo ? 'Demo keyboard (no hardware)' : 'Flow2@Lofree',
    vendorId: 0x388d,
    productId: def.productId,
    protocol: 12,
    layers: LAYERS,
    rows: def.rows,
    cols,
    keys: def.keys,
  });
  const list = (): ProfileInfo[] =>
    profiles.map(({ name, model, layers }) => ({ name, model, layers })).sort((a, b) => a.name.localeCompare(b.name));
  const need = () => {
    if (!connected) throw err('no_session', 'No keyboard is connected.');
  };

  return {
    async connect(wantDemo) {
      await sleep(250);
      if (!wantDemo) {
        if (flag === 'timeout' && tries++ < 2) throw err('timeout', 'no response from the keyboard');
        if (flag !== 'connected' && flag !== 'writefail' && flag !== 'timeout') {
          throw err('not_found', 'no Flow 2 found. Connect it with USB-C: Bluetooth cannot be configured');
        }
      }
      connected = true;
      demo = wantDemo;
      device = { layers: seed(), backlight: { mode: 'steady', brightness: 255 } };
      savedBacklight = { ...device.backlight };
      return info();
    },
    async disconnect() {
      connected = false;
    },
    async readState() {
      need();
      await sleep(700);
      return { layers: device.layers.map((l) => [...l]), backlight: { ...savedBacklight } };
    },
    async setKey(layer, row, col, code) {
      need();
      device.layers[layer]![row * cols + col] = code;
    },
    async applyKeymap(layers) {
      need();
      await sleep(700);
      if (flag === 'writefail') throw err('verify_failed', 'write not confirmed at layer 0 row 1 col 2: wrote 0x0004, read back 0x0000');
      let changed = 0;
      layers.forEach((l, i) =>
        l.forEach((code, j) => {
          if (device.layers[i]![j] !== code) {
            device.layers[i]![j] = code;
            changed++;
          }
        }),
      );
      return changed;
    },
    async previewBacklight(b) {
      need();
      device.backlight = { ...b };
    },
    async applyBacklight(b) {
      need();
      await sleep(300);
      device.backlight = { ...b };
      savedBacklight = { ...b };
    },
    async listProfiles() {
      await sleep(200);
      return list();
    },
    async saveProfile(name, layers, backlight) {
      const trimmed = name.trim();
      profiles = profiles.filter((p) => p.name !== trimmed);
      profiles.push({ name: trimmed, model: 'flow2-mac-84', layers: layers.length, layersData: layers.map((l) => [...l]), backlight: { ...backlight } });
      return list();
    },
    async applyProfile(name) {
      need();
      await sleep(500);
      const p = profiles.find((x) => x.name === name);
      if (!p) throw err('profile', `no profile named "${name}"`);
      let changed = 0;
      p.layersData.forEach((l, i) => l.forEach((code, j) => { if (device.layers[i]![j] !== code) { device.layers[i]![j] = code; changed++; } }));
      device.backlight = { ...p.backlight };
      savedBacklight = { ...p.backlight };
      return changed;
    },
    async duplicateProfile(name, newName) {
      const p = profiles.find((x) => x.name === name);
      if (!p) throw err('profile', `no profile named "${name}"`);
      if (profiles.some((x) => x.name.toLowerCase() === newName.trim().toLowerCase())) throw err('profile', `a profile named "${newName}" already exists`);
      profiles.push({ ...p, name: newName.trim() });
      return list();
    },
    async deleteProfile(name) {
      profiles = profiles.filter((p) => p.name !== name);
      return list();
    },
    async exportProfile() {
      await sleep(150);
      return true;
    },
    async importProfile() {
      await sleep(150);
      const layers = seed();
      profiles.push({ name: 'Imported profile', model: 'flow2-mac-84', layers: layers.length, layersData: layers, backlight: { mode: 'steady', brightness: 200 } });
      return list();
    },
    async battery() {
      return flag === 'connected' ? 87 : null;
    },
    async catalog() {
      need();
      return catalog(true, LAYERS);
    },
    async legends(codes) {
      return codes.map((c) => legend(c, true));
    },
    async factoryBackupInfo() {
      need();
      return demo ? { path: null, exists: false } : { path: 'C:\\Users\\you\\AppData\\Roaming\\openlofree\\factory-backup-flow2-mac-84.json', exists: true };
    },
    async onProfileApplied() {
      return () => {};
    },
    async onTrayError() {
      return () => {};
    },
  };
}
