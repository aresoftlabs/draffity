<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import InputNumber from 'primevue/inputnumber';
import SparklineChart from '@/components/SparklineChart.vue';
import type { DailyWriting, WritingStats } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';

/**
 * Writing stats + daily goal section of Settings, extracted from the god-view
 * (AUD-28): streak counters, the daily word goal, and the last-30-days
 * sparkline. Self-contained â€” loads on mount.
 */
const { t } = useI18n();
const { run } = useIpcError();

const stats = ref<WritingStats | null>(null);
const dailySeries = ref<DailyWriting[]>([]);
const dailyGoal = ref<number | null>(null);

const totalWords30d = computed(() => dailySeries.value.reduce((acc, d) => acc + d.words, 0));
const activeDays30d = computed(() => dailySeries.value.filter((d) => d.sessions > 0).length);

async function onDailyGoalChange(value: number | null) {
  const goal = value && value > 0 ? Math.floor(value) : null;
  dailyGoal.value = goal;
  await run(t('settings.dailyGoalError'), () => ipc.setDailyGoal(goal));
  // Refresh the streak + series so the goal-met state reflects the new goal.
  stats.value = await ipc.getWritingStats();
  dailySeries.value = await ipc.getRecentDailyWriting(30);
}

onMounted(async () => {
  try {
    stats.value = await ipc.getWritingStats();
  } catch (e) {
    stats.value = null;
    console.error('[settings]', 'writingStats', e);
  }
  try {
    dailySeries.value = await ipc.getRecentDailyWriting(30);
  } catch (e) {
    dailySeries.value = [];
    console.error('[settings]', 'dailySeries', e);
  }
  try {
    dailyGoal.value = await ipc.getDailyGoal();
  } catch (e) {
    dailyGoal.value = null;
    console.error('[settings]', 'dailyGoal', e);
  }
});
</script>

<template>
  <section>
    <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
      {{ t('settings.writingStats') }}
    </h2>
    <dl v-if="stats" class="text-sm space-y-1">
      <div class="flex justify-between gap-2">
        <dt class="opacity-60">{{ t('settings.currentStreak') }}</dt>
        <dd class="font-mono">{{ stats.currentStreak }}</dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt class="opacity-60">{{ t('settings.longestStreak') }}</dt>
        <dd class="font-mono">{{ stats.longestStreak }}</dd>
      </div>
      <div class="flex justify-between gap-2">
        <dt class="opacity-60">{{ t('settings.goalMetStreak') }}</dt>
        <dd class="font-mono">{{ stats.goalMetStreak }}</dd>
      </div>
      <div v-if="stats.lastWritingDate" class="flex justify-between gap-2">
        <dt class="opacity-60">{{ t('settings.lastWritingDate') }}</dt>
        <dd class="font-mono">{{ stats.lastWritingDate }}</dd>
      </div>
    </dl>
    <p v-else class="text-xs opacity-60">…</p>

    <div class="mt-4 flex items-center justify-between gap-3">
      <label for="set-daily-goal" class="text-sm opacity-80">
        {{ t('settings.dailyGoal') }}
      </label>
      <InputNumber
        input-id="set-daily-goal"
        :model-value="dailyGoal"
        :min="0"
        :step="50"
        show-buttons
        :placeholder="t('settings.dailyGoalNone')"
        class="!w-40"
        @update:model-value="onDailyGoalChange"
      />
    </div>
    <p class="text-xs opacity-55 mt-1">{{ t('settings.dailyGoalHint') }}</p>

    <div class="mt-5">
      <div class="flex items-baseline justify-between mb-2 text-xs">
        <span class="opacity-70">{{ t('settings.last30Days') }}</span>
        <span class="opacity-60">
          {{ t('settings.totalWords', { count: totalWords30d }) }} ·
          {{ t('settings.activeDays', { count: activeDays30d }) }}
        </span>
      </div>
      <SparklineChart
        :series="dailySeries"
        :height="56"
        :aria-label="t('settings.last30DaysAria')"
      />
    </div>
  </section>
</template>
