import { describe, expect, it } from 'vitest';
import { changeSummary, cloneLayers, isChanged, keyIndex, withKey } from './draft';

const saved = [
  [1, 2, 3],
  [4, 5, 6],
];

describe('draft helpers', () => {
  it('indexes row-major', () => {
    expect(keyIndex(15, 2, 3)).toBe(33);
  });

  it('withKey returns a changed copy and leaves the original alone', () => {
    const next = withKey(saved, 1, 2, 99);
    expect(next[1]).toEqual([4, 5, 99]);
    expect(saved[1]).toEqual([4, 5, 6]);
    expect(next[0]).toBe(saved[0]); // untouched layers are shared, not copied
  });

  it('cloneLayers makes independent rows', () => {
    const copy = cloneLayers(saved);
    copy[0]![0] = 7;
    expect(saved[0]![0]).toBe(1);
  });

  it('counts changes by key and by layer', () => {
    expect(changeSummary(saved, saved)).toEqual({ keys: 0, layers: 0 });
    const d = withKey(withKey(withKey(saved, 0, 0, 9), 0, 1, 9), 1, 2, 9);
    expect(changeSummary(saved, d)).toEqual({ keys: 3, layers: 2 });
  });

  it('a key set back to its saved value is not a change', () => {
    const d = withKey(withKey(saved, 0, 0, 9), 0, 0, 1);
    expect(isChanged(saved, d, 0, 0)).toBe(false);
    expect(changeSummary(saved, d).keys).toBe(0);
  });
});
