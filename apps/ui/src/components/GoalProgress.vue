<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import InputNumber from 'primevue/inputnumber';

const props = defineProps<{
  /** Current value (e.g. word count). */
  current: number;
  /** Target value. `null` means no goal set. */
  goal: number | null;
  /** When true, hides the edit affordance. */
  readOnly?: boolean;
  /** When true, uses a tighter inline layout suitable for headers. */
  compact?: boolean;
}>();

const emit = defineEmits<{
  'update:goal': [value: number | null];
}>();

const { t } = useI18n();

const editing = ref(false);
const draft = ref<number | null>(null);
const inputRef = ref<InstanceType<typeof InputNumber> | null>(null);

const percent = computed(() => {
  if (props.goal == null || props.goal <= 0) return 0;
  return Math.min(100, Math.round((props.current / props.goal) * 100));
});

const barColor = computed(() => {
  if (percent.value >= 100) return 'bg-emerald-500';
  if (percent.value >= 66) return 'bg-emerald-400';
  if (percent.value >= 33) return 'bg-amber-400';
  return 'bg-primary-400';
});

function startEdit() {
  if (props.readOnly) return;
  draft.value = props.goal;
  editing.value = true;
  void nextTick(() => {
    const inst = inputRef.value as unknown as { $el?: HTMLElement };
    inst?.$el?.querySelector('input')?.focus();
  });
}

function commit() {
  const v = draft.value;
  editing.value = false;
  if (v == null || v <= 0) {
    emit('update:goal', null);
  } else {
    emit('update:goal', Math.floor(v));
  }
}

function clear() {
  editing.value = false;
  emit('update:goal', null);
}

function onKey(e: KeyboardEvent) {
  if (e.key === 'Enter') {
    e.preventDefault();
    commit();
  } else if (e.key === 'Escape') {
    e.preventDefault();
    editing.value = false;
  }
}

watch(
  () => props.goal,
  () => {
    if (!editing.value) draft.value = props.goal;
  },
  { immediate: true },
);
</script>

<template>
  <div :class="compact ? 'flex items-center gap-2' : 'flex flex-col gap-2'">
    <!-- Editing mode -->
    <template v-if="editing">
      <InputNumber
        ref="inputRef"
        v-model="draft"
        :min="0"
        :placeholder="t('goal.placeholder')"
        :show-buttons="false"
        :input-style="{ width: '6rem' }"
        size="small"
        @keydown="onKey"
      />
      <div class="flex items-center gap-1">
        <Button icon="pi pi-check" text size="small" :aria-label="t('goal.save')" @click="commit" />
        <Button
          v-if="goal != null"
          icon="pi pi-times"
          text
          severity="secondary"
          size="small"
          :aria-label="t('goal.clear')"
          @click="clear"
        />
      </div>
    </template>

    <!-- Display mode with goal set -->
    <template v-else-if="goal != null">
      <div :class="compact ? 'flex items-center gap-2 flex-1 min-w-0' : 'flex flex-col gap-1'">
        <div class="flex items-center justify-between gap-2 text-xs">
          <span class="font-mono opacity-80">{{ current }} / {{ goal }}</span>
          <span class="opacity-60">{{ percent }}%</span>
        </div>
        <div
          class="h-1.5 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden"
          :class="compact ? 'flex-1' : 'w-full'"
        >
          <div
            class="h-full transition-[width] duration-300"
            :class="barColor"
            :style="{ width: percent + '%' }"
            :aria-valuenow="percent"
            aria-valuemin="0"
            aria-valuemax="100"
            role="progressbar"
          />
        </div>
      </div>
      <Button
        v-if="!readOnly"
        icon="pi pi-pencil"
        text
        severity="secondary"
        size="small"
        :aria-label="t('goal.edit')"
        @click="startEdit"
      />
    </template>

    <!-- Display mode without a goal -->
    <template v-else>
      <Button
        v-if="!readOnly"
        :label="t('goal.set')"
        icon="pi pi-flag"
        text
        severity="secondary"
        size="small"
        @click="startEdit"
      />
      <span v-else class="text-xs opacity-50">{{ t('goal.notSet') }}</span>
    </template>
  </div>
</template>
