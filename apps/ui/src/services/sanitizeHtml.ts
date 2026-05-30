import DOMPurify from 'dompurify';

/**
 * HTML sanitizers for the two places that render stored content with `v-html`.
 *
 * Document HTML is user-authored but can also arrive via import (markdown/docx),
 * so it must be treated as untrusted: an imported `<script>`/`onerror` payload
 * would otherwise execute in the webview. We sanitize with an allowlist instead
 * of escaping so legitimate formatting survives.
 */

/** Tags the editor emits (StarterKit + underline + table) plus links and the
 *  FTS `<mark>` highlight. Formatting only — no embeds, scripts or handlers. */
const DOCUMENT_TAGS = [
  'p',
  'br',
  'hr',
  'strong',
  'b',
  'em',
  'i',
  'u',
  's',
  'del',
  'h1',
  'h2',
  'h3',
  'h4',
  'h5',
  'h6',
  'blockquote',
  'ul',
  'ol',
  'li',
  'a',
  'mark',
  'code',
  'pre',
  'span',
  'sub',
  'sup',
  'table',
  'thead',
  'tbody',
  'tr',
  'th',
  'td',
];

const DOCUMENT_ATTR = ['href', 'class', 'colspan', 'rowspan', 'start', 'data-type'];

/** Sanitize rendered document HTML, preserving rich formatting. */
export function sanitizeDocumentHtml(html: string): string {
  return DOMPurify.sanitize(html, {
    ALLOWED_TAGS: DOCUMENT_TAGS,
    ALLOWED_ATTR: DOCUMENT_ATTR,
  });
}

/** Sanitize an FTS5 snippet: only the `<mark>` highlight survives; the
 *  surrounding document text is reduced to inert text. */
export function sanitizeSearchExcerpt(excerpt: string): string {
  return DOMPurify.sanitize(excerpt, {
    ALLOWED_TAGS: ['mark'],
    ALLOWED_ATTR: [],
  });
}
