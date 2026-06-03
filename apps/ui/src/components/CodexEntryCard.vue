<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Tag from 'primevue/tag';
import type { CodexEntry, CodexKind } from '@draffity/shared-types';

const props = defineProps<{ entry: CodexEntry }>();

defineEmits<{
  open: [id: string];
}>();

const { t } = useI18n();

const KIND_ICONS: Record<CodexKind, string> = {
  character: 'pi pi-user',
  place: 'pi pi-map-marker',
  object: 'pi pi-box',
  note: 'pi pi-bookmark',
};

const KIND_COLOR_CLASSES: Record<CodexKind, string> = {
  character: 'text-amber-600 dark:text-amber-400',
  place: 'text-emerald-600 dark:text-emerald-400',
  object: 'text-sky-600 dark:text-sky-400',
  note: 'text-slate-500 dark:text-slate-400',
};

const kindLabel = computed(() => t(`codex.kind.${props.entry.kind}`));

/** Strip HTML and clamp to a sensible preview length so cards stay grid-
 *  friendly even when the user pasted a wall of text into `body`. */
const bodyPreview = computed(() => {
  const raw = props.entry.body ?? '';
  if (!raw.trim()) return '';
  const stripped = raw
    .replace(/<[^>]*>/g, ' ')
    .replace(/\s+/g, ' ')
    .trim();
  return stripped.length > 220 ? `${stripped.slice(0, 220)}â€¦` : stripped;
});
</script>

<template>
  <button
    type="button"
    class="text-left flex flex-col gap-2 p-4 rounded-lg border border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-950 hover:border-primary-400 dark:hover:border-primary-500 hover:shadow-sm transition-colors"
    @click="$emit('open', entry.id)"
  >
    <div class="flex items-center gap-2">
      <i :class="[KIND_ICONS[entry.kind], KIND_COLOR_CLASSES[entry.kind]]" />
      <span class="font-medium truncate">{{ entry.name }}</span>
      <span class="ml-auto text-[10px] uppercase tracking-wide opacity-60">
        {{ kindLabel }}
      </span>
    </div>
    <p v-if="bodyPreview" class="text-sm opacity-80 leading-snug line-clamp-4">
      {{ bodyPreview }}
    </p>
    <p v-else class="text-xs italic opacity-40">{{ t('codex.noBody') }}</p>
    <div v-if="entry.tags.length > 0" class="flex flex-wrap gap-1">
      <Tag
        v-for="tag in entry.tags"
        :key="tag"
        :value="tag"
        severity="secondary"
        class="!text-[10px] !py-0 !px-1.5"
      />
    </div>
  </button>
</template>
