<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from 'primevue/usetoast';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import Select from 'primevue/select';
import { ipc, type AiModelInfo, type AiStatus } from '@/services/ipc';
import { useAiUsageStore } from '@/stores/aiUsage';
import { useVoiceSettingsStore } from '@/stores/voiceSettings';

/**
 * AI / BYOK section of Settings (extracted from the god-view, AUD-28).
 * Manages the OpenRouter BYOK key (stored in the OS keyring) and shows
 * the token-usage meter. Self-contained.
 */
const { t } = useI18n();
const toast = useToast();

const aiStatus = ref<AiStatus | null>(null);
const openrouterKey = ref('');
const savingKey = ref(false);
const aiUsage = useAiUsageStore();
const voiceSettings = useVoiceSettingsStore();

const aiModels = ref<AiModelInfo[]>([]);
const spendingLimitInput = ref<number | null>(voiceSettings.spendingLimitMonthly);

/** Group models by provider for PrimeVue Select option groups. */
const aiModelGroups = computed(() => {
  const groups: { label: string; items: { label: string; value: string }[] }[] = [];
  const map = new Map<string, { label: string; value: string }[]>();
  for (const m of aiModels.value) {
    if (!map.has(m.provider)) map.set(m.provider, []);
    map.get(m.provider)!.push({
      label: `${m.name} — ${t('settings.aiModelTokens', { ctx: m.contextLength.toLocaleString() })} — ${t('settings.aiModelCost', { cost: m.costPer1kTokens.toFixed(4) })}`,
      value: m.id,
    });
  }
  for (const [provider, items] of map) {
    groups.push({ label: provider, items });
  }
  return groups;
});

/** Total token spend formatted as USD. */
const totalSpend = computed(() => {
  // Rough estimate: assume ~$0.005/1k tokens average
  // The actual cost varies per model, this is a ballpark
  const total = aiUsage.sent + aiUsage.received;
  return (total / 1000) * 0.005;
});

const isOverBudget = computed(() => {
  if (voiceSettings.spendingLimitMonthly === null) return false;
  return totalSpend.value > voiceSettings.spendingLimitMonthly;
});

const excessAmount = computed(() => {
  if (!isOverBudget.value || voiceSettings.spendingLimitMonthly === null) return 0;
  return totalSpend.value - voiceSettings.spendingLimitMonthly;
});

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

async function loadAiModels() {
  try {
    aiModels.value = await ipc.listAiModels();
  } catch {
    aiModels.value = [];
  }
}

function onSpendingBlur() {
  const val = spendingLimitInput.value;
  if (val === null || val === undefined) {
    voiceSettings.spendingLimitMonthly = null;
  } else if (val >= 0) {
    voiceSettings.spendingLimitMonthly = val;
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
  void loadAiModels();
  aiUsage.rollIfNeeded();
});
</script>

<template>
  <section>
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

    <!-- AI Model Selector -->
    <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-700">
      <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
        {{ t('settings.aiModelSelect') }}
      </h3>
      <Select
        v-if="aiModels.length > 0"
        v-model="voiceSettings.aiModelId"
        :options="aiModelGroups"
        option-label="label"
        option-value="value"
        option-group-label="label"
        option-group-children="items"
        :placeholder="t('settings.aiModelUnset')"
        class="w-full"
        :aria-label="t('settings.aiModelSelect')"
      />
      <p v-else class="text-xs opacity-60 italic">
        {{ t('settings.aiModelNone') }}
      </p>
      <a
        class="text-xs underline opacity-60 hover:opacity-100 mt-1 inline-block"
        href="https://openrouter.ai/models"
        target="_blank"
        rel="noopener noreferrer"
      >
        {{ t('settings.aiModelAllLink') }}
      </a>
    </div>

    <!-- Spending Limit -->
    <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-700">
      <label class="text-xs font-semibold uppercase tracking-wide opacity-60 block mb-2">
        {{ t('settings.aiSpendingLimit') }}
      </label>
      <InputNumber
        v-model="spendingLimitInput"
        :min="0"
        :step="0.5"
        :placeholder="t('settings.aiSpendingLimit')"
        class="w-40"
        :aria-label="t('settings.aiSpendingLimit')"
        @blur="onSpendingBlur"
      />
    </div>

    <!-- Token usage + Budget display -->
    <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-700">
      <div class="flex items-center justify-between gap-2 text-xs">
        <span class="opacity-70">
          {{ t('settings.aiUsageLabel', { sent: aiUsage.sent, received: aiUsage.received }) }}
        </span>
        <Button
          :label="t('settings.aiUsageReset')"
          size="small"
          text
          severity="secondary"
          @click="aiUsage.reset()"
        />
      </div>
      <div v-if="voiceSettings.spendingLimitMonthly !== null" class="text-xs mt-1">
        <span v-if="!isOverBudget" class="text-green-600 dark:text-green-400">
          {{
            t('settings.aiBudgetRemaining', {
              used: totalSpend.toFixed(2),
              limit: voiceSettings.spendingLimitMonthly.toFixed(2),
            })
          }}
        </span>
        <span v-else class="text-red-600 dark:text-red-400 font-semibold">
          {{ t('settings.aiBudgetOver', { excess: excessAmount.toFixed(2) }) }}
        </span>
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
</template>
