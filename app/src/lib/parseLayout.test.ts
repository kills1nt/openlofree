import { describe, expect, it } from 'vitest';
import layout from '../../../layouts/flow2-mac-84.json';
import { parseDefinition } from './parseLayout';

describe('parseDefinition', () => {
  it('computes KLE geometry like the Rust parser', () => {
    const def = parseDefinition({
      name: 'Tiny',
      vendorId: '0x388d',
      productId: '0x00AB',
      matrix: { rows: 2, cols: 3 },
      layouts: { keymap: [[{ w: 1.5 }, '0,0', '0,1', { x: 0.5 }, '0,2'], [{ y: 0.25 }, '1,0\nDecal', { w: 2 }, '1,1']] },
    });
    expect(def.productId).toBe(0xab);
    expect(def.keys[1]!.x).toBe(1.5);
    expect(def.keys[2]!.x).toBe(3);
    expect(def.keys[3]).toEqual({ row: 1, col: 0, x: 0, y: 1.25, w: 1, h: 1 });
    expect(def.keys[4]!.w).toBe(2);
  });

  it('reads the real 84-key layout: 84 keys, every row 16 units wide', () => {
    const def = parseDefinition(layout);
    expect(def.keys).toHaveLength(84);
    expect(def.productId).toBe(0x28);
    for (let r = 0; r < def.rows; r++) {
      const w = def.keys.filter((k) => k.row === r).reduce((s, k) => s + k.w, 0);
      expect(w).toBe(16);
    }
  });

  it('rejects a malformed key label', () => {
    expect(() =>
      parseDefinition({ name: 'x', vendorId: '0x1', productId: '0x2', matrix: { rows: 1, cols: 1 }, layouts: { keymap: [['nope']] } }),
    ).toThrow();
  });
});
