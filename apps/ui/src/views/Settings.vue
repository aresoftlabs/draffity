<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import SelectButton from 'primevue/selectbutton';
import Select from 'primevue/select';
import Slider from 'primevue/slider';
import ToggleSwitch from 'primevue/toggleswitch';
import Button from 'primevue/button';
import InputNumber from 'primevue/inputnumber';
import Chips from 'primevue/chips';
import Textarea from 'primevue/textarea';
import { useToast } from 'primevue/usetoast';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import KeybindingsEditor from '@/components/KeybindingsEditor.vue';
import LegalDialog, { type LegalKind } from '@/components/LegalDialog.vue';
import SettingsBackups from '@/components/SettingsBackups.vue';
import SettingsAI from '@/components/SettingsAI.vue';
import SettingsVoice from '@/components/SettingsVoice.vue';
import SettingsStats from '@/components/SettingsStats.vue';
import SettingsUpdates from '@/components/SettingsUpdates.vue';
import { useUiStore } from '@/stores/ui';
import { useProjectStore } from '@/stores/project';
import { builtInFamily, useEditorSettings, type EditorFont } from '@/composables/useEditorSettings';
import { ipc } from '@/services/ipc';
import type { MediaAsset } from '@draffity/shared-types';

const { t, locale } = useI18n();
const ui = useUiStore();
const projectStore = useProjectStore();
const { autosaveMs, font, fontFamily, customFontId, customCss } = useEditorSettings();

const themeOptions = computed(() => [
  { label: t('settings.themeLight'), value: 'light' },
  { label: t('settings.themeDark'), value: 'dark' },
  { label: t('settings.themeHighContrast'), value: 'high-contrast' },
  { label: t('settings.themeSystem'), value: 'system' },
]);

const localeOptions = [
  { label: 'Español', value: 'es' },
  { label: 'English', value: 'en' },
];

const toast = useToast();

const fontOptions = computed(() => [
  { label: t('settings.fontSerif'), value: 'serif' as EditorFont },
  { label: t('settings.fontSans'), value: 'sans' as EditorFont },
  { label: t('settings.fontMono'), value: 'mono' as EditorFont },
]);

// Common system fonts. Best-effort: not every machine has Garamond etc.,
// so the CSS family stack always falls back to the closest built-in.
const SYSTEM_FONTS: { label: string; value: string }[] = [
  { label: 'Georgia', value: "Georgia, 'Times New Roman', serif" },
  { label: 'Garamond', value: "Garamond, 'EB Garamond', Georgia, serif" },
  { label: 'Palatino', value: "Palatino, 'Palatino Linotype', 'Book Antiqua', serif" },
  { label: 'Helvetica', value: 'Helvetica, Arial, sans-serif' },
  { label: 'Verdana', value: 'Verdana, Geneva, sans-serif' },
  { label: 'Courier', value: "'Courier New', Courier, monospace" },
];

const customFonts = ref<MediaAsset[]>([]);
const uploadingFont = ref(false);

async function loadCustomFonts() {
  const pid = projectStore.active?.id;
  if (!pid) {
    customFonts.value = [];
    return;
  }
  try {
    const all = await ipc.listProjectMedia(pid);
    customFonts.value = all.filter((m) => m.mime.startsWith('font/'));
  } catch (e) {
    customFonts.value = [];
    reportLoadError('customFonts', e);
  }
}

const fontSelectGroups = computed(() => {
  const groups: { label: string; items: { label: string; value: string }[] }[] = [
    {
      label: t('settings.fontGroupBuiltIn'),
      items: [
        { label: 'Source Serif 4 (Serif)', value: builtInFamily('serif') },
        { label: 'Inter (Sans)', value: builtInFamily('sans') },
        { label: 'JetBrains Mono (Mono)', value: builtInFamily('mono') },
      ],
    },
    { label: t('settings.fontGroupSystem'), items: SYSTEM_FONTS },
  ];
  if (customFonts.value.length > 0) {
    groups.push({
      label: t('settings.fontGroupCustom'),
      items: customFonts.value.map((f) => ({
        label: prettyFontName(f),
        value: `custom:${f.id}`,
      })),
    });
  }
  return groups;
});

function prettyFontName(asset: MediaAsset): string {
  const base = asset.pathRelative.split(/[\\/]/).pop() ?? asset.id;
  return base.replace(/\.[^.]+$/, '');
}

const fontFamilyModel = computed({
  get: () => (customFontId.value ? `custom:${customFontId.value}` : fontFamily.value),
  set: (v: string) => {
    if (v.startsWith('custom:')) {
      customFontId.value = v.slice('custom:'.length);
    } else {
      customFontId.value = null;
      fontFamily.value = v;
    }
  },
});

async function onUploadFont() {
  const pid = projectStore.active?.id;
  if (!pid) return;
  const picked = await open({
    multiple: false,
    directory: false,
    filters: [{ name: 'Fonts', extensions: ['ttf', 'otf'] }],
    title: t('settings.uploadFont'),
  });
  if (typeof picked !== 'string') return;
  uploadingFont.value = true;
  try {
    const bytes = await readFile(picked);
    const mime = picked.toLowerCase().endsWith('.otf') ? 'font/otf' : 'font/ttf';
    const asset = await ipc.uploadMedia({
      projectId: pid,
      mime,
      bytes: Array.from(bytes),
    });
    await loadCustomFonts();
    customFontId.value = asset.id;
    toast.add({
      severity: 'success',
      summary: t('settings.editorFont'),
      detail: t('settings.fontUploaded'),
      life: 3000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.editorFont'),
      detail: t('settings.fontUploadFailed'),
      life: 5000,
    });
  } finally {
    uploadingFont.value = false;
  }
}

const themeModel = computed({
  get: () => ui.theme,
  set: (v) => ui.setTheme(v as 'light' | 'dark' | 'high-contrast' | 'system'),
});

const localeModel = computed({
  get: () => locale.value,
  set: (v: string) => ui.setLocale(v as 'es' | 'en'),
});

/**
 * Surface a loader IPC failure instead of degrading silently to an empty state.
 * Always logs (telemetry-ready); `notify` also toasts so the user gets a signal
 * when a settings loader fails (AUD-16).
 */
function reportLoadError(scope: string, e: unknown, notify = false) {
  console.error('[settings]', scope, e);
  if (notify) {
    toast.add({ severity: 'error', summary: t('settings.loadError'), life: 5000 });
  }
}

const crashReportingActive = ref(false);
const crashReportingEnabled = ref(false);
async function loadCrashReporting() {
  try {
    const status = await ipc.getCrashReportingStatus();
    crashReportingActive.value = status.active;
    crashReportingEnabled.value = status.enabled;
  } catch (e) {
    crashReportingActive.value = false;
    crashReportingEnabled.value = false;
    reportLoadError('crashReporting', e);
  }
}
async function onToggleCrashReporting(value: boolean) {
  crashReportingEnabled.value = value;
  try {
    await ipc.setCrashReportingEnabled(value);
  } catch {
    // Revert visual state if the IPC call fails.
    crashReportingEnabled.value = !value;
  }
}

const legalKind = ref<LegalKind | null>(null);
const legalVisible = computed({
  get: () => legalKind.value !== null,
  set: (v: boolean) => {
    if (!v) legalKind.value = null;
  },
});

function onOpenPolicy(kind: LegalKind) {
  legalKind.value = kind;
}

onMounted(async () => {
  await loadCustomFonts();
  await loadCrashReporting();
  try {
    resourcesPath.value = await ipc.getResourcesPath();
  } catch {
    // best-effort
  }
});

type SettingsSection =
  | 'appearance'
  | 'editor'
  | 'language'
  | 'audio'
  | 'ai'
  | 'shortcuts'
  | 'goals'
  | 'data'
  | 'about';

const activeSection = ref<SettingsSection>('appearance');

const resourcesPath = ref('');

async function onChangeResourcesPath() {
  const picked = await open({
    multiple: false,
    directory: true,
    title: t('settings.resourcesPathChange'),
  });
  if (typeof picked !== 'string') return;
  try {
    await ipc.setResourcesPath(picked);
    resourcesPath.value = picked;
    toast.add({
      severity: 'success',
      summary: t('settings.resourcesPath'),
      detail: t('settings.resourcesPathSaved'),
      life: 5000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.resourcesPath'),
      detail: t('settings.loadError'),
      life: 5000,
    });
  }
}

const navSections: { id: SettingsSection; key: string }[] = [
  { id: 'appearance', key: 'settings.nav.appearance' },
  { id: 'editor', key: 'settings.nav.editor' },
  { id: 'language', key: 'settings.nav.language' },
  { id: 'audio', key: 'settings.nav.audio' },
  { id: 'ai', key: 'settings.nav.ai' },
  { id: 'shortcuts', key: 'settings.nav.shortcuts' },
  { id: 'goals', key: 'settings.nav.goals' },
  { id: 'data', key: 'settings.nav.data' },
  { id: 'about', key: 'settings.nav.about' },
];
</script>

<template>
  <section class="flex-1 min-h-0 overflow-y-auto p-8 max-w-5xl w-full mx-auto">
    <h1 class="text-2xl font-display font-bold mb-6">{{ t('settings.title') }}</h1>
    <div class="flex gap-8 items-start">
      <nav class="w-52 shrink-0 flex flex-col gap-1 sticky top-4" :aria-label="t('settings.title')">
        <button
          v-for="s in navSections"
          :key="s.id"
          type="button"
          class="text-left text-sm px-3 py-2 rounded-lg transition-colors"
          :class="
            activeSection === s.id
              ? 'bg-surface-0 dark:bg-surface-800 font-medium text-surface-900 dark:text-surface-50 shadow-sm'
              : 'text-surface-600 dark:text-surface-300 hover:bg-surface-100 dark:hover:bg-surface-800'
          "
          :aria-current="activeSection === s.id ? 'page' : undefined"
          @click="activeSection = s.id"
        >
          {{ t(s.key) }}
        </button>
      </nav>
      <div class="flex-1 min-w-0 max-w-2xl">
        <!-- APARIENCIA -->
        <div v-show="activeSection === 'appearance'" class="space-y-8">
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
              {{ t('settings.editorFont') }}
            </h2>
            <div class="flex items-center gap-2">
              <Select
                v-model="fontFamilyModel"
                :options="fontSelectGroups"
                option-label="label"
                option-value="value"
                option-group-label="label"
                option-group-children="items"
                class="flex-1"
                :placeholder="t('settings.fontPickerPlaceholder')"
              />
              <Button
                v-tooltip.left="
                  projectStore.active ? t('settings.uploadFont') : t('settings.uploadFontNoProject')
                "
                :aria-label="t('settings.uploadFont')"
                icon="pi pi-upload"
                size="small"
                severity="secondary"
                :disabled="!projectStore.active || uploadingFont"
                :loading="uploadingFont"
                @click="onUploadFont"
              />
            </div>
            <p class="text-xs opacity-60 mt-2">{{ t('settings.fontHint') }}</p>
            <!-- Legacy 3-button selector kept for users who prefer it as a quick
                 toggle; it just swaps the family stack underneath. -->
            <div class="mt-3">
              <SelectButton
                v-model="font"
                :options="fontOptions"
                option-label="label"
                option-value="value"
                @update:model-value="(v: EditorFont) => (fontFamilyModel = builtInFamily(v))"
              />
            </div>
          </section>

          <section class="flex items-center justify-between gap-4">
            <div>
              <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70">
                {{ t('settings.readingSpeed') }}
              </h2>
              <p class="text-xs opacity-60 mt-1">{{ t('settings.readingSpeedHint') }}</p>
            </div>
            <InputNumber
              v-model="ui.readingWpm"
              :min="50"
              :max="1000"
              :step="10"
              suffix=" wpm"
              show-buttons
              class="!w-40"
              :aria-label="t('settings.readingSpeed')"
            />
          </section>

          <section>
            <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
              {{ t('settings.customCss') }}
            </h2>
            <p class="text-xs opacity-60 mb-2">{{ t('settings.customCssHint') }}</p>
            <Textarea
              v-model="customCss"
              rows="6"
              class="w-full font-mono text-xs"
              :placeholder="t('settings.customCssPlaceholder')"
              spellcheck="false"
            />
          </section>
        </div>

        <!-- EDITOR -->
        <div v-show="activeSection === 'editor'" class="space-y-8">
          <section>
            <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
              {{ t('settings.autosave') }}
              <span class="font-mono opacity-60">{{ autosaveMs }} ms</span>
            </h2>
            <Slider v-model="autosaveMs" :min="200" :max="3000" :step="100" />
            <p class="text-xs opacity-60 mt-2">{{ t('settings.autosaveHint') }}</p>
          </section>

          <section class="flex items-center justify-between gap-4">
            <div>
              <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70">
                {{ t('settings.typewriter') }}
              </h2>
              <p class="text-xs opacity-60 mt-1">{{ t('settings.typewriterHint') }}</p>
            </div>
            <ToggleSwitch v-model="ui.typewriterMode" />
          </section>

          <section>
            <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
              {{ t('settings.linguisticFocusTitle') }}
            </h2>
            <p class="text-xs opacity-60 mb-2">{{ t('settings.linguisticFocusHint') }}</p>
            <Chips
              v-model="ui.linguisticExtraWords"
              :placeholder="t('settings.linguisticExtraPlaceholder')"
              separator=","
              class="w-full"
            />
          </section>
        </div>

        <!-- IDIOMA -->
        <div v-show="activeSection === 'language'" class="space-y-8">
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
        </div>

        <!-- AUDIO -->
        <div v-show="activeSection === 'audio'" class="space-y-8">
          <SettingsVoice />
        </div>

        <!-- IA -->
        <div v-show="activeSection === 'ai'" class="space-y-8">
          <SettingsAI />
        </div>

        <!-- ATAJOS -->
        <div v-show="activeSection === 'shortcuts'" class="space-y-8">
          <section>
            <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
              {{ t('shortcuts.title') }}
            </h2>
            <p class="text-xs opacity-60 mb-2">{{ t('shortcuts.hint') }}</p>
            <KeybindingsEditor />
          </section>
        </div>

        <!-- OBJETIVOS -->
        <div v-show="activeSection === 'goals'" class="space-y-8">
          <SettingsStats />
        </div>

        <!-- COPIAS -->
        <div v-show="activeSection === 'data'" class="space-y-8">
          <section>
            <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
              {{ t('settings.resourcesPath') }}
            </h2>
            <p class="text-xs font-mono opacity-70 mb-2 truncate">{{ resourcesPath }}</p>
            <Button
              :label="t('settings.resourcesPathChange')"
              icon="pi pi-folder-open"
              size="small"
              text
              @click="onChangeResourcesPath"
            />
          </section>
          <SettingsBackups />
        </div>

        <!-- ACERCA DE -->
        <div v-show="activeSection === 'about'" class="space-y-8">
          <SettingsUpdates />

          <section v-if="crashReportingActive" class="flex items-center justify-between gap-4">
            <div>
              <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70">
                {{ t('settings.crashReporting') }}
              </h2>
              <p class="text-xs opacity-60 mt-1">{{ t('settings.crashReportingHint') }}</p>
            </div>
            <ToggleSwitch
              :model-value="crashReportingEnabled"
              :aria-label="t('settings.crashReporting')"
              @update:model-value="onToggleCrashReporting"
            />
          </section>

          <section>
            <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-3">
              {{ t('settings.legal') }}
            </h2>
            <p class="text-xs opacity-60 mb-2">{{ t('settings.legalHint') }}</p>
            <div class="flex flex-col gap-1 text-sm">
              <a
                class="underline opacity-80 hover:opacity-100 cursor-pointer"
                tabindex="0"
                @click="onOpenPolicy('privacy')"
                @keydown.enter="onOpenPolicy('privacy')"
              >
                {{ t('settings.privacyLink') }}
              </a>
              <a
                class="underline opacity-80 hover:opacity-100 cursor-pointer"
                tabindex="0"
                @click="onOpenPolicy('tos')"
                @keydown.enter="onOpenPolicy('tos')"
              >
                {{ t('settings.tosLink') }}
              </a>
            </div>
          </section>
        </div>
      </div>
    </div>

    <LegalDialog v-model:visible="legalVisible" :kind="legalKind" />
  </section>
</template>
