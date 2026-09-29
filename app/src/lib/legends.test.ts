import { describe, expect, it } from 'vitest';
import { catalog, legend } from './legends';

describe('legend', () => {
  it('matches the Rust labels for codes read from the real keyboard', () => {
    expect(legend(0x29, true)).toBe('Esc');
    expect(legend(0x00a8, true)).toBe('Mute');
    expect(legend(0x00be, true)).toBe('Bright-');
    expect(legend(0x7e0b, true)).toBe('Custom 11');
    expect(legend(0x5221, true)).toBe('MO(1)');
    expect(legend(0x7793, true)).toBe('BT1');
    expect(legend(0x1234, true)).toBe('0x1234');
  });

  it('uses Option and Cmd on Mac and Alt and Win elsewhere', () => {
    expect(legend(0xe2, true)).toBe('Opt');
    expect(legend(0xe3, true)).toBe('Cmd');
    expect(legend(0xe2, false)).toBe('Alt');
    expect(legend(0xe7, false)).toBe('R Win');
  });
});

describe('catalog', () => {
  it('has unique codes and labels that match legend', () => {
    const seen = new Set<number>();
    for (const g of catalog(true, 6)) {
      expect(g.items.length).toBeGreaterThan(0);
      for (const i of g.items) {
        expect(i.label).toBe(legend(i.code, true));
        expect(seen.has(i.code)).toBe(false);
        seen.add(i.code);
      }
    }
  });

  it('offers three layer keys per layer', () => {
    const layers = (n: number) => catalog(true, n).find((g) => g.name === 'Layers')!.items.length;
    expect(layers(1)).toBe(3);
    expect(layers(6)).toBe(18);
  });
});
