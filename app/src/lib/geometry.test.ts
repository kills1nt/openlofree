import { describe, expect, it } from 'vitest';
import { bounds, neighbor } from './geometry';
import type { KeyDef } from './types';

const k = (row: number, col: number, x: number, y: number, w = 1): KeyDef => ({ row, col, x, y, w, h: 1 });

// Two rows: [A B C] on top, [   Space  ] below.
const keys = [k(0, 0, 0, 0), k(0, 1, 1, 0), k(0, 2, 2, 0), k(1, 0, 0, 1, 3)];

describe('bounds', () => {
  it('covers every key', () => {
    expect(bounds(keys)).toEqual({ w: 3, h: 2 });
    expect(bounds([])).toEqual({ w: 0, h: 0 });
  });
});

describe('neighbor', () => {
  it('moves along a row', () => {
    expect(neighbor(keys, keys[0]!, 'right')).toBe(keys[1]);
    expect(neighbor(keys, keys[2]!, 'left')).toBe(keys[1]);
  });

  it('stops at the edge', () => {
    expect(neighbor(keys, keys[0]!, 'left')).toBeUndefined();
    expect(neighbor(keys, keys[0]!, 'up')).toBeUndefined();
  });

  it('goes down to the wide key from any key above it', () => {
    for (const top of keys.slice(0, 3)) expect(neighbor(keys, top, 'down')).toBe(keys[3]);
  });

  it('goes up to the key nearest the wide key center', () => {
    expect(neighbor(keys, keys[3]!, 'up')).toBe(keys[1]);
  });
});
