import { describe, expect, it } from 'vitest';
import { keycodeForEvent } from './eventcodes';

describe('keycodeForEvent', () => {
  it('maps letters, digits and function keys to QMK basic codes', () => {
    expect(keycodeForEvent('KeyA')).toBe(0x04);
    expect(keycodeForEvent('KeyZ')).toBe(0x1d);
    expect(keycodeForEvent('Digit1')).toBe(0x1e);
    expect(keycodeForEvent('Digit9')).toBe(0x26);
    expect(keycodeForEvent('Digit0')).toBe(0x27);
    expect(keycodeForEvent('F1')).toBe(0x3a);
    expect(keycodeForEvent('F12')).toBe(0x45);
  });

  it('maps editing, arrows and modifiers', () => {
    expect(keycodeForEvent('Space')).toBe(0x2c);
    expect(keycodeForEvent('Escape')).toBe(0x29);
    expect(keycodeForEvent('ArrowUp')).toBe(0x52);
    expect(keycodeForEvent('ShiftRight')).toBe(0xe5);
    expect(keycodeForEvent('MetaLeft')).toBe(0xe3);
  });

  it('ignores keys the keyboard cannot report', () => {
    expect(keycodeForEvent('AudioVolumeUp')).toBeUndefined();
    expect(keycodeForEvent('')).toBeUndefined();
  });
});
