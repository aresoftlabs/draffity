<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from 'primevue/usetoast';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import { ipc, type AiStatus } from '@/services/ipc';
import { useCapability } from '@/composables/useCapability';
import { useAiUsageStore } from '@/stores/aiUsage';

/**
 * AI / BYOK section of Settings (extracted from the god-view, AUD-28).
 * Premium-gated (`ai_features`); manages the OpenRouter key (stored in the OS
 * keyring) and shows the token-usage meter. Self-contained.
 */
const { t } = useI18n();
const toast = useToast();
const aiEnabled = useCapability('ai_features');

const aiStatus = ref<AiStatus | null>(null);
const openrouterKey = ref('');
const savingKey = ref(false);
const aiUsage = useAiUsageStore();

async function loadAiStatus() {
  try {
    aiStatus.value = await ipc.getAiStatus();
  } catch (e) {
    aiStatus.value = null;
    // Surface the load failure rather than silently looking un-configured.
    console.error('[settings]', 'aiStatus', e);
    toast.add({ severity: 'error', summary: t('settings.loadError'), life: 5000 });
  }
}

async function onSaveOpenrouterKey() {
  const key = openrouterKey.value.trim();
  if (!key) return;
  savingKey.value = true;
  try {
    aiStatus.value = await ipc.setOpenrouterKey(key);
    openrouterKey.value = '';
    toast.add({
      severity: 'success',
      summary: t('settings.aiTitle'),
      detail: t('settings.aiKeySavedToast'),
      life: 3000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.aiTitle'),
      detail: t('settings.aiKeyError'),
      life: 5000,
    });
  } finally {
    savingKey.value = false;
  }
}

async function onClearOpenrouterKey() {
  try {
    aiStatus.value = await ipc.clearOpenrouterKey();
  } catch {
    // best-effort
  }
}

onMounted(() => {
  void loadAiStatus();
  aiUsage.rollIfNeeded();
});
</script>

<template>
  <section v-if="aiEnabled">
    <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
      {{ t('settings.aiTitle') }}
    </h2>
    <p class="text-xs opacity-60 mb-2">{{ t('settings.aiKeyHint') }}</p>
    <div
      v-if="aiStatus?.hasKey"
      class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm"
    >
      <span>
        <i class="pi pi-check-circle text-green-500 mr-1" />
        {{ t('settings.aiKeySaved') }}
      </span>
      <Button
        :label="t('settings.aiKeyClear')"
        size="small"
        text
        severity="danger"
        @click="onClearOpenrouterKey"
      />
    </div>
    <div v-else class="flex items-center gap-2">
      <InputText
        v-model="openrouterKey"
        type="password"
        class="flex-1 font-mono text-xs"
        :placeholder="t('settings.aiKeyPlaceholder')"
        :aria-label="t('settings.aiKeyLabel')"
        @keydown.enter="onSaveOpenrouterKey"
      />
      <Button
        :label="t('settings.aiKeySave')"
        size="small"
        :loading="savingKey"
        :disabled="!openrouterKey.trim()"
        @click="onSaveOpenrouterKey"
      />
    </div>
    <a
      class="text-xs underline opacity-60 hover:opacity-100 mt-2 inline-block"
      href="https://openrouter.ai/keys"
      target="_blank"
      rel="noopener noreferrer"
    >
      {{ t('settings.aiKeyGetLink') }}
    </a>

    <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-700">
      <div class="flex items-center justify-between gap-2 text-xs">
        <span class="opacity-70">
          {{ t('settings.aiUsageThisMonth', { sent: aiUsage.sent, received: aiUsage.received }) }}
        </span>
        <Button
          :label="t('settings.aiUsageReset')"
          size="small"
          text
          severity="secondary"
          @click="aiUsage.reset()"
        />
      </div>
      <a
        class="text-xs underline opacity-60 hover:opacity-100 mt-1 inline-block"
        href="https://openrouter.ai/activity"
        target="_blank"
        rel="noopener noreferrer"
      >
        {{ t('settings.aiUsageCostsLink') }}
      </a>
    </div>
  </section>
  <p v-else class="text-sm opacity-60">{{ t('capability.unavailable') }}</p>
</template>
