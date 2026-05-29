<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import DatePicker from 'primevue/datepicker';
import Button from 'primevue/button';
import { computePacemaker } from '@/composables/usePacemaker';

const props = defineProps<{
  goalWords: number | null;
  currentWords: number;
  deadline: number | null;
  /** Words written this session — drives the on-track/behind status. */
  wordsThisSession: number;
  readOnly?: boolean;
}>();

const emit = defineEmits<{ 'update:deadline': [number | null] }>();

const { t, n } = useI18n();

const pace = computed(() =>
  computePacemaker({
    goal: props.goalWords,
    current: props.currentWords,
    deadline: props.deadline,
    wordsThisSession: props.wordsThisSession,
  }),
);

const deadlineDate = computed<Date | null>(() =>
  props.deadline != null ? new Date(props.deadline) : null,
);

function onPick(v: Date | Date[] | (Date | null)[] | null | undefined) {
  const d = Array.isArray(v) ? v[0] : v;
  emit('update:deadline', d instanceof Date ? d.getTime() : null);
}

/** Tailwind tint per status — green on track, amber close, rose behind/overdue. */
const chipClass = computed(() => {
  switch (pace.value.status) {
    case 'done':
    case 'ontrack':
      return 'bg-emerald-100 text-emerald-800 dark:bg-emerald-900/40 dark:text-emerald-200';
    case 'close':
      return 'bg-amber-100 text-amber-800 dark:bg-amber-900/40 dark:text-amber-200';
    default:
      return 'bg-rose-100 text-rose-800 dark:bg-rose-900/40 dark:text-rose-200';
  }
});

const chipLabel = computed(() => {
  const p = pace.value;
  if (p.status === 'done') return t('pacemaker.done');
  if (p.status === 'overdue') return t('pacemaker.overdue');
  return t('pacemaker.perDay', { n: n(p.wordsPerDay) });
});

const chipTitle = computed(() =>
  t('pacemaker.tooltip', { days: pace.value.daysRemaining, words: n(pace.value.wordsRemaining) }),
);
</script>

<template>
  <div class="flex items-center gap-1">
    <span
      v-if="pace.active"
      class="rounded-full px-2 py-0.5 text-[11px] font-medium whitespace-nowrap"
      :class="chipClass"
      :title="chipTitle"
    >
      {{ chipLabel }}
    </span>
    <DatePicker
      :model-value="deadlineDate"
      :disabled="readOnly"
      size="small"
      date-format="yy-mm-dd"
      :placeholder="t('pacemaker.setDeadline')"
      show-icon
      icon-display="input"
      class="!w-[9rem] !text-xs"
      :aria-label="t('pacemaker.setDeadline')"
      @update:model-value="onPick"
    />
    <Button
      v-if="deadline != null && !readOnly"
      icon="pi pi-times"
      text
      size="small"
      :pt="{ root: { class: '!w-6 !h-6 !p-0' } }"
      :aria-label="t('pacemaker.clear')"
      @click="emit('update:deadline', null)"
    />
  </div>
</template>
