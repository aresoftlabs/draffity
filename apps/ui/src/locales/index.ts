import { createI18n } from 'vue-i18n';
import es from './es.json';
import en from './en.json';
import pt from './pt.json';
import fr from './fr.json';
import it from './it.json';

const STORAGE_KEY = 'draffity.locale';

export const SUPPORTED_LOCALES = ['es', 'en', 'pt', 'fr', 'it'] as const;
export type Locale = (typeof SUPPORTED_LOCALES)[number];

export const LOCALE_NAMES: Record<Locale, string> = {
  es: 'Español',
  en: 'English',
  pt: 'Português',
  fr: 'Français',
  it: 'Italiano',
};

function isLocale(v: string | null | undefined): v is Locale {
  return !!v && (SUPPORTED_LOCALES as readonly string[]).includes(v);
}

export function detectLocale(): Locale {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (isLocale(stored)) return stored;
  return 'es';
}

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: detectLocale(),
  fallbackLocale: 'es',
  messages: { es, en, pt, fr, it },
});

export function setLocale(locale: Locale) {
  i18n.global.locale.value = locale;
  localStorage.setItem(STORAGE_KEY, locale);
}
