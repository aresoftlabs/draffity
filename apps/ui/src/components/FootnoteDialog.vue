<script setup lang="ts">
import { ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Textarea from 'primevue/textarea';
import Button from 'primevue/button';

const props = defineProps<{
  visible: boolean;
  initialContent?: string;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  save: [content: string];
  remove: [];
}>();

const { t } = useI18n();
const content = ref('');
const editing = ref(false);

watch(
  () => [props.visible, props.initialContent] as const,
  ([v, initial]) => {
    if (!v) return;
    content.value = initial ?? '';
    editing.value = (initial ?? '').length > 0;
  },
);

function onSave() {
  emit('save', content.value.trim());
  emit('update:visible', false);
}

function onRemove() {
  emit('remove');
  emit('update:visible', false);
}

function onCancel() {
  emit('update:visible', false);
}
</script>

<template>
  <Dialog
    :visible="visible"
    :header="editing ? t('footnote.editTitle') : t('footnote.insertTitle')"
    modal
    :style="{ width: '480px' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <p class="text-xs opacity-70 mb-2">{{ t('footnote.hint') }}</p>
    <Textarea
      v-model="content"
      rows="5"
      class="w-full"
      :placeholder="t('footnote.placeholder')"
      autofocus
    />
    <template #footer>
      <div class="flex items-center justify-between w-full">
        <Button
          v-if="editing"
          :label="t('footnote.delete')"
          severity="danger"
          text
          @click="onRemove"
        />
        <span v-else />
        <div class="flex gap-2">
          <Button :label="t('footnote.cancel')" text @click="onCancel" />
          <Button
            :label="editing ? t('footnote.save') : t('footnote.insert')"
            :disabled="content.trim().length === 0"
            @click="onSave"
          />
        </div>
      </div>
    </template>
  </Dialog>
</template>
