<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { useUpdater } from '@/composables/useUpdater';

const { t } = useI18n();
const u = useUpdater();
</script>

<template>
  <div
    v-if="u.bannerVisible.value"
    class="fixed bottom-4 left-1/2 -translate-x-1/2 z-50 flex items-center gap-3 rounded-xl px-4 py-3 shadow-lg bg-surface-0 dark:bg-surface-800 border border-surface-200 dark:border-surface-700 text-sm"
    role="status"
  >
    <!-- AVAILABLE -->
    <template v-if="u.status.value === 'available'">
      <span class="font-medium">{{
        t('updater.available', { version: u.availableVersion.value })
      }}</span>
      <button
        data-test="update-now"
        type="button"
        class="px-3 py-1 rounded-lg bg-primary-500 text-white hover:bg-primary-600 transition-colors"
        @click="u.downloadAndInstall()"
      >
        {{ t('updater.updateNow') }}
      </button>
      <button
        data-test="update-later"
        type="button"
        class="px-3 py-1 rounded-lg text-surface-600 dark:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-700 transition-colors"
        @click="u.dismiss()"
      >
        {{ t('updater.later') }}
      </button>
    </template>

    <!-- DOWNLOADING -->
    <template v-else-if="u.status.value === 'downloading'">
      <span>{{ t('updater.downloading', { percent: u.progress.value }) }}</span>
      <div class="w-32 h-1.5 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden">
        <div
          class="h-full bg-primary-500 transition-all"
          :style="{ width: u.progress.value + '%' }"
        />
      </div>
    </template>

    <!-- READY -->
    <template v-else-if="u.status.value === 'ready'">
      <button
        data-test="update-restart"
        type="button"
        class="px-3 py-1 rounded-lg bg-primary-500 text-white hover:bg-primary-600 transition-colors"
        @click="u.relaunchApp()"
      >
        {{ t('updater.restart') }}
      </button>
    </template>
  </div>
</template>
