<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import Popover from 'primevue/popover';
import InputNumber from 'primevue/inputnumber';
import { useWritingTimer } from '@/composables/useWritingTimer';

const { t } = useI18n();
const timer = useWritingTimer();
const settingsPopover = ref<InstanceType<typeof Popover> | null>(null);

const phaseLabel = computed(() => {
  switch (timer.phase.value) {
    case 'work':
      return t('pomodoro.work');
    case 'break':
      return t('pomodoro.break');
    case 'paused':
      return t('pomodoro.paused');
    default:
      return t('pomodoro.idle');
  }
});

const phaseClass = computed(() => {
  switch (timer.phase.value) {
    case 'work':
      return 'bg-rose-500';
    case 'break':
      return 'bg-emerald-500';
    case 'paused':
      return 'bg-amber-500';
    default:
      return 'bg-surface-300 dark:bg-surface-600';
  }
});

function toggle() {
  if (timer.running.value) timer.pause();
  else timer.start();
}

function openSettings(e: MouseEvent) {
  settingsPopover.value?.toggle(e);
}
</script>

<template>
  <div class="flex items-center gap-1">
    <button
      type="button"
      class="flex items-center gap-1.5 px-2 py-1 rounded text-xs hover:bg-surface-100 dark:hover:bg-surface-800"
      :title="phaseLabel"
      @click="toggle"
    >
      <span class="w-1.5 h-1.5 rounded-full" :class="phaseClass" />
      <span class="font-mono">{{ timer.display.value }}</span>
      <span class="opacity-60 hidden lg:inline">{{ phaseLabel }}</span>
    </button>
    <Button
      v-if="timer.phase.value !== 'idle'"
      icon="pi pi-forward"
      text
      severity="secondary"
      size="small"
      :aria-label="t('pomodoro.skip')"
      @click="timer.skip"
    />
    <Button
      v-if="timer.phase.value !== 'idle'"
      icon="pi pi-replay"
      text
      severity="secondary"
      size="small"
      :aria-label="t('pomodoro.reset')"
      @click="timer.reset"
    />
    <Button
      icon="pi pi-sliders-h"
      text
      severity="secondary"
      size="small"
      :aria-label="t('pomodoro.settings')"
      @click="openSettings"
    />

    <Popover ref="settingsPopover">
      <div class="flex flex-col gap-3 p-2 min-w-[14rem]">
        <h4 class="text-xs font-semibold uppercase tracking-wide opacity-70">
          {{ t('pomodoro.title') }}
        </h4>
        <div class="flex items-center justify-between gap-2">
          <label class="text-xs">{{ t('pomodoro.focusMin') }}</label>
          <InputNumber
            v-model="timer.focusMin.value"
            :min="1"
            :max="120"
            :show-buttons="true"
            button-layout="horizontal"
            :input-style="{ width: '3.5rem' }"
            size="small"
          />
        </div>
        <div class="flex items-center justify-between gap-2">
          <label class="text-xs">{{ t('pomodoro.breakMin') }}</label>
          <InputNumber
            v-model="timer.breakMin.value"
            :min="1"
            :max="60"
            :show-buttons="true"
            button-layout="horizontal"
            :input-style="{ width: '3.5rem' }"
            size="small"
          />
        </div>
        <p class="text-xs opacity-60">
          {{ t('pomodoro.completed', { n: timer.sessionsCompleted.value }) }}
        </p>
      </div>
    </Popover>
  </div>
</template>
