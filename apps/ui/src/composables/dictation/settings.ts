import { useVoiceSettingsStore } from '@/stores/voiceSettings';
import { i18n, type Locale } from '@/locales';

/** ASR model id from settings, or null (backend uses its default). */
export function resolveAsrModelId(): string | null {
  try {
    return useVoiceSettingsStore().asrModelId;
  } catch {
    return null;
  }
}

/** Preferred microphone `deviceId` from settings (null = default). */
export function resolveInputDeviceId(): string | null {
  try {
    return useVoiceSettingsStore().inputDeviceId;
  } catch {
    return null;
  }
}

/** Whether to auto-stop after sustained silence. */
export function resolveAutoStop(): boolean {
  try {
    return useVoiceSettingsStore().autoStopOnSilence;
  } catch {
    return false;
  }
}

/** Modo de dictado activo. Default 'manual'. Punto único para futuras opciones. */
export function resolveDictationMode(): 'manual' | 'streaming' {
  try {
    return useVoiceSettingsStore().dictationMode;
  } catch {
    return 'manual';
  }
}

/** Idioma para la voz: override de voz si está, si no el idioma global. */
export function resolveVoiceLanguage(): Locale | 'auto' {
  try {
    const override = useVoiceSettingsStore().voiceLanguage;
    if (override) return override;
  } catch {
    /* sin pinia: cae al locale global */
  }
  return i18n.global.locale.value as Locale;
}
