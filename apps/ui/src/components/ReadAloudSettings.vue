<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useToast } from 'primevue/usetoast';
import Button from 'primevue/button';
import { open } from '@tauri-apps/plugin-dialog';
import VoiceCatalog from './VoiceCatalog.vue';
import { ipc, type CatalogLang, type VoiceStatus } from '@/services/ipc';

/**
 * "Lectura en voz alta" block (Piper/TTS): Piper binary status and the dynamic
 * voice catalog. Owns its own catalog state and TTS test playback; emits
 * `reload` so the container refetches the shared voice status after a binary
 * download/import or a catalog change.
 */
defineOptions({ name: 'ReadAloudSettings' });

const props = defineProps<{
  voiceStatus: VoiceStatus | null;
  downloadPct: Record<string, number>;
}>();
const emit = defineEmits<{ reload: [] }>();
const { t } = useI18n();
const toast = useToast();

const catalog = ref<CatalogLang[]>([]);
const testingVoiceId = ref<string | null>(null);
const importingPiper = ref(false);
const downloadingBinary = ref(false);
// Reused across test plays so we don't leak AudioContext instances.
let ttsCtx: AudioContext | null = null;

async function loadCatalog() {
  try {
    catalog.value = await ipc.getVoiceCatalog();
  } catch {
    catalog.value = [];
  }
}

async function onDownload(id: string) {
  try {
    await ipc.downloadVoiceVoice(id);
    await loadCatalog();
    emit('reload');
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceModelError'),
      life: 5000,
    });
  }
}

async function onDelete(id: string) {
  try {
    await ipc.deleteVoiceVoice(id);
    await loadCatalog();
    emit('reload');
  } catch {
    /* best-effort */
  }
}

async function onTest(id: string) {
  testingVoiceId.value = id;
  try {
    const audio = await ipc.synthesizeSpeech('Hello, this is a test voice.', id);
    if (!ttsCtx) ttsCtx = new AudioContext();
    if (ttsCtx.state === 'suspended') await ttsCtx.resume();
    const buf = ttsCtx.createBuffer(
      1,
      Math.max(1, audio.samplesPcm16.length),
      audio.sampleRate || 22050,
    );
    const ch = buf.getChannelData(0);
    for (let i = 0; i < audio.samplesPcm16.length; i++) ch[i] = audio.samplesPcm16[i] / 32768;
    const src = ttsCtx.createBufferSource();
    src.buffer = buf;
    src.connect(ttsCtx.destination);
    src.onended = () => {
      testingVoiceId.value = null;
    };
    src.start();
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceTestError'),
      life: 5000,
    });
    testingVoiceId.value = null;
  }
}

async function onDownloadBinary() {
  downloadingBinary.value = true;
  try {
    await ipc.downloadVoiceBinary('piper');
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error('[settings] downloadBinary failed:', 'piper', msg);
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: msg,
      life: 8000,
    });
  } finally {
    downloadingBinary.value = false;
    emit('reload');
  }
}

async function onImportPiper() {
  const picked = await open({
    multiple: false,
    directory: false,
    title: t('settings.voiceImportPiper'),
  });
  if (typeof picked !== 'string') return;
  importingPiper.value = true;
  try {
    await ipc.importPiperBinary(picked);
    toast.add({
      severity: 'success',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceBinaryImported'),
      life: 3000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceModelError'),
      life: 5000,
    });
  } finally {
    importingPiper.value = false;
    emit('reload');
  }
}

onMounted(loadCatalog);
</script>

<template>
  <section data-test="read-aloud-settings">
    <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mt-4 mb-2">
      {{ t('settings.voiceReadAloud') }}
    </h3>
    <div
      class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm mb-3"
    >
      <span>
        <i
          :class="
            props.voiceStatus?.piperInstalled
              ? 'pi pi-check-circle text-green-500'
              : 'pi pi-exclamation-circle text-amber-500'
          "
          class="mr-1"
        />
        {{
          props.voiceStatus?.piperInstalled
            ? t('settings.voicePiperInstalled')
            : t('settings.voicePiperMissing')
        }}
      </span>
      <div class="flex items-center gap-2">
        <span
          v-if="props.downloadPct['piper'] !== undefined"
          class="text-xs font-mono opacity-70 mr-2"
          >{{ `${props.downloadPct['piper']}%` }}</span
        >
        <Button
          v-if="!props.voiceStatus?.piperInstalled"
          :label="t('settings.voiceDownloadBinary')"
          icon="pi pi-download"
          size="small"
          :loading="downloadingBinary"
          :disabled="downloadingBinary"
          @click="onDownloadBinary"
        />
        <Button
          :label="t('settings.voiceImportPiper')"
          size="small"
          text
          :loading="importingPiper"
          @click="onImportPiper"
        />
      </div>
    </div>
    <VoiceCatalog
      :catalog="catalog"
      :download-pct="props.downloadPct"
      :testing-voice-id="testingVoiceId"
      @download="onDownload"
      @delete="onDelete"
      @test="onTest"
    />
  </section>
</template>
