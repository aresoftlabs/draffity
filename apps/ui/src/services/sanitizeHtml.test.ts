// @vitest-environment jsdom
// jsdom (not the project-default happy-dom) is the canonical DOM for testing
// DOMPurify: happy-dom mishandles literal <script> nodes during sanitization.
import { describe, expect, it } from 'vitest';
import { sanitizeDocumentHtml, sanitizeSearchExcerpt } from './sanitizeHtml';

describe('sanitizeDocumentHtml', () => {
  it('preserves rich formatting produced by the editor', () => {
    const html = '<h2>Title</h2><p><strong>bold</strong> and <em>italic</em></p>';
    expect(sanitizeDocumentHtml(html)).toBe(html);
  });

  it('keeps lists, blockquotes and tables', () => {
    const html =
      '<ul><li>one</li></ul><blockquote>q</blockquote><table><tbody><tr><td>c</td></tr></tbody></table>';
    const out = sanitizeDocumentHtml(html);
    expect(out).toContain('<li>one</li>');
    expect(out).toContain('<blockquote>q</blockquote>');
    expect(out).toContain('<td>c</td>');
  });

  it('strips <script> tags and their content', () => {
    const out = sanitizeDocumentHtml('<p>safe</p><script>alert(1)</script>');
    expect(out).toContain('<p>safe</p>');
    expect(out).not.toContain('script');
    expect(out).not.toContain('alert');
  });

  it('strips inline event handlers', () => {
    const out = sanitizeDocumentHtml('<p onclick="alert(1)">click</p>');
    expect(out).toContain('click');
    expect(out).not.toContain('onclick');
    expect(out).not.toContain('alert');
  });

  it('removes onerror image payloads', () => {
    const out = sanitizeDocumentHtml('<img src=x onerror="alert(1)">');
    expect(out).not.toContain('onerror');
    expect(out).not.toContain('alert');
  });

  it('neutralizes javascript: URLs on links', () => {
    const out = sanitizeDocumentHtml('<a href="javascript:alert(1)">x</a>');
    expect(out).not.toContain('javascript:');
    expect(out).not.toContain('alert');
  });

  it('drops iframes', () => {
    const out = sanitizeDocumentHtml('<p>a</p><iframe src="https://evil.test"></iframe>');
    expect(out).toContain('<p>a</p>');
    expect(out).not.toContain('iframe');
  });
});

describe('sanitizeSearchExcerpt', () => {
  it('keeps the <mark> highlight wrappers and plain text', () => {
    const out = sanitizeSearchExcerpt('the <mark>quick</mark> brown fox');
    expect(out).toBe('the <mark>quick</mark> brown fox');
  });

  it('strips any tag other than <mark>', () => {
    const out = sanitizeSearchExcerpt('<p>around</p> <mark>hit</mark>');
    expect(out).toContain('<mark>hit</mark>');
    expect(out).toContain('around');
    expect(out).not.toContain('<p>');
  });

  it('removes a <script> payload smuggled in document content', () => {
    const out = sanitizeSearchExcerpt('text <script>alert(1)</script> <mark>hit</mark>');
    expect(out).toContain('<mark>hit</mark>');
    expect(out).not.toContain('script');
    expect(out).not.toContain('alert');
  });

  it('removes onerror image payloads in the excerpt', () => {
    const out = sanitizeSearchExcerpt('<img src=x onerror="alert(1)"> <mark>hit</mark>');
    expect(out).not.toContain('onerror');
    expect(out).not.toContain('alert');
  });
});
