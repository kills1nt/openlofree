import type { Layers } from './types';

export const cloneLayers = (layers: Layers): Layers => layers.map((l) => [...l]);

export const keyIndex = (cols: number, row: number, col: number) => row * cols + col;

/** A copy of `layers` with one key changed. */
export function withKey(layers: Layers, layer: number, index: number, code: number): Layers {
  return layers.map((l, i) => (i === layer ? l.map((c, j) => (j === index ? code : c)) : l));
}

export function isChanged(saved: Layers, draft: Layers, layer: number, index: number): boolean {
  return saved[layer]?.[index] !== draft[layer]?.[index];
}

/** How many keys differ, and on how many layers. */
export function changeSummary(saved: Layers, draft: Layers): { keys: number; layers: number } {
  let keys = 0;
  let layers = 0;
  draft.forEach((l, i) => {
    const n = l.reduce((sum, c, j) => sum + (saved[i]?.[j] !== c ? 1 : 0), 0);
    keys += n;
    if (n > 0) layers++;
  });
  return { keys, layers };
}
