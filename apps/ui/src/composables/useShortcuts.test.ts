import { describe, expect, it } from 'vitest';
import { _formatCombo } from './useShortcuts';

function makeKey(opts: Partial<KeyboardEvent>): KeyboardEvent {
  return {
    key: 's',
    ctrlKey: false,
    shiftKey: false,
    altKey: false,
    metaKey: false,
    ...opts,
  } as KeyboardEvent;
}

describe('formatCombo', () => {
  it('formats plain key as lowercase', () => {
    expect(_formatCombo(makeKey({ key: 'A' }))).toBe('a');
  });

  it('prefixes ctrl when ctrlKey or metaKey present', () => {
    expect(_formatCombo(makeKey({ key: 's', ctrlKey: true }))).toBe('ctrl+s');
    expect(_formatCombo(makeKey({ key: 's', metaKey: true }))).toBe('ctrl+s');
  });

  it('orders modifiers ctrl shift alt', () => {
    expect(_formatCombo(makeKey({ key: 'b', ctrlKey: true, shiftKey: true, altKey: true }))).toBe(
      'ctrl+shift+alt+b',
    );
  });
});
