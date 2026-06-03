<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import { useConfirm } from 'primevue/useconfirm';
import type { Snapshot } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';
import SnapshotDiffView from './SnapshotDiffView.vue';

const props = defineProps<{
  documentId: string | null;
  /** Current editor HTML â€” used by the diff view as the "after" side
   *  when comparing a saved snapshot against the live document. */
  currentHtml?: string;
  readOnly?: boolean;
}>();

const emit = defineEmits<{
  restored: [];
}>();

const { t, d, locale } = useI18n();
const { run } = useIpcError();
const confirm = useConfirm();

const snapshots = ref<Snapshot[]>([]);
const creating = ref(false);
const labelDraft = ref('');
const showLabel = ref(false);

const visibleLimit = ref(5);
const visible = computed(() => snapshots.value.slice(0, visibleLimit.value));

watch(
  () => props.documentId,
  () => {
    void load();
  },
);

onMounted(load);

async function load() {
  if (!props.documentId) {
    snapshots.value = [];
    return;
  }
  const list = await run(t('errors.loadSnapshots'), () => ipc.listSnapshots(props.documentId!));
  snapshots.value = list ?? [];
}

async function onCreate() {
  if (!props.documentId) return;
  creating.value = true;
  const snap = await run(t('errors.createSnapshot'), () =>
    ipc.createSnapshot({
      documentId: props.documentId!,
      label: labelDraft.value.trim() || undefined,
    }),
  );
  creating.value = false;
  if (snap) {
    labelDraft.value = '';
    showLabel.value = false;
    snapshots.value = [snap, ...snapshots.value];
  }
}

function onRestore(s: Snapshot) {
  confirm.require({
    message: t('snapshots.confirmRestore'),
    icon: 'pi pi-history',
    acceptLabel: t('snapshots.restore'),
    rejectLabel: t('actions.cancel'),
    accept: async () => {
      const restored = await run(t('errors.restoreSnapshot'), () => ipc.restoreSnapshot(s.id));
      if (restored) {
        emit('restored');
        await load();
      }
    },
  });
}

function formatDate(ts: number): string {
  try {
    return d(new Date(ts), 'short');
  } catch {
    return new Date(ts).toLocaleString(locale.value);
  }
}

const diffSnapshot = ref<Snapshot | null>(null);
const diffVisible = computed({
  get: () => diffSnapshot.value !== null,
  set: (v: boolean) => {
    if (!v) diffSnapshot.value = null;
  },
});

function onCompare(s: Snapshot) {
  diffSnapshot.value = s;
}
</script>

<template>
  <section v-if="documentId">
    <div class="flex items-center justify-between mb-2">
      <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60">
        {{ t('snapshots.title') }}
      </h4>
      <Button
        v-if="!readOnly"
        icon="pi pi-plus"
        text
        severity="secondary"
        size="small"
        :aria-label="t('snapshots.newVersion')"
        @click="showLabel = !showLabel"
      />
    </div>

    <div v-if="showLabel && !readOnly" class="flex gap-2 mb-2">
      <InputText
        v-model="labelDraft"
        :placeholder="t('snapshots.labelPlaceholder')"
        class="flex-1"
      />
      <Button icon="pi pi-check" size="small" :loading="creating" @click="onCreate" />
    </div>

    <ul v-if="snapshots.length" class="space-y-1 text-sm">
      <li v-for="s in visible" :key="s.id" class="flex items-center justify-between gap-2 group">
        <span class="flex flex-col leading-tight min-w-0">
          <span class="truncate">{{ s.label || t('snapshots.unlabeled') }}</span>
          <span class="text-xs opacity-60 font-mono">{{ formatDate(s.createdAt) }}</span>
        </span>
        <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
          <Button
            icon="pi pi-eye"
            text
            severity="secondary"
            size="small"
            :aria-label="t('snapshots.compare')"
            @click="onCompare(s)"
          />
          <Button
            v-if="!readOnly"
            icon="pi pi-history"
            text
            severity="secondary"
            size="small"
            :aria-label="t('snapshots.restore')"
            @click="onRestore(s)"
          />
        </div>
      </li>
    </ul>
    <p v-else class="text-xs opacity-60">{{ t('snapshots.empty') }}</p>

    <button
      v-if="snapshots.length > visibleLimit"
      type="button"
      class="text-xs opacity-70 hover:opacity-100 mt-1"
      @click="visibleLimit += 5"
    >
      {{ t('snapshots.more') }}
    </button>

    <SnapshotDiffView
      v-model:visible="diffVisible"
      :before-label="
        diffSnapshot
          ? `${diffSnapshot.label ?? t('snapshots.unlabeled')} Â· ${formatDate(diffSnapshot.createdAt)}`
          : ''
      "
      :after-label="t('diff.current')"
      :before-html="diffSnapshot?.content ?? ''"
      :after-html="currentHtml ?? ''"
    />
  </section>
</template>
