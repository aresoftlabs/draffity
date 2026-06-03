<script setup lang="ts">
import { computed, ref, watchEffect } from 'vue';
import { NodeViewWrapper, nodeViewProps } from '@tiptap/vue-3';
import { useMediaStore } from '@/stores/media';

const props = defineProps(nodeViewProps);

const store = useMediaStore();
const src = ref<string | null>(null);
const failed = ref(false);

const attrs = computed(() => props.node.attrs as { mediaId?: string; alt?: string });
const alt = computed(() => attrs.value.alt ?? '');

watchEffect(async () => {
  const mediaId = attrs.value.mediaId;
  if (!mediaId) {
    src.value = null;
    failed.value = false;
    return;
  }
  failed.value = false;
  const url = await store.resolve(mediaId);
  src.value = url;
  failed.value = url === null;
});
</script>

<template>
  <NodeViewWrapper as="span" class="draffity-image-wrapper">
    <img v-if="src" :src="src" :alt="alt" class="draffity-image max-w-full h-auto rounded" />
    <span
      v-else-if="failed"
      class="draffity-image-missing inline-flex items-center gap-1 px-2 py-1 text-xs rounded bg-rose-100 dark:bg-rose-900/40 text-rose-700 dark:text-rose-300"
    >
      <i class="pi pi-exclamation-triangle" aria-hidden="true" />
      <span>{{ $t('editor.image.missing') }}</span>
    </span>
    <span
      v-else
      class="draffity-image-loading inline-block w-32 h-20 rounded bg-surface-100 dark:bg-surface-800 animate-pulse"
    />
  </NodeViewWrapper>
</template>
