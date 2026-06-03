import { beforeEach, describe, expect, it } from 'vitest';
import { nextTick } from 'vue';
import { setActivePinia, createPinia } from 'pinia';

import { useVoiceSettingsStore } from './voiceSettings';

const STORAGE_KEY = 'draffity.voiceSettings';

function setStorage(json: unknown) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(json));
}

function clearStorage() {
  localStorage.removeItem(STORAGE_KEY);
}

describe('useVoiceSettingsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    clearStorage();
  });

  describe('ttsVoiceId', () => {
    it('defaults to null when nothing is stored', () => {
      const store = useVoiceSettingsStore();
      expect(store.ttsVoiceId).toBeNull();
    });

    it('reads persisted value from localStorage', () => {
      setStorage({
        ttsVoiceId: 'es_ES-carlfm',
        asrModelId: null,
        aiModelId: null,
        spendingLimitMonthly: null,
      });
      const store = useVoiceSettingsStore();
      expect(store.ttsVoiceId).toBe('es_ES-carlfm');
    });

    it('persists to localStorage when set', async () => {
      const store = useVoiceSettingsStore();
      store.ttsVoiceId = 'en_US-amy-medium';
      await nextTick();
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
      expect(saved.ttsVoiceId).toBe('en_US-amy-medium');
    });
  });

  describe('asrModelId', () => {
    it('defaults to null', () => {
      const store = useVoiceSettingsStore();
      expect(store.asrModelId).toBeNull();
    });

    it('reads persisted value', () => {
      setStorage({
        ttsVoiceId: null,
        asrModelId: 'base.en',
        aiModelId: null,
        spendingLimitMonthly: null,
      });
      const store = useVoiceSettingsStore();
      expect(store.asrModelId).toBe('base.en');
    });

    it('persists when set', async () => {
      const store = useVoiceSettingsStore();
      store.asrModelId = 'large-v3';
      await nextTick();
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
      expect(saved.asrModelId).toBe('large-v3');
    });
  });

  describe('aiModelId', () => {
    it('defaults to null', () => {
      const store = useVoiceSettingsStore();
      expect(store.aiModelId).toBeNull();
    });

    it('reads persisted value', () => {
      setStorage({
        ttsVoiceId: null,
        asrModelId: null,
        aiModelId: 'gpt-4o',
        spendingLimitMonthly: null,
      });
      const store = useVoiceSettingsStore();
      expect(store.aiModelId).toBe('gpt-4o');
    });

    it('persists when set', async () => {
      const store = useVoiceSettingsStore();
      store.aiModelId = 'claude-3-haiku';
      await nextTick();
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
      expect(saved.aiModelId).toBe('claude-3-haiku');
    });
  });

  describe('spendingLimitMonthly', () => {
    it('defaults to null', () => {
      const store = useVoiceSettingsStore();
      expect(store.spendingLimitMonthly).toBeNull();
    });

    it('reads persisted value', () => {
      setStorage({ ttsVoiceId: null, asrModelId: null, aiModelId: null, spendingLimitMonthly: 20 });
      const store = useVoiceSettingsStore();
      expect(store.spendingLimitMonthly).toBe(20);
    });

    it('persists when set', async () => {
      const store = useVoiceSettingsStore();
      store.spendingLimitMonthly = 15;
      await nextTick();
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY)!);
      expect(saved.spendingLimitMonthly).toBe(15);
    });
  });

  describe('corrupt localStorage', () => {
    it('resets to defaults when JSON is invalid', () => {
      localStorage.setItem(STORAGE_KEY, 'not valid json!!!');
      const store = useVoiceSettingsStore();
      expect(store.ttsVoiceId).toBeNull();
      expect(store.asrModelId).toBeNull();
      expect(store.aiModelId).toBeNull();
      expect(store.spendingLimitMonthly).toBeNull();
    });

    it('resets to defaults when stored value is not an object', () => {
      localStorage.setItem(STORAGE_KEY, '"just a string"');
      const store = useVoiceSettingsStore();
      expect(store.ttsVoiceId).toBeNull();
    });
  });

  describe('reset', () => {
    it('clears all fields back to null', () => {
      setStorage({
        ttsVoiceId: 'en_US-amy',
        asrModelId: 'base',
        aiModelId: 'gpt-4o',
        spendingLimitMonthly: 10,
      });
      const store = useVoiceSettingsStore();
      store.reset();
      expect(store.ttsVoiceId).toBeNull();
      expect(store.asrModelId).toBeNull();
      expect(store.aiModelId).toBeNull();
      expect(store.spendingLimitMonthly).toBeNull();
    });
  });
});
