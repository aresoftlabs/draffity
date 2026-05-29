<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import InputText from 'primevue/inputtext';
import Textarea from 'primevue/textarea';
import Select from 'primevue/select';
import Button from 'primevue/button';
import AutoComplete, { type AutoCompleteCompleteEvent } from 'primevue/autocomplete';
import { useToast } from 'primevue/usetoast';
import type { CodexEntry, CodexKind } from '@draffity/shared-types';
import { useCodexStore } from '@/stores/codex';
import { useIpcError } from '@/composables/useIpcError';
import NameGeneratorDialog from '@/components/NameGeneratorDialog.vue';

const props = defineProps<{
  visible: boolean;
  projectId: string | null;
  /** When set, the dialog is in "edit" mode and pre-fills from the entry. */
  entry?: CodexEntry | null;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  saved: [entry: CodexEntry];
}>();

const { t } = useI18n();
const { run } = useIpcError();
const toast = useToast();
const store = useCodexStore();

const name = ref('');
const kind = ref<CodexKind>('character');
const body = ref('');
const tags = ref<string[]>([]);
const tagSuggestions = ref<string[]>([]);
const saving = ref(false);
const showNameGenerator = ref(false);

const isEdit = computed(() => !!props.entry);

const kindOptions = computed(() => [
  { value: 'character', label: t('codex.kind.character') },
  { value: 'place', label: t('codex.kind.place') },
  { value: 'object', label: t('codex.kind.object') },
  { value: 'note', label: t('codex.kind.note') },
]);

watch(
  () => [props.visible, props.entry] as const,
  ([v, e]) => {
    if (!v) return;
    saving.value = false;
    if (e) {
      name.value = e.name;
      kind.value = e.kind;
      body.value = e.body ?? '';
      tags.value = [...e.tags];
    } else {
      name.value = '';
      kind.value = 'character';
      body.value = '';
      tags.value = [];
    }
  },
  { immediate: true },
);

function close() {
  emit('update:visible', false);
}

function onTagsSearch(event: AutoCompleteCompleteEvent) {
  const q = event.query.toLowerCase();
  tagSuggestions.value = store.allTags.filter(
    (t) => t.toLowerCase().includes(q) && !tags.value.includes(t),
  );
}

async function onSave() {
  if (!props.projectId) return;
  const trimmedName = name.value.trim();
  if (!trimmedName) return;
  saving.value = true;

  const cleanBody = body.value.trim();
  const cleanTags = Array.from(
    new Set(tags.value.map((t) => t.trim()).filter((t) => t.length > 0)),
  );

  let saved: CodexEntry | null = null;
  if (props.entry) {
    saved = await run(t('codex.saveError'), () =>
      store.update(props.entry!.id, {
        name: trimmedName,
        kind: kind.value,
        body: cleanBody,
        tags: cleanTags,
      }),
    );
  } else {
    saved = await run(t('codex.saveError'), () =>
      store.create({
        projectId: props.projectId!,
        kind: kind.value,
        name: trimmedName,
        body: cleanBody || null,
        tags: cleanTags,
      }),
    );
  }

  saving.value = false;
  if (saved) {
    toast.add({
      severity: 'success',
      summary: t('codex.title'),
      detail: isEdit.value
        ? t('codex.updateSuccess', { name: saved.name })
        : t('codex.createSuccess', { name: saved.name }),
      life: 3000,
    });
    emit('saved', saved);
    close();
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="isEdit ? t('codex.editTitle') : t('codex.createTitle')"
    :style="{ width: '36rem', maxHeight: '85vh' }"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <div class="flex flex-col gap-3">
      <div class="grid grid-cols-3 gap-3">
        <div class="col-span-2 flex flex-col gap-1">
          <label for="codex-name" class="text-sm font-medium">{{ t('codex.name') }}</label>
          <div class="flex gap-1">
            <InputText id="codex-name" v-model="name" autofocus class="flex-1" />
            <Button
              v-tooltip.bottom="t('nameGen.title')"
              icon="pi pi-bolt"
              text
              severity="secondary"
              :aria-label="t('nameGen.title')"
              @click="showNameGenerator = true"
            />
          </div>
        </div>
        <div class="flex flex-col gap-1">
          <label for="codex-kind" class="text-sm font-medium">{{ t('codex.kindLabel') }}</label>
          <Select
            id="codex-kind"
            v-model="kind"
            :options="kindOptions"
            option-label="label"
            option-value="value"
          />
        </div>
      </div>
      <div class="flex flex-col gap-1">
        <label for="codex-body" class="text-sm font-medium">{{ t('codex.body') }}</label>
        <Textarea id="codex-body" v-model="body" rows="6" auto-resize />
      </div>
      <div class="flex flex-col gap-1">
        <label for="codex-tags" class="text-sm font-medium">{{ t('codex.tags') }}</label>
        <AutoComplete
          id="codex-tags"
          v-model="tags"
          :suggestions="tagSuggestions"
          multiple
          fluid
          :placeholder="t('codex.tagsPlaceholder')"
          @complete="onTagsSearch"
        />
      </div>
    </div>
    <template #footer>
      <Button :label="t('actions.cancel')" text severity="secondary" @click="close" />
      <Button
        :label="isEdit ? t('actions.save') : t('actions.create')"
        :loading="saving"
        :disabled="!name.trim() || saving"
        @click="onSave"
      />
    </template>

    <NameGeneratorDialog v-model:visible="showNameGenerator" @select="(n) => (name = n)" />
  </Dialog>
</template>
