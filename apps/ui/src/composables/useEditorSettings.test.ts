import { describe, expect, it } from 'vitest';
import { builtInFamily } from './useEditorSettings';

describe('builtInFamily', () => {
  it('serif preset usa Source Serif 4 como fuente de lectura', () => {
    expect(builtInFamily('serif')).toContain('Source Serif 4');
  });

  it('mantiene Lora como respaldo para no regresionar usuarios previos', () => {
    expect(builtInFamily('serif')).toContain('Lora');
  });

  it('sans y mono permanecen estables', () => {
    expect(builtInFamily('sans')).toContain('Inter');
    expect(builtInFamily('mono')).toContain('JetBrains Mono');
  });
});
