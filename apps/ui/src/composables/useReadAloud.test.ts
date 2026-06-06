import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { setActivePinia, createPinia } from 'pinia';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';

// Import the pure functions we test
import { resolveVoiceId, resolveTtsVoiceId } from './useReadAloud';

// ---------------------------------------------------------------------------
// Mock @/services/ipc so no Electron bridge is needed in unit tests.
// vi.mock is hoisted, so we CANNOT reference outer `const` variables inside
// the factory. Instead we use vi.fn() inline and retrieve spies via vi.mocked().
// ---------------------------------------------------------------------------
vi.mock('@/services/ipc', () => ({
  ipc: {
    getVoiceCatalog: vi.fn(),
    synthesizeSpeech: vi.fn(),
  },
}));

// Mock the settings composable so we can control resolveVoiceLanguage()
vi.mock('@/composables/dictation/settings', () => ({
  resolveVoiceLanguage: vi.fn(),
  resolveAsrModelId: vi.fn(() => null),
  resolveInputDeviceId: vi.fn(() => null),
  resolveAutoStop: vi.fn(() => false),
  resolveDictationMode: vi.fn(() => 'manual'),
}));

// Mock @/locales so we control the global locale value (default 'es')
vi.mock('@/locales', () => ({
  i18n: { global: { locale: { value: 'es' } } },
  SUPPORTED_LOCALES: ['es', 'en', 'pt', 'fr', 'it'],
  LOCALE_NAMES: { es: 'Español', en: 'English', pt: 'Português', fr: 'Français', it: 'Italiano' },
  detectLocale: vi.fn(() => 'es'),
  setLocale: vi.fn(),
}));

// ---------------------------------------------------------------------------
// Typed helpers to access the mocked modules after hoisting
// ---------------------------------------------------------------------------
import { ipc } from '@/services/ipc';
import { resolveVoiceLanguage } from '@/composables/dictation/settings';

const mockGetVoiceCatalog = vi.mocked(ipc.getVoiceCatalog);
const mockResolveVoiceLanguage = vi.mocked(resolveVoiceLanguage);

// ---------------------------------------------------------------------------
// Sample catalog fixtures
// ---------------------------------------------------------------------------
const FR_CATALOG = [
  {
    lang: 'fr',
    langName: 'Français',
    featured: true,
    voices: [
      {
        id: 'fr_FR-x-medium',
        name: 'X',
        lang: 'fr',
        quality: 'medium',
        sizeMb: 60,
        recommended: true,
        installed: true,
      },
      {
        id: 'fr_FR-y-low',
        name: 'Y',
        lang: 'fr',
        quality: 'low',
        sizeMb: 30,
        recommended: false,
        installed: true,
      },
    ],
  },
];

const FR_CATALOG_NO_INSTALLED = [
  {
    lang: 'fr',
    langName: 'Français',
    featured: true,
    voices: [
      {
        id: 'fr_FR-x-medium',
        name: 'X',
        lang: 'fr',
        quality: 'medium',
        sizeMb: 60,
        recommended: true,
        installed: false,
      },
    ],
  },
];

// ---------------------------------------------------------------------------
// Legacy: resolveVoiceId (kept for backward-compat)
// ---------------------------------------------------------------------------
describe('resolveVoiceId', () => {
  afterEach(() => {
    localStorage.clear();
  });

  it('returns ttsVoiceId from the store when set', () => {
    setActivePinia(createPinia());
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = 'es_ES-carlfm';
    expect(resolveVoiceId()).toBe('es_ES-carlfm');
  });

  it('falls back to empty string when ttsVoiceId is null', () => {
    setActivePinia(createPinia());
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = null;
    expect(resolveVoiceId()).toBe('');
  });
});

// ---------------------------------------------------------------------------
// New: resolveTtsVoiceId — language-aware resolver
// ---------------------------------------------------------------------------
describe('resolveTtsVoiceId', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    mockGetVoiceCatalog.mockReset();
    mockResolveVoiceLanguage.mockReset();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('uses ttsVoiceId directly when set, WITHOUT consulting the catalog', async () => {
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = 'es_ES-carlfm';

    const result = await resolveTtsVoiceId();

    expect(result).toBe('es_ES-carlfm');
    expect(mockGetVoiceCatalog).not.toHaveBeenCalled();
  });

  it('returns the recommended installed voice for the resolved language (fr)', async () => {
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = null;
    mockResolveVoiceLanguage.mockReturnValue('fr');
    mockGetVoiceCatalog.mockResolvedValue(FR_CATALOG);

    const result = await resolveTtsVoiceId();

    expect(mockGetVoiceCatalog).toHaveBeenCalledOnce();
    expect(result).toBe('fr_FR-x-medium'); // recommended: true
  });

  it('falls back to the first installed voice when none is marked recommended', async () => {
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = null;
    mockResolveVoiceLanguage.mockReturnValue('fr');

    const catalog = [
      {
        lang: 'fr',
        langName: 'Français',
        featured: true,
        voices: [
          {
            id: 'fr_FR-y-low',
            name: 'Y',
            lang: 'fr',
            quality: 'low',
            sizeMb: 30,
            recommended: false,
            installed: true,
          },
          {
            id: 'fr_FR-x-medium',
            name: 'X',
            lang: 'fr',
            quality: 'medium',
            sizeMb: 60,
            recommended: false,
            installed: false,
          },
        ],
      },
    ];
    mockGetVoiceCatalog.mockResolvedValue(catalog);

    const result = await resolveTtsVoiceId();

    expect(result).toBe('fr_FR-y-low');
  });

  it('returns empty string when no installed voice exists for the language', async () => {
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = null;
    mockResolveVoiceLanguage.mockReturnValue('fr');
    mockGetVoiceCatalog.mockResolvedValue(FR_CATALOG_NO_INSTALLED);

    const result = await resolveTtsVoiceId();

    expect(result).toBe('');
  });

  it('returns empty string when the catalog has no entry for the language', async () => {
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = null;
    mockResolveVoiceLanguage.mockReturnValue('fr');
    mockGetVoiceCatalog.mockResolvedValue([]);

    const result = await resolveTtsVoiceId();

    expect(result).toBe('');
  });

  it('returns empty string when getVoiceCatalog throws', async () => {
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = null;
    mockResolveVoiceLanguage.mockReturnValue('fr');
    mockGetVoiceCatalog.mockRejectedValue(new Error('IPC error'));

    const result = await resolveTtsVoiceId();

    expect(result).toBe('');
  });

  it("maps 'auto' from resolveVoiceLanguage to the global locale ('es')", async () => {
    const store = useVoiceSettingsStore();
    store.ttsVoiceId = null;
    mockResolveVoiceLanguage.mockReturnValue('auto');

    const esCatalog = [
      {
        lang: 'es',
        langName: 'Español',
        featured: true,
        voices: [
          {
            id: 'es_ES-carlfm',
            name: 'Carlos',
            lang: 'es',
            quality: 'medium',
            sizeMb: 70,
            recommended: true,
            installed: true,
          },
        ],
      },
    ];
    mockGetVoiceCatalog.mockResolvedValue(esCatalog);

    const result = await resolveTtsVoiceId();

    // i18n.global.locale.value is mocked to 'es'
    expect(result).toBe('es_ES-carlfm');
  });
});
