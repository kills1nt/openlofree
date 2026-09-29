// Port of flow2-core's keycodes.rs, used only by the browser mock. The app gets labels from Rust.
import type { CatalogGroup } from './types';

const media: Record<number, string> = {
  0xa8: 'Mute', 0xa9: 'Vol+', 0xaa: 'Vol-', 0xab: 'Next', 0xac: 'Prev', 0xad: 'Stop', 0xae: 'Play',
  0xaf: 'Media', 0xb0: 'Eject', 0xb1: 'Mail', 0xb2: 'Calc', 0xb3: 'My PC', 0xb4: 'Search',
  0xb5: 'WWW Home', 0xb6: 'WWW Back', 0xb7: 'WWW Fwd', 0xb8: 'WWW Stop', 0xb9: 'Refresh',
  0xba: 'Favorites', 0xbb: 'FF', 0xbc: 'Rew', 0xbd: 'Bright+', 0xbe: 'Bright-',
};
const named: Record<number, string> = {
  0x0: 'None', 0x1: 'Trns', 0x27: '0', 0x28: 'Enter', 0x29: 'Esc', 0x2a: 'Bksp', 0x2b: 'Tab',
  0x2c: 'Space', 0x2d: '-', 0x2e: '=', 0x2f: '[', 0x30: ']', 0x31: '\\', 0x33: ';', 0x34: "'",
  0x35: '`', 0x36: ',', 0x37: '.', 0x38: '/', 0x39: 'Caps', 0x46: 'PrtSc', 0x47: 'ScrLk',
  0x48: 'Pause', 0x49: 'Ins', 0x4a: 'Home', 0x4b: 'PgUp', 0x4c: 'Del', 0x4d: 'End', 0x4e: 'PgDn',
  0x4f: 'Right', 0x50: 'Left', 0x51: 'Down', 0x52: 'Up', 0xe0: 'Ctrl', 0xe1: 'Shift', 0xe4: 'R Ctrl',
  0xe5: 'R Shift', 0x7793: 'BT1', 0x7794: 'BT2', 0x7795: 'BT3', 0x7785: '2.4G',
};

export function legend(code: number, mac: boolean): string {
  const alt = mac ? 'Opt' : 'Alt';
  const gui = mac ? 'Cmd' : 'Win';
  if (code === 0xe2) return alt;
  if (code === 0xe3) return gui;
  if (code === 0xe6) return `R ${alt}`;
  if (code === 0xe7) return `R ${gui}`;
  if (named[code] !== undefined) return named[code];
  if (media[code] !== undefined) return media[code];
  if (code >= 0x04 && code <= 0x1d) return String.fromCharCode(65 + code - 0x04);
  if (code >= 0x1e && code <= 0x26) return String(code - 0x1d);
  if (code >= 0x3a && code <= 0x45) return `F${code - 0x39}`;
  if (code >= 0x7e00 && code <= 0x7e3f) return `Custom ${code - 0x7e00}`;
  if (code >= 0x5200 && code < 0x5220) return `TO(${code - 0x5200})`;
  if (code >= 0x5220 && code < 0x5240) return `MO(${code - 0x5220})`;
  if (code >= 0x5260 && code < 0x5280) return `TG(${code - 0x5260})`;
  return `0x${code.toString(16).toUpperCase().padStart(4, '0')}`;
}

const range = (from: number, to: number) => Array.from({ length: to - from + 1 }, (_, i) => from + i);

export function catalog(mac: boolean, layers: number): CatalogGroup[] {
  const group = (name: string, codes: number[]): CatalogGroup => ({
    name,
    items: codes.map((code) => ({ code, label: legend(code, mac) })),
  });
  const layerKeys = range(0, Math.min(layers, 32) - 1).flatMap((n) => [0x5220 + n, 0x5260 + n, 0x5200 + n]);
  return [
    group('Letters', range(0x04, 0x1d)),
    group('Numbers', range(0x1e, 0x27)),
    group('Punctuation', [0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38]),
    group('Editing', [0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x39, 0x4c, 0x49, 0x46, 0x47, 0x48]),
    group('Navigation', [0x4a, 0x4d, 0x4b, 0x4e, 0x52, 0x51, 0x50, 0x4f]),
    group('Function', range(0x3a, 0x45)),
    group('Modifiers', range(0xe0, 0xe7)),
    group('Media', range(0xa8, 0xbe)),
    group('Layers', layerKeys),
    group('Wireless', [0x7793, 0x7794, 0x7795, 0x7785]),
    group('Special', [0x0, 0x1]),
  ];
}
