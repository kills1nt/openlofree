import type { KeyDef } from './types';

export const keyId = (k: Pick<KeyDef, 'row' | 'col'>) => `${k.row},${k.col}`;

/** Size of the keyboard in key units. */
export function bounds(keys: KeyDef[]): { w: number; h: number } {
  return {
    w: Math.max(0, ...keys.map((k) => k.x + k.w)),
    h: Math.max(0, ...keys.map((k) => k.y + k.h)),
  };
}

export type Dir = 'left' | 'right' | 'up' | 'down';

const center = (k: KeyDef) => ({ x: k.x + k.w / 2, y: k.y + k.h / 2 });

/** The key an arrow press should move to: nearest in that direction, straight lines beat diagonals. */
export function neighbor(keys: KeyDef[], from: KeyDef, dir: Dir): KeyDef | undefined {
  const c = center(from);
  let best: KeyDef | undefined;
  let bestScore = Infinity;
  for (const k of keys) {
    if (k === from) continue;
    const o = center(k);
    const dx = o.x - c.x;
    const dy = o.y - c.y;
    const along = dir === 'right' ? dx : dir === 'left' ? -dx : dir === 'down' ? dy : -dy;
    const across = dir === 'left' || dir === 'right' ? Math.abs(dy) : Math.abs(dx);
    if (along <= 0.25) continue;
    const score = along + across * 2.5;
    if (score < bestScore) {
      bestScore = score;
      best = k;
    }
  }
  return best;
}
