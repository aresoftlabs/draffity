<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import VoiceWaveform from './VoiceWaveform.vue';
import { formatElapsed } from '@/audio/formatElapsed';

defineProps<{
  state: 'recording' | 'transcribing';
  waveform: Uint8Array;
  elapsedMs: number;
  isSilent: boolean;
  progress: number | null;
}>();
defineEmits<{ stop: []; cancel: [] }>();

const { t } = useI18n();
</script>

<template>
  <div
    class="flex flex-col gap-1 rounded-2xl border border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-900 shadow-lg px-4 py-2"
    role="status"
    :aria-label="
      state === 'recording' ? t('voice.dictation.recording') : t('voice.dictation.transcribing')
    "
  >
    <div class="flex items-center gap-3">
      <template v-if="state === 'recording'">
        <span class="relative flex h-3 w-3 shrink-0" aria-hidden="true">
          <span
            class="absolute inline-flex h-full w-full animate-ping rounded-full bg-red-400 opacity-75"
          />
          <span class="relative inline-flex h-3 w-3 rounded-full bg-red-500" />
        </span>
        <VoiceWaveform :data="waveform" />
        <span class="text-sm tabular-nums">{{ formatElapsed(elapsedMs) }}</span>
        <Button
          data-test="rec-stop"
          :label="t('voice.dictation.stop')"
          icon="pi pi-check"
          size="small"
          @click="$emit('stop')"
        />
        <Button
          data-test="rec-cancel"
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
        <template v-if="progress != null">
          <div class="h-2 w-28 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden">
            <div
              data-test="rec-progress"
              class="h-full bg-primary-500 transition-[width] duration-150"
              :style="{ width: `${Math.min(100, Math.max(0, progress))}%` }"
            />
          </div>
          <span class="text-sm tabular-nums">{{ Math.round(progress) }}%</span>
        </template>
        <i v-else class="pi pi-spin pi-spinner" aria-hidden="true" />
        <span class="text-sm">{{ t('voice.dictation.transcribing') }}</span>
      </template>
    </div>
    <p v-if="state === 'recording' && isSilent" class="text-xs text-amber-500">
      {{ t('voice.dictation.silent') }}
    </p>
  </div>
</template>
