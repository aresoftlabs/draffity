import { createI18n } from 'vue-i18n';
import es from './es.json';
import en from './en.json';

const STORAGE_KEY = 'draffity.locale';

function detectLocale(): 'es' | 'en' {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === 'es' || stored === 'en') return stored;
  const browser = navigator.language?.toLowerCase() ?? 'es';
  return browser.startsWith('en') ? 'en' : 'es';
}

export const i18n = createI18n({
  legacy: false,
  globalInjection: true,
  locale: detectLocale(),
  fallbackLocale: 'es',
  messages: { es, en },
});

export function setLocale(locale: 'es' | 'en') {
  i18n.global.locale.value = locale;
  localStorage.setItem(STORAGE_KEY, locale);
}
