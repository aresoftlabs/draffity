import { useVoiceSettingsStore } from '@/stores/voiceSettings';

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
