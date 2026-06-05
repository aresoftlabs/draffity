<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import { getVersion } from '@tauri-apps/api/app';
import { useUpdater } from '@/composables/useUpdater';

const { t } = useI18n();
const u = useUpdater();
const currentVersion = ref('');

onMounted(async () => {
  try {
    currentVersion.value = await getVersion();
  } catch {
    // best-effort: outside Tauri (e.g. browser dev) getVersion is unavailable.
  }
});
</script>

<template>
  <section class="flex items-center justify-between gap-4">
    <div>
      <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70">
        {{ t('updater.section') }}
      </h2>
      <p class="text-xs opacity-60 mt-1">
        {{ t('updater.currentVersion') }}: <span class="font-mono">{{ currentVersion }}</span>
      </p>
      <p v-if="u.status.value === 'uptodate'" class="text-xs opacity-60 mt-1">
        {{ t('updater.upToDate') }}
      </p>
      <p v-else-if="u.status.value === 'available'" class="text-xs opacity-80 mt-1">
        {{ t('updater.available', { version: u.availableVersion.value }) }}
      </p>
      <p v-else-if="u.status.value === 'error'" class="text-xs text-red-500 mt-1">
        {{ t('updater.error') }}
      </p>
    </div>
    <Button
      :label="u.status.value === 'checking' ? t('updater.checking') : t('updater.checkButton')"
      icon="pi pi-sync"
      size="small"
      severity="secondary"
      :loading="u.status.value === 'checking'"
      @click="u.check({ silent: false })"
    />
  </section>
</template>
