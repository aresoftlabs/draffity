<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import Select from 'primevue/select';
import SelectButton from 'primevue/selectbutton';
import InputNumber from 'primevue/inputnumber';
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import { NAME_ORIGINS, type NameGender } from '@/data/names';
import { pickNames, generateNames } from '@/composables/useNameGenerator';
import { useUiStore } from '@/stores/ui';
import { useIpcError } from '@/composables/useIpcError';

const props = defineProps<{ visible: boolean }>();
const emit = defineEmits<{ 'update:visible': [boolean]; select: [string] }>();

const { t } = useI18n();
const ui = useUiStore();
const { run } = useIpcError();

const visibleModel = computed({
  get: () => props.visible,
  set: (v: boolean) => emit('update:visible', v),
});

const originId = ref<string>(NAME_ORIGINS[0].id);
const gender = ref<NameGender>('unisex');
const count = ref<number>(8);
const results = ref<string[]>([]);

const originOptions = computed(() => [
  ...NAME_ORIGINS.map((o) => ({ value: o.id, label: t(`nameGen.origin.${o.id}`) })),
  ...ui.customNameLists.map((l) => ({ value: l.id, label: `★ ${l.label}` })),
]);

const genderOptions = computed<{ value: NameGender; label: string }[]>(() => [
  { value: 'masc', label: t('nameGen.masc') },
  { value: 'fem', label: t('nameGen.fem') },
  { value: 'unisex', label: t('nameGen.unisex') },
]);

const isCustom = computed(() => originId.value.startsWith('custom:'));

function generate() {
  const n = Math.max(1, Math.min(30, count.value || 8));
  if (isCustom.value) {
    const list = ui.customNameLists.find((l) => l.id === originId.value);
    results.value = list ? pickNames(list.names, n) : [];
  } else {
    results.value = generateNames(originId.value, gender.value, n);
  }
}

function onPick(name: string) {
  emit('select', name);
  visibleModel.value = false;
}

async function copy(name: string) {
  try {
    await navigator.clipboard.writeText(name);
  } catch {
    /* clipboard may be unavailable; ignore */
  }
}

async function importList() {
  const path = await open({
    multiple: false,
    filters: [{ name: 'Listas', extensions: ['txt', 'json'] }],
  });
  if (typeof path !== 'string') return;
  await run(t('nameGen.importError'), async () => {
    const raw = await readTextFile(path);
    let names: string[];
    if (path.toLowerCase().endsWith('.json')) {
      const parsed = JSON.parse(raw);
      names = Array.isArray(parsed) ? parsed.map(String) : [];
    } else {
      names = raw.split(/\r?\n/);
    }
    const label =
      path
        .split(/[\\/]/)
        .pop()
        ?.replace(/\.(txt|json)$/i, '') ?? 'lista';
    ui.addCustomNameList(label, names);
    const added = ui.customNameLists[ui.customNameLists.length - 1];
    if (added) {
      originId.value = added.id;
      generate();
    }
  });
}

// Generate an initial batch whenever the dialog opens.
watch(
  () => props.visible,
  (v) => {
    if (v) generate();
  },
);
</script>

<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :header="t('nameGen.title')"
    :style="{ width: '30rem', maxWidth: '95vw' }"
  >
    <div class="space-y-3">
      <div class="flex gap-2">
        <Select
          v-model="originId"
          :options="originOptions"
          option-label="label"
          option-value="value"
          class="flex-1"
          size="small"
          @change="generate"
        />
        <Button
          v-tooltip.bottom="t('nameGen.import')"
          icon="pi pi-upload"
          text
          size="small"
          :aria-label="t('nameGen.import')"
          @click="importList"
        />
      </div>

      <div class="flex items-center gap-2 justify-between">
        <SelectButton
          v-model="gender"
          :options="genderOptions"
          option-label="label"
          option-value="value"
          :allow-empty="false"
          :disabled="isCustom"
          size="small"
        />
        <InputNumber v-model="count" :min="1" :max="30" show-buttons class="!w-28" size="small" />
      </div>

      <Button :label="t('nameGen.generate')" icon="pi pi-refresh" size="small" @click="generate" />

      <ul v-if="results.length > 0" class="grid grid-cols-2 gap-1 pt-1">
        <li
          v-for="name in results"
          :key="name"
          class="group flex items-center gap-1 rounded px-2 py-1 hover:bg-surface-100 dark:hover:bg-surface-800"
        >
          <button class="flex-1 min-w-0 truncate text-left text-sm" @click="onPick(name)">
            {{ name }}
          </button>
          <Button
            icon="pi pi-copy"
            text
            size="small"
            class="opacity-0 group-hover:opacity-100"
            :pt="{ root: { class: '!w-6 !h-6 !p-0' } }"
            :aria-label="t('nameGen.copy')"
            @click="copy(name)"
          />
        </li>
      </ul>
      <p v-else class="text-sm opacity-50">{{ t('nameGen.empty') }}</p>
      <p class="text-[11px] opacity-50">{{ t('nameGen.pickHint') }}</p>
    </div>

    <template #footer>
      <Button :label="t('actions.close')" text @click="visibleModel = false" />
    </template>
  </Dialog>
</template>
