<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';

const props = defineProps<{
  visible: boolean;
  nextTitle: string;
  currentTitle: string | null;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  confirm: [];
}>();

const { t } = useI18n();

const message = computed(() => {
  if (props.currentTitle) {
    return t('switchProject.message', {
      next: props.nextTitle,
      current: props.currentTitle,
    });
  }
  return t('switchProject.messageNoCurrent', { next: props.nextTitle });
});
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="t('switchProject.title')"
    :style="{ width: '32rem' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <p class="text-sm leading-relaxed">{{ message }}</p>
    <template #footer>
      <Button
        :label="t('actions.cancel')"
        text
        severity="secondary"
        @click="emit('update:visible', false)"
      />
      <Button :label="t('switchProject.confirm')" icon="pi pi-check" @click="emit('confirm')" />
    </template>
  </Dialog>
</template>
