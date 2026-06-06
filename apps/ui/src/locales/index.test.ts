import { afterEach, describe, expect, it } from 'vitest';
import { SUPPORTED_LOCALES, LOCALE_NAMES, detectLocale, setLocale, i18n } from './index';

describe('i18n locales', () => {
  afterEach(() => localStorage.clear());

  it('supports the 5 languages with display names', () => {
    expect([...SUPPORTED_LOCALES]).toEqual(['es', 'en', 'pt', 'fr', 'it']);
    expect(LOCALE_NAMES.pt).toBe('Português');
    expect(LOCALE_NAMES.fr).toBe('Français');
    expect(LOCALE_NAMES.it).toBe('Italiano');
  });

  it('setLocale changes the active locale and persists it', () => {
    setLocale('fr');
    expect(i18n.global.locale.value).toBe('fr');
    expect(localStorage.getItem('draffity.locale')).toBe('fr');
  });

  it('detectLocale honours a stored supported locale', () => {
    localStorage.setItem('draffity.locale', 'pt');
    expect(detectLocale()).toBe('pt');
  });

  it('detectLocale falls back to es for unsupported stored/browser values', () => {
    localStorage.setItem('draffity.locale', 'de');
    expect(detectLocale()).toBe('es');
  });
});
