<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import Select from 'primevue/select';
import type { ReadAloudPhase } from '@/composables/useReadAloud';

const props = defineProps<{ phase: ReadAloudPhase; speed: number; speeds: readonly number[] }>();
const emit = defineEmits<{
  pause: [];
  resume: [];
  stop: [];
  skip: [];
  'update:speed': [number];
}>();

const { t } = useI18n();

function togglePlay() {
  if (props.phase === 'playing') emit('pause');
  else emit('resume');
}
</script>

<template>
  <div
    v-if="phase !== 'idle'"
    class="fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center gap-2 rounded-full border border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-900 shadow-lg px-3 py-2"
    role="status"
    :aria-label="t('voice.readAloud.title')"
  >
    <i class="pi pi-volume-up text-primary-500" />
    <Button
      :icon="phase === 'playing' ? 'pi pi-pause' : 'pi pi-play'"
      size="small"
      text
      :aria-label="phase === 'playing' ? t('voice.readAloud.pause') : t('voice.readAloud.resume')"
      @click="togglePlay"
    />
    <Button
      icon="pi pi-step-forward"
      size="small"
      text
      :aria-label="t('voice.readAloud.skip')"
      @click="emit('skip')"
    />
    <Select
      :model-value="speed"
      :options="[...speeds]"
      class="w-20"
      :aria-label="t('voice.readAloud.speed')"
      @update:model-value="(v: number) => emit('update:speed', v)"
    >
      <template #value="{ value }">{{ `${value}×` }}</template>
      <template #option="{ option }">{{ `${option}×` }}</template>
    </Select>
    <Button
      icon="pi pi-times"
      size="small"
      text
      severity="secondary"
      :aria-label="t('voice.readAloud.stop')"
      @click="emit('stop')"
    />
  </div>
</template>
