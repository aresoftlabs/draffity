<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import SaveIndicator from '@/components/SaveIndicator.vue';
import GoalProgress from '@/components/GoalProgress.vue';
import PacemakerWidget from '@/components/PacemakerWidget.vue';
import PomodoroWidget from '@/components/PomodoroWidget.vue';
import type { SaveState } from '@/stores/document';

/** Barra de estado inferior presentacional. Todas las entradas por props; emite
 *  las ediciones de objetivo/deadline para que ProjectView aplique la lógica. */
defineProps<{
  totalWordCount: number;
  saveState: SaveState;
  lastSavedAt: number | null;
  projectGoal: number | null;
  projectDeadline: number | null;
  sessionWords: number;
  sessionGoal: number | null;
  readOnly: boolean;
  voiceTts?: boolean;
  voiceDictation?: boolean;
  readAloudActive?: boolean;
  dictationActive?: boolean;
}>();

const emit = defineEmits<{
  'update:projectGoal': [number | null];
  'update:projectDeadline': [number | null];
  'update:sessionGoal': [number | null];
  toggleReadAloud: [];
  toggleDictation: [];
}>();

const { t } = useI18n();
</script>

<template>
  <footer
    class="h-9 shrink-0 flex items-center gap-4 px-4 border-t border-surface-200 dark:border-surface-700 bg-surface-50/90 dark:bg-surface-900/90 backdrop-blur text-xs text-surface-600 dark:text-surface-300"
  >
    <SaveIndicator :state="saveState" :last-saved-at="lastSavedAt" />

    <button
      v-if="voiceTts"
      type="button"
      data-test="read-aloud"
      class="w-6 h-6 rounded-md flex items-center justify-center transition-colors"
      :class="
        readAloudActive
          ? 'text-primary-500'
          : 'text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800'
      "
      :title="t('voice.readAloud.button')"
      :aria-label="t('voice.readAloud.button')"
      :aria-pressed="readAloudActive"
      @click="emit('toggleReadAloud')"
    >
      <i class="pi pi-volume-up text-xs" />
    </button>
    <button
      v-if="voiceDictation"
      type="button"
      data-test="dictation"
      class="w-6 h-6 rounded-md flex items-center justify-center transition-colors"
      :class="
        dictationActive
          ? 'text-red-500'
          : 'text-surface-500 hover:bg-surface-100 dark:hover:bg-surface-800'
      "
      :disabled="readOnly"
      :title="t('voice.dictation.button')"
      :aria-label="t('voice.dictation.button')"
      :aria-pressed="dictationActive"
      @click="emit('toggleDictation')"
    >
      <i class="pi pi-microphone text-xs" />
    </button>

    <span class="font-mono tabular-nums">
      {{ totalWordCount.toLocaleString() }}
      <span class="opacity-60">{{ t('statusBar.words') }}</span>
    </span>

    <span class="flex-1" />

    <div class="hidden md:flex items-center gap-2" :title="t('statusBar.session')">
      <span class="opacity-60">{{ t('statusBar.session') }}</span>
      <GoalProgress
        :current="sessionWords"
        :goal="sessionGoal"
        compact
        @update:goal="(v: number | null) => emit('update:sessionGoal', v)"
      />
    </div>

    <div class="hidden lg:flex items-center gap-2" :title="t('statusBar.goal')">
      <span class="opacity-60">{{ t('statusBar.goal') }}</span>
      <GoalProgress
        :current="totalWordCount"
        :goal="projectGoal"
        :read-only="readOnly"
        compact
        @update:goal="(v: number | null) => emit('update:projectGoal', v)"
      />
    </div>

    <PacemakerWidget
      :goal-words="projectGoal"
      :current-words="totalWordCount"
      :deadline="projectDeadline"
      :words-this-session="sessionWords"
      :read-only="readOnly"
      @update:deadline="(v: number | null) => emit('update:projectDeadline', v)"
    />

    <PomodoroWidget />
  </footer>
</template>
