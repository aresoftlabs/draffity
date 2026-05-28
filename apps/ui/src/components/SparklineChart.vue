<script setup lang="ts">
import { computed } from 'vue';
import type { DailyWriting } from '@draffity/shared-types';

const props = withDefaults(
  defineProps<{
    series: DailyWriting[];
    height?: number;
    ariaLabel?: string;
  }>(),
  { height: 64 },
);

const WIDTH = 320;

const max = computed(() => {
  const m = props.series.reduce((acc, d) => Math.max(acc, d.words), 0);
  return m > 0 ? m : 1;
});

const total = computed(() => props.series.reduce((acc, d) => acc + d.words, 0));

const bars = computed(() => {
  const n = props.series.length;
  if (n === 0) return [];
  const gap = 1;
  const barWidth = Math.max(2, (WIDTH - gap * (n - 1)) / n);
  return props.series.map((d, i) => {
    const ratio = d.words / max.value;
    const h = Math.max(d.words > 0 ? 2 : 0, ratio * props.height);
    return {
      x: i * (barWidth + gap),
      y: props.height - h,
      width: barWidth,
      height: h,
      empty: d.words === 0,
      tooltip: `${d.date}: ${d.words}`,
    };
  });
});
</script>

<template>
  <figure :aria-label="ariaLabel" role="img" class="w-full">
    <svg
      :viewBox="`0 0 ${WIDTH} ${height}`"
      preserveAspectRatio="none"
      class="w-full block"
      :style="{ height: `${height}px` }"
    >
      <rect
        v-for="(b, i) in bars"
        :key="i"
        :x="b.x"
        :y="b.y"
        :width="b.width"
        :height="b.height"
        :class="
          b.empty
            ? 'fill-surface-200 dark:fill-surface-700'
            : 'fill-primary-500 dark:fill-primary-400'
        "
        :opacity="b.empty ? 0.4 : 1"
      >
        <title>{{ b.tooltip }}</title>
      </rect>
    </svg>
    <figcaption class="sr-only">{{ total }}</figcaption>
  </figure>
</template>
