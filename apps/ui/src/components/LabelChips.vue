<script setup lang="ts">
import { computed } from 'vue';
import type { Label } from '@draffity/shared-types';
import { useLabelStore } from '@/stores/labels';

const props = defineProps<{
  labelIds: string[];
  /** Cap the number of chips shown; the rest collapse into a `+N` counter. */
  max?: number;
}>();

const labelStore = useLabelStore();

/** Resolve ids to labels via the store, skipping any that no longer exist
 *  (e.g. deleted while the doc list was cached). */
const resolved = computed<Label[]>(() =>
  props.labelIds.map((id) => labelStore.byId.get(id)).filter((l): l is Label => l != null),
);

const shown = computed(() => (props.max ? resolved.value.slice(0, props.max) : resolved.value));
const overflow = computed(() => Math.max(0, resolved.value.length - shown.value.length));
</script>

<template>
  <div v-if="resolved.length > 0" class="flex flex-wrap items-center gap-1">
    <span
      v-for="label in shown"
      :key="label.id"
      class="inline-flex items-center gap-1 rounded-full bg-surface-100 dark:bg-surface-800 px-1.5 py-0.5 text-[10px] leading-none max-w-[10rem]"
    >
      <span
        class="w-2 h-2 rounded-full shrink-0"
        :style="{ backgroundColor: label.color }"
        aria-hidden="true"
      />
      <span class="truncate">{{ label.name }}</span>
    </span>
    <span v-if="overflow > 0" class="text-[10px] opacity-60">{{ `+${overflow}` }}</span>
  </div>
</template>
