import { describe, expect, it } from 'vitest';
import { builtInFamily, sanitizeUserCss } from './useEditorSettings';

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

describe('sanitizeUserCss', () => {
  it('strips </style> tags', () => {
    expect(sanitizeUserCss('before</style>after')).toBe('beforeafter');
  });

  it('strips @import rules', () => {
    const input = '@import url("evil.css");\nbody { color: red; }';
    expect(sanitizeUserCss(input)).not.toContain('@import');
  });

  it('neutralizes url() calls', () => {
    expect(sanitizeUserCss('background: url("evil.png")')).toContain('/*url(*/');
  });

  it('passes through clean CSS unchanged', () => {
    const input = 'body { color: red; }';
    expect(sanitizeUserCss(input)).toBe(input);
  });

  it('handles case-insensitive </style>', () => {
    expect(sanitizeUserCss('</STYLE>')).toBe('');
    expect(sanitizeUserCss('</Style>')).toBe('');
  });

  it('handles empty string', () => {
    expect(sanitizeUserCss('')).toBe('');
  });
});
