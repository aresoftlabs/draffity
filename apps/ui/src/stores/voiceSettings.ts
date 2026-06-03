import { defineStore } from 'pinia';
import { ref, watch } from 'vue';

const STORAGE_KEY = 'draffity.voiceSettings';

interface VoiceSettings {
  ttsVoiceId: string | null;
  asrModelId: string | null;
  aiModelId: string | null;
  spendingLimitMonthly: number | null;
}

const DEFAULTS: VoiceSettings = {
  ttsVoiceId: null,
  asrModelId: null,
  aiModelId: null,
  spendingLimitMonthly: null,
};

function load(): VoiceSettings {
  if (typeof localStorage === 'undefined') return { ...DEFAULTS };
  const raw = localStorage.getItem(STORAGE_KEY);
  if (!raw) return { ...DEFAULTS };
  try {
    const parsed = JSON.parse(raw);
    if (typeof parsed !== 'object' || parsed === null) return { ...DEFAULTS };
    return {
      ttsVoiceId: 'ttsVoiceId' in parsed ? (parsed.ttsVoiceId ?? null) : null,
      asrModelId: 'asrModelId' in parsed ? (parsed.asrModelId ?? null) : null,
      aiModelId: 'aiModelId' in parsed ? (parsed.aiModelId ?? null) : null,
      spendingLimitMonthly:
        'spendingLimitMonthly' in parsed
          ? typeof parsed.spendingLimitMonthly === 'number'
            ? parsed.spendingLimitMonthly
            : null
          : null,
    };
  } catch {
    return { ...DEFAULTS };
  }
}

function save(state: VoiceSettings) {
  if (typeof localStorage === 'undefined') return;
  localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

export const useVoiceSettingsStore = defineStore('voiceSettings', () => {
  const initial = load();

  const ttsVoiceId = ref<string | null>(initial.ttsVoiceId);
  const asrModelId = ref<string | null>(initial.asrModelId);
  const aiModelId = ref<string | null>(initial.aiModelId);
  const spendingLimitMonthly = ref<number | null>(initial.spendingLimitMonthly);

  watch(
    [ttsVoiceId, asrModelId, aiModelId, spendingLimitMonthly],
    () => {
      save({
        ttsVoiceId: ttsVoiceId.value,
        asrModelId: asrModelId.value,
        aiModelId: aiModelId.value,
        spendingLimitMonthly: spendingLimitMonthly.value,
      });
    },
    { deep: true },
  );

  function reset() {
    ttsVoiceId.value = null;
    asrModelId.value = null;
    aiModelId.value = null;
    spendingLimitMonthly.value = null;
  }

  return {
    ttsVoiceId,
    asrModelId,
    aiModelId,
    spendingLimitMonthly,
    reset,
  };
});
