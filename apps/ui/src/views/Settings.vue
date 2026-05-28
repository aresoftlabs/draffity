<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import SelectButton from 'primevue/selectbutton';
import Slider from 'primevue/slider';
import { useUiStore } from '@/stores/ui';
import { useEditorSettings, type EditorFont } from '@/composables/useEditorSettings';
import { ipc } from '@/services/ipc';
import type { WritingStats } from '@draffity/shared-types';

const { t, locale } = useI18n();
const ui = useUiStore();
const { autosaveMs, font } = useEditorSettings();

const themeOptions = computed(() => [
  { label: t('settings.themeLight'), value: 'light' },
  { label: t('settings.themeDark'), value: 'dark' },
  { label: t('settings.themeSystem'), value: 'system' },
]);

const localeOptions = [
  { label: 'Español', value: 'es' },
  { label: 'English', value: 'en' },
];

const fontOptions = computed(() => [
  { label: t('settings.fontSerif'), value: 'serif' as EditorFont },
  { label: t('settings.fontSans'), value: 'sans' as EditorFont },
  { label: t('settings.fontMono'), value: 'mono' as EditorFont },
]);

const themeModel = computed({
  get: () => ui.theme,
  set: (v) => ui.setTheme(v as 'light' | 'dark' | 'system'),
});

const localeModel = computed({
  get: () => locale.value,
  set: (v: string) => ui.setLocale(v as 'es' | 'en'),
});

const stats = ref<WritingStats | null>(null);

onMounted(async () => {
  try {
    stats.value = await ipc.getWritingStats();
  } catch {
    stats.value = null;
  }
});
</script>

<template>
  <section class="flex-1 p-8 max-w-2xl w-full mx-auto">
    <h1 class="text-2xl font-serif font-bold mb-8">{{ t('settings.title') }}</h1>

    <div class="space-y-8">
      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.theme') }}
        </h2>
        <SelectButton
          v-model="themeModel"
          :options="themeOptions"
          option-label="label"
          option-value="value"
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.language') }}
        </h2>
        <SelectButton
          v-model="localeModel"
          :options="localeOptions"
          option-label="label"
          option-value="value"
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.editorFont') }}
        </h2>
        <SelectButton
          v-model="font"
          :options="fontOptions"
          option-label="label"
          option-value="value"
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.autosave') }}
          <span class="font-mono opacity-60">{{ autosaveMs }} ms</span>
        </h2>
        <Slider v-model="autosaveMs" :min="200" :max="3000" :step="100" />
        <p class="text-xs opacity-60 mt-2">{{ t('settings.autosaveHint') }}</p>
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
          {{ t('settings.writingStats') }}
        </h2>
        <dl v-if="stats" class="text-sm space-y-1">
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('settings.currentStreak') }}</dt>
            <dd class="font-mono">{{ stats.currentStreak }}</dd>
          </div>
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('settings.longestStreak') }}</dt>
            <dd class="font-mono">{{ stats.longestStreak }}</dd>
          </div>
          <div v-if="stats.lastWritingDate" class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('settings.lastWritingDate') }}</dt>
            <dd class="font-mono">{{ stats.lastWritingDate }}</dd>
          </div>
        </dl>
        <p v-else class="text-xs opacity-60">…</p>
      </section>
    </div>
  </section>
</template>
