<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import ToggleSwitch from 'primevue/toggleswitch';
import { useAudioRecorder } from '@/audio/useAudioRecorder';
import { useIpcError } from '@/composables/useIpcError';
import { ipc } from '@/services/ipc';
import type { MediaAsset } from '@draffity/shared-types';

const props = defineProps<{ visible: boolean; projectId: string }>();
const emit = defineEmits<{ 'update:visible': [boolean] }>();

const { t } = useI18n();
const { run: ipcRun } = useIpcError();
const recorder = useAudioRecorder();

const notes = ref<MediaAsset[]>([]);
const urls = ref<Record<string, string>>({});
const transcribeOn = ref(true);
const saving = ref(false);

const visibleModel = computed({
  get: () => props.visible,
  set: (v: boolean) => emit('update:visible', v),
});

function revokeUrls() {
  for (const u of Object.values(urls.value)) URL.revokeObjectURL(u);
  urls.value = {};
}

async function loadNotes() {
  const list = await ipcRun(t('voice.notes.error'), () => ipc.listVoiceNotes(props.projectId));
  notes.value = list ?? [];
  revokeUrls();
  // Resolve a playable blob URL per note (audio/wav).
  const next: Record<string, string> = {};
  for (const n of notes.value) {
    try {
      const bytes = await ipc.readMediaBytes(n.id);
      next[n.id] = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: 'audio/wav' }));
    } catch {
      // skip unplayable
    }
  }
  urls.value = next;
}

watch(
  () => props.visible,
  (v) => {
    if (v) void loadNotes();
    else recorder.cancel();
  },
);

async function onStartStop() {
  if (recorder.state.value === 'recording') {
    saving.value = true;
    try {
      const rec = await recorder.stop();
      await ipcRun(t('voice.notes.error'), () =>
        ipc.saveVoiceNote({
          projectId: props.projectId,
          wav: rec.wav,
          durationMs: rec.durationMs,
          transcribe: transcribeOn.value,
        }),
      );
      await loadNotes();
    } finally {
      saving.value = false;
    }
  } else {
    await recorder.start();
  }
}

async function onDelete(note: MediaAsset) {
  await ipcRun(t('voice.notes.error'), () => ipc.deleteVoiceNote(note.id));
  await loadNotes();
}

function formatDuration(ms?: number | null): string {
  if (!ms) return '';
  const s = Math.round(ms / 1000);
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, '0')}`;
}

function formatDate(ms: number): string {
  return new Date(ms).toLocaleString();
}

onBeforeUnmount(() => {
  recorder.cancel();
  revokeUrls();
});
</script>

<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :header="t('voice.notes.title')"
    :style="{ width: '34rem', maxWidth: '95vw' }"
  >
    <div class="space-y-4">
      <!-- Recorder -->
      <div
        class="flex items-center gap-3 p-3 rounded border border-surface-200 dark:border-surface-700"
      >
        <Button
          :icon="recorder.state.value === 'recording' ? 'pi pi-stop-circle' : 'pi pi-microphone'"
          :label="
            recorder.state.value === 'recording' ? t('voice.notes.stop') : t('voice.notes.record')
          "
          :severity="recorder.state.value === 'recording' ? 'danger' : 'primary'"
          size="small"
          :loading="saving"
          @click="onStartStop"
        />
        <div
          v-if="recorder.state.value === 'recording'"
          class="h-2 flex-1 rounded-full bg-surface-200 dark:bg-surface-700 overflow-hidden"
        >
          <div
            class="h-full bg-red-500 transition-[width] duration-75"
            :style="{ width: `${Math.min(100, Math.round(recorder.level.value * 240))}%` }"
          />
        </div>
        <label class="flex items-center gap-2 text-xs ml-auto">
          {{ t('voice.notes.transcribe') }}
          <ToggleSwitch v-model="transcribeOn" />
        </label>
      </div>

      <!-- List -->
      <div v-if="notes.length === 0" class="text-sm opacity-60 text-center py-4">
        {{ t('voice.notes.empty') }}
      </div>
      <ul v-else class="space-y-3 max-h-[50vh] overflow-auto">
        <li
          v-for="n in notes"
          :key="n.id"
          class="rounded border border-surface-200 dark:border-surface-700 p-3"
        >
          <div class="flex items-center justify-between gap-2 mb-1 text-xs opacity-60">
            <span>{{ formatDate(n.createdAt) }}</span>
            <span class="flex items-center gap-2">
              <span v-if="n.durationMs" class="font-mono">{{ formatDuration(n.durationMs) }}</span>
              <Button
                icon="pi pi-trash"
                size="small"
                text
                severity="danger"
                :aria-label="t('voice.notes.delete')"
                @click="onDelete(n)"
              />
            </span>
          </div>
          <audio v-if="urls[n.id]" :src="urls[n.id]" controls class="w-full" />
          <p v-if="n.transcribedText" class="text-sm mt-2 opacity-80 italic">
            {{ n.transcribedText }}
          </p>
        </li>
      </ul>
    </div>
  </Dialog>
</template>
