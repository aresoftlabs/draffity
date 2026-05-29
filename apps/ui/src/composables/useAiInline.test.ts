import { describe, expect, it } from 'vitest';
import { textToHtml } from './useAiInline';

describe('textToHtml', () => {
  it('wraps a single paragraph', () => {
    expect(textToHtml('hola mundo')).toBe('<p>hola mundo</p>');
  });

  it('splits blank-line-separated paragraphs', () => {
    expect(textToHtml('uno\n\ndos')).toBe('<p>uno</p><p>dos</p>');
  });

  it('turns single newlines into <br>', () => {
    expect(textToHtml('línea uno\nlínea dos')).toBe('<p>línea uno<br>línea dos</p>');
  });

  it('escapes HTML metacharacters to prevent injection', () => {
    expect(textToHtml('a < b & c > d')).toBe('<p>a &lt; b &amp; c &gt; d</p>');
  });

  it('trims surrounding whitespace and collapses extra blank lines', () => {
    expect(textToHtml('\n\n  texto  \n\n\n')).toBe('<p>texto</p>');
  });
});
