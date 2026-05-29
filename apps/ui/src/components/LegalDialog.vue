<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';

import privacyEs from '@/assets/legal/privacy.es.md?raw';
import privacyEn from '@/assets/legal/privacy.en.md?raw';
import tosEs from '@/assets/legal/tos.es.md?raw';
import tosEn from '@/assets/legal/tos.en.md?raw';

/**
 * Renders the privacy policy or the terms of use inside an in-app dialog.
 * The source markdown lives under `src/assets/legal/` (copies of the
 * canonical files in `/docs`) so the policies stay readable offline.
 * No markdown renderer is pulled in — the text is shown as pre-wrapped
 * monospace, which is good enough for legalese and avoids a runtime dep.
 */

export type LegalKind = 'privacy' | 'tos';

const props = defineProps<{
  visible: boolean;
  kind: LegalKind | null;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
}>();

const { t, locale } = useI18n();

const content = computed(() => {
  if (!props.kind) return '';
  if (props.kind === 'privacy') {
    return locale.value === 'en' ? privacyEn : privacyEs;
  }
  return locale.value === 'en' ? tosEn : tosEs;
});

const title = computed(() =>
  props.kind === 'tos' ? t('legal.tosTitle') : t('legal.privacyTitle'),
);

function close() {
  emit('update:visible', false);
}
</script>

<template>
  <Dialog
    :visible="visible"
    :header="title"
    modal
    :style="{ width: '80vw', maxWidth: '760px' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <pre class="font-mono text-sm whitespace-pre-wrap break-words max-h-[70vh] overflow-auto">{{
      content
    }}</pre>
    <template #footer>
      <Button :label="t('legal.close')" @click="close" />
    </template>
  </Dialog>
</template>
