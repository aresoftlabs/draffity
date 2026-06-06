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

  it('detectLocale uses the browser language when nothing is stored', () => {
    const orig = navigator.language;
    Object.defineProperty(navigator, 'language', { value: 'fr-FR', configurable: true });
    expect(detectLocale()).toBe('fr');
    Object.defineProperty(navigator, 'language', { value: orig, configurable: true });
  });

  it('detectLocale falls back to es when neither stored nor browser is supported', () => {
    localStorage.setItem('draffity.locale', 'de');
    const orig = navigator.language;
    Object.defineProperty(navigator, 'language', { value: 'de-DE', configurable: true });
    expect(detectLocale()).toBe('es');
    Object.defineProperty(navigator, 'language', { value: orig, configurable: true });
  });
});
