import { describe, expect, it } from 'vitest';
import { useCommandPalette } from './useCommandPalette';

describe('useCommandPalette', () => {
  it('abre, cierra y togglea la visibilidad módulo-compartida', () => {
    const { visible, open, close } = useCommandPalette();
    close();
    expect(visible.value).toBe(false);
    open();
    expect(visible.value).toBe(true);
    close();
    expect(visible.value).toBe(false);
  });

  it('comparte estado entre invocaciones (mismo singleton)', () => {
    const a = useCommandPalette();
    const b = useCommandPalette();
    a.open();
    expect(b.visible.value).toBe(true);
    b.close();
    expect(a.visible.value).toBe(false);
  });
});
