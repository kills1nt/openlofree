// Port of flow2-core's layout parser. Used only by the browser mock, the real app gets keys from Rust.
import type { KeyDef } from './types';

export interface Definition {
  name: string;
  vendorId: number;
  productId: number;
  rows: number;
  cols: number;
  keys: KeyDef[];
}

type Item = string | { x?: number; y?: number; w?: number; h?: number };

export function parseDefinition(json: {
  name: string;
  vendorId: string;
  productId: string;
  matrix: { rows: number; cols: number };
  layouts: { keymap: Item[][] };
}): Definition {
  const keys: KeyDef[] = [];
  let y = 0;
  for (const row of json.layouts.keymap) {
    let x = 0;
    let w = 1;
    let h = 1;
    for (const item of row) {
      if (typeof item === 'string') {
        const [r, c] = (item.split('\n')[0] ?? '').split(',').map((n) => Number.parseInt(n, 10));
        if (r === undefined || c === undefined || Number.isNaN(r) || Number.isNaN(c)) {
          throw new Error(`bad key label ${item}`);
        }
        keys.push({ row: r, col: c, x, y, w, h });
        x += w;
        w = 1;
        h = 1;
      } else {
        x += item.x ?? 0;
        y += item.y ?? 0;
        w = item.w ?? w;
        h = item.h ?? h;
      }
    }
    y += 1;
  }
  return {
    name: json.name,
    vendorId: Number.parseInt(json.vendorId, 16),
    productId: Number.parseInt(json.productId, 16),
    rows: json.matrix.rows,
    cols: json.matrix.cols,
    keys,
  };
}
