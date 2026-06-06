<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import {
  ipc,
  type VoiceStatus,
  type VoiceDownloadProgress,
  type DiskUsageEntry,
} from '@/services/ipc';
import DictationSettings from './DictationSettings.vue';
import ReadAloudSettings from './ReadAloudSettings.vue';

/**
 * Voice section of Settings (Épica H). Thin container: owns the shared voice
 * status, the single download-progress listener and the disk-usage footer,
 * and delegates the two blocks ("Dictado" / "Lectura en voz alta") to focused
 * child components.
 */
const { t } = useI18n();

const voiceStatus = ref<VoiceStatus | null>(null);
const diskUsage = ref<DiskUsageEntry[]>([]);
const downloadPct = ref<Record<string, number>>({});
let unlistenVoiceProgress: UnlistenFn | null = null;

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB'];
  const i = Math.min(Math.floor(Math.log2(bytes) / 10), units.length - 1);
  const val = bytes / Math.pow(1024, i);
  return `${val.toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
}

const totalDiskBytes = computed(() => diskUsage.value.reduce((sum, d) => sum + d.bytes, 0));
const installedCount = computed(() => diskUsage.value.length);

async function loadShared() {
  try {
    voiceStatus.value = await ipc.getVoiceStatus();
    diskUsage.value = await ipc.getDiskUsage();
  } catch (e) {
    voiceStatus.value = null;
    diskUsage.value = [];
    console.error('[settings]', 'voice', e);
  }
}

onMounted(async () => {
  await loadShared();
  unlistenVoiceProgress = await listen<VoiceDownloadProgress>('voice:download:progress', (e) => {
    const p = e.payload;
    if (p.total && p.total > 0) {
      downloadPct.value = {
        ...downloadPct.value,
        [p.modelId]: Math.round((p.downloaded / p.total) * 100),
      };
    }
  });
});

onBeforeUnmount(() => {
  unlistenVoiceProgress?.();
});
</script>

<template>
  <section>
    <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
      {{ t('settings.voiceTitle') }}
    </h2>
    <p class="text-xs opacity-60 mb-2">{{ t('settings.voiceHint') }}</p>

    <DictationSettings
      :voice-status="voiceStatus"
      :download-pct="downloadPct"
      @reload="loadShared"
    />

    <ReadAloudSettings
      :voice-status="voiceStatus"
      :download-pct="downloadPct"
      @reload="loadShared"
    />

    <!-- Disk usage summary -->
    <div class="mt-3 mb-1">
      <template v-if="installedCount > 0">
        <p class="text-xs opacity-70">
          {{
            t('settings.voiceDiskUsage', {
              count: String(installedCount),
              size: formatBytes(totalDiskBytes),
            })
          }}
        </p>
      </template>
      <p v-else class="text-xs opacity-50 italic">
        {{ t('settings.voiceDiskNone') }}
      </p>
    </div>
  </section>
</template>
