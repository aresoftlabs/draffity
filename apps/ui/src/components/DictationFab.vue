<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';

defineProps<{ available: boolean; active: boolean }>();
defineEmits<{ toggle: [] }>();

const { t } = useI18n();
</script>

<template>
  <Button
    v-if="available && !active"
    data-test="dictation-fab"
    v-tooltip.left="t('toolbar.dictate')"
    :aria-label="t('toolbar.dictate')"
    icon="pi pi-microphone"
    rounded
    severity="secondary"
    class="fixed bottom-6 right-6 z-40 shadow-md !w-11 !h-11 bg-surface-0 dark:bg-surface-900 border border-surface-200 dark:border-surface-700"
    @click="$emit('toggle')"
  />
</template>
