<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import type { SaveState } from '@/stores/document';

const props = defineProps<{
  state: SaveState;
  lastSavedAt?: number | null;
}>();

const { t } = useI18n();

const label = computed(() => {
  switch (props.state) {
    case 'saving':
      return t('save.saving');
    case 'saved':
      return t('save.saved');
    case 'error':
      return t('save.error');
    default:
      return '';
  }
});

const iconClass = computed(() => {
  switch (props.state) {
    case 'saving':
      return 'pi pi-spin pi-spinner text-amber-500';
    case 'saved':
      return 'pi pi-check text-emerald-500';
    case 'error':
      return 'pi pi-exclamation-triangle text-rose-500';
    default:
      return 'pi pi-circle text-surface-400';
  }
});
</script>

<template>
  <span
    class="inline-flex items-center gap-2 text-xs opacity-75 select-none"
    role="status"
    aria-live="polite"
    aria-atomic="true"
  >
    <i :class="iconClass" aria-hidden="true" />
    <span v-if="label">{{ label }}</span>
  </span>
</template>
