<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import Textarea from 'primevue/textarea';
import { useToast } from 'primevue/usetoast';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';

const props = defineProps<{
  visible: boolean;
  projectId: string | null;
}>();

const emit = defineEmits<{ 'update:visible': [value: boolean] }>();

const { t, locale } = useI18n();
const { run } = useIpcError();
const toast = useToast();

const name = ref('');
const description = ref('');
const saving = ref(false);

watch(
  () => props.visible,
  (v) => {
    if (v) {
      name.value = '';
      description.value = '';
      saving.value = false;
    }
  },
);

function close() {
  emit('update:visible', false);
}

async function onSave() {
  if (!props.projectId || !name.value.trim()) return;
  saving.value = true;
  const saved = await run(t('saveAsTemplate.saveError'), () =>
    ipc.saveProjectAsTemplate({
      projectId: props.projectId!,
      name: name.value.trim(),
      description: description.value.trim() || undefined,
      locale: locale.value,
    }),
  );
  saving.value = false;
  if (saved) {
    toast.add({
      severity: 'success',
      summary: t('saveAsTemplate.title'),
      detail: t('saveAsTemplate.saveSuccess', { name: saved.name }),
      life: 4000,
    });
    close();
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="t('saveAsTemplate.title')"
    :style="{ width: '28rem' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="flex flex-col gap-4">
      <p class="text-sm opacity-70">{{ t('saveAsTemplate.subtitle') }}</p>
      <InputText v-model="name" :placeholder="t('saveAsTemplate.namePlaceholder')" autofocus />
      <Textarea
        v-model="description"
        :placeholder="t('saveAsTemplate.descriptionPlaceholder')"
        rows="3"
      />
    </div>
    <template #footer>
      <Button :label="t('actions.cancel')" text severity="secondary" @click="close" />
      <Button
        :label="t('saveAsTemplate.save')"
        icon="pi pi-save"
        :loading="saving"
        :disabled="!name.trim() || saving"
        @click="onSave"
      />
    </template>
  </Dialog>
</template>
