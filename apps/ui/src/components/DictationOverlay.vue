<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import type { DictationPhase } from '@/composables/useDictation';

const props = defineProps<{ phase: DictationPhase; level: number }>();
defineEmits<{ stop: []; cancel: [] }>();

const { t } = useI18n();

// Map RMS level (0..1, usually small) to a visible meter width.
const meterWidth = computed(() => `${Math.min(100, Math.round(props.level * 240))}%`);
</script>

<template>
  <div
    v-if="phase !== 'idle'"
    class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-3 rounded-full border border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-900 shadow-lg px-4 py-2"
    role="status"
    :aria-label="
      phase === 'recording' ? t('voice.dictation.recording') : t('voice.dictation.transcribing')
    "
  >
    <template v-if="phase === 'recording'">
      <span class="relative flex h-3 w-3 shrink-0">
        <span
          class="absolute inline-flex h-full w-full animate-ping rounded-full bg-red-400 opacity-75"
        />
        <span class="relative inline-flex h-3 w-3 rounded-full bg-red-500" />
      </span>
      <div class="h-2 w-24 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden">
        <div
          class="h-full bg-red-500 transition-[width] duration-75"
          :style="{ width: meterWidth }"
        />
      </div>
      <span class="text-sm">{{ t('voice.dictation.recording') }}</span>
      <Button
        :label="t('voice.dictation.stop')"
        icon="pi pi-check"
        size="small"
        @click="$emit('stop')"
      />
      <Button
        :aria-label="t('voice.dictation.cancel')"
        v-tooltip.top="t('voice.dictation.cancel')"
        icon="pi pi-times"
        size="small"
        text
        severity="secondary"
        @click="$emit('cancel')"
      />
    </template>
    <template v-else>
      <i class="pi pi-spin pi-spinner" />
      <span class="text-sm">{{ t('voice.dictation.transcribing') }}</span>
    </template>
  </div>
</template>
