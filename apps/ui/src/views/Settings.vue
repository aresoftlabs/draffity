<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import SelectButton from 'primevue/selectbutton';
import Select from 'primevue/select';
import Slider from 'primevue/slider';
import ToggleSwitch from 'primevue/toggleswitch';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import Chips from 'primevue/chips';
import Textarea from 'primevue/textarea';
import { useToast } from 'primevue/usetoast';
import { open } from '@tauri-apps/plugin-dialog';
import { readFile } from '@tauri-apps/plugin-fs';
import KeybindingsEditor from '@/components/KeybindingsEditor.vue';
import LegalDialog, { type LegalKind } from '@/components/LegalDialog.vue';
import SparklineChart from '@/components/SparklineChart.vue';
import { useUiStore } from '@/stores/ui';
import { useProjectStore } from '@/stores/project';
import { builtInFamily, useEditorSettings, type EditorFont } from '@/composables/useEditorSettings';
import { useIpcError } from '@/composables/useIpcError';
import { useCapability, refreshCapabilities } from '@/composables/useCapability';
import { useAiUsageStore } from '@/stores/aiUsage';
import {
  ipc,
  type AiStatus,
  type PremiumStatus,
  type VoiceModel,
  type VoiceStatus,
  type VoiceVoice,
  type VoiceDownloadProgress,
} from '@/services/ipc';
import type { BackupRecord, DailyWriting, MediaAsset, WritingStats } from '@draffity/shared-types';

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
  } catch {
    customFonts.value = [];
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

const crashReportingActive = ref(false);
const crashReportingEnabled = ref(false);
async function loadCrashReporting() {
  try {
    const status = await ipc.getCrashReportingStatus();
    crashReportingActive.value = status.active;
    crashReportingEnabled.value = status.enabled;
  } catch {
    crashReportingActive.value = false;
    crashReportingEnabled.value = false;
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

// Premium activation (E-07/E-08). Capability-gated sections only render once
// premium is active — Free tier sees nothing (no premium leakage). The
// activation field itself only shows when the build can validate licenses.
const aiEnabled = useCapability('ai_features');
const voiceEnabled = useCapability('voice_to_text');

// BYOK OpenRouter key (F-01 commands). Shown inside the premium-gated "IA"
// section so it never leaks to free users.
const aiStatus = ref<AiStatus | null>(null);
const openrouterKey = ref('');
const savingKey = ref(false);
const aiUsage = useAiUsageStore();

async function loadAiStatus() {
  try {
    aiStatus.value = await ipc.getAiStatus();
  } catch {
    aiStatus.value = null;
  }
}

async function onSaveOpenrouterKey() {
  const key = openrouterKey.value.trim();
  if (!key) return;
  savingKey.value = true;
  try {
    aiStatus.value = await ipc.setOpenrouterKey(key);
    openrouterKey.value = '';
    toast.add({
      severity: 'success',
      summary: t('settings.aiTitle'),
      detail: t('settings.aiKeySavedToast'),
      life: 3000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.aiTitle'),
      detail: t('settings.aiKeyError'),
      life: 5000,
    });
  } finally {
    savingKey.value = false;
  }
}

async function onClearOpenrouterKey() {
  try {
    aiStatus.value = await ipc.clearOpenrouterKey();
  } catch {
    // best-effort
  }
}
const premium = ref<PremiumStatus | null>(null);
const licenseKey = ref('');
const activatingPremium = ref(false);

async function loadPremium() {
  try {
    premium.value = await ipc.getPremiumStatus();
  } catch {
    premium.value = null;
  }
}

async function onActivatePremium() {
  const key = licenseKey.value.trim();
  if (!key) return;
  activatingPremium.value = true;
  try {
    premium.value = await ipc.activatePremium(key);
    licenseKey.value = '';
    await refreshCapabilities();
    toast.add({
      severity: 'success',
      summary: t('settings.premiumTitle'),
      detail: t('settings.premiumActivated'),
      life: 4000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.premiumTitle'),
      detail: t('settings.premiumInvalid'),
      life: 5000,
    });
  } finally {
    activatingPremium.value = false;
  }
}

async function onDeactivatePremium() {
  try {
    premium.value = await ipc.deactivatePremium();
    await refreshCapabilities();
    toast.add({
      severity: 'info',
      summary: t('settings.premiumTitle'),
      detail: t('settings.premiumDeactivated'),
      life: 4000,
    });
  } catch {
    // Best-effort; leave current state untouched on failure.
  }
}

// Voice models (Épica H). Shown inside the premium-gated "Voz" section.
const voiceStatus = ref<VoiceStatus | null>(null);
const voiceModels = ref<VoiceModel[]>([]);
const voiceVoices = ref<VoiceVoice[]>([]);
const downloadPct = ref<Record<string, number>>({});
const importingBinary = ref(false);
const importingPiper = ref(false);
let unlistenVoiceProgress: UnlistenFn | null = null;

async function loadVoice() {
  try {
    voiceStatus.value = await ipc.getVoiceStatus();
    voiceModels.value = await ipc.listVoiceModels();
    voiceVoices.value = await ipc.listVoiceVoices();
  } catch {
    voiceStatus.value = null;
    voiceModels.value = [];
    voiceVoices.value = [];
  }
}

async function onDownloadVoice(v: VoiceVoice) {
  downloadPct.value = { ...downloadPct.value, [v.id]: 0 };
  try {
    await ipc.downloadVoiceVoice(v.id);
    toast.add({
      severity: 'success',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceModelDownloaded'),
      life: 3000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceModelError'),
      life: 5000,
    });
  } finally {
    const rest = { ...downloadPct.value };
    delete rest[v.id];
    downloadPct.value = rest;
    await loadVoice();
  }
}

async function onImportPiper() {
  const picked = await open({
    multiple: false,
    directory: false,
    title: t('settings.voiceImportPiper'),
  });
  if (typeof picked !== 'string') return;
  importingPiper.value = true;
  try {
    await ipc.importPiperBinary(picked);
    toast.add({
      severity: 'success',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceBinaryImported'),
      life: 3000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceModelError'),
      life: 5000,
    });
  } finally {
    importingPiper.value = false;
    await loadVoice();
  }
}

async function onDownloadModel(m: VoiceModel) {
  downloadPct.value = { ...downloadPct.value, [m.id]: 0 };
  try {
    await ipc.downloadVoiceModel(m.id);
    toast.add({
      severity: 'success',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceModelDownloaded'),
      life: 3000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceModelError'),
      life: 5000,
    });
  } finally {
    const rest = { ...downloadPct.value };
    delete rest[m.id];
    downloadPct.value = rest;
    await loadVoice();
  }
}

async function onDeleteModel(m: VoiceModel) {
  try {
    await ipc.deleteVoiceModel(m.id);
  } catch {
    // best-effort
  }
  await loadVoice();
}

async function onImportBinary() {
  const picked = await open({
    multiple: false,
    directory: false,
    title: t('settings.voiceImportBinary'),
  });
  if (typeof picked !== 'string') return;
  importingBinary.value = true;
  try {
    await ipc.importVoiceBinary(picked);
    toast.add({
      severity: 'success',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceBinaryImported'),
      life: 3000,
    });
  } catch {
    toast.add({
      severity: 'error',
      summary: t('settings.voiceTitle'),
      detail: t('settings.voiceModelError'),
      life: 5000,
    });
  } finally {
    importingBinary.value = false;
    await loadVoice();
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

const stats = ref<WritingStats | null>(null);
const dailySeries = ref<DailyWriting[]>([]);
const dailyGoal = ref<number | null>(null);
const backups = ref<BackupRecord[]>([]);
const creatingBackup = ref(false);
const restoringId = ref<string | null>(null);
const { run } = useIpcError();

const totalWords30d = computed(() => dailySeries.value.reduce((acc, d) => acc + d.words, 0));
const activeDays30d = computed(() => dailySeries.value.filter((d) => d.sessions > 0).length);

async function onDailyGoalChange(value: number | null) {
  const goal = value && value > 0 ? Math.floor(value) : null;
  dailyGoal.value = goal;
  await run(t('settings.dailyGoalError'), () => ipc.setDailyGoal(goal));
  // Refresh the streak + series so the goal-met state reflects the new goal.
  stats.value = await ipc.getWritingStats();
  dailySeries.value = await ipc.getRecentDailyWriting(30);
}

onMounted(async () => {
  try {
    stats.value = await ipc.getWritingStats();
  } catch {
    stats.value = null;
  }
  try {
    dailySeries.value = await ipc.getRecentDailyWriting(30);
  } catch {
    dailySeries.value = [];
  }
  try {
    dailyGoal.value = await ipc.getDailyGoal();
  } catch {
    dailyGoal.value = null;
  }
  await loadBackups();
  await loadCustomFonts();
  await loadCrashReporting();
  await loadPremium();
  await loadAiStatus();
  aiUsage.rollIfNeeded();
  await loadVoice();
  unlistenVoiceProgress = await listen<VoiceDownloadProgress>('voice.download.progress', (e) => {
    const p = e.payload;
    if (p.total && p.total > 0) {
      downloadPct.value = {
        ...downloadPct.value,
        [p.modelId]: Math.round((p.downloaded / p.total) * 100),
      };
    }
  });
});

onBeforeUnmount(() => {
  unlistenVoiceProgress?.();
});

async function loadBackups() {
  const list = await run(t('settings.backupsError'), () => ipc.listBackups());
  if (list) backups.value = list;
}

async function onCreateBackup() {
  creatingBackup.value = true;
  const rec = await run(t('settings.backupsError'), () => ipc.createManualBackup());
  creatingBackup.value = false;
  if (rec) {
    backups.value = [rec, ...backups.value];
    toast.add({
      severity: 'success',
      summary: t('settings.backupsTitle'),
      detail: t('settings.backupCreated'),
      life: 3000,
    });
  }
}

async function onRestore(b: BackupRecord) {
  if (!confirm(t('settings.restoreConfirm'))) return;
  restoringId.value = b.id;
  await run(t('settings.backupsError'), () => ipc.restoreBackup(b.id));
  restoringId.value = null;
  toast.add({
    severity: 'success',
    summary: t('settings.backupsTitle'),
    detail: t('settings.restoreSuccess'),
    life: 6000,
  });
  await loadBackups();
}

function formatDate(ms: number): string {
  return new Date(ms).toLocaleString();
}

function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(2)} MB`;
}

function kindLabel(kind: BackupRecord['kind']): string {
  return t(`settings.backupKind.${kind}`);
}
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
        />
      </section>

      <section>
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
          {{ t('shortcuts.title') }}
        </h2>
        <p class="text-xs opacity-60 mb-2">{{ t('shortcuts.hint') }}</p>
        <KeybindingsEditor />
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
          <div class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('settings.goalMetStreak') }}</dt>
            <dd class="font-mono">{{ stats.goalMetStreak }}</dd>
          </div>
          <div v-if="stats.lastWritingDate" class="flex justify-between gap-2">
            <dt class="opacity-60">{{ t('settings.lastWritingDate') }}</dt>
            <dd class="font-mono">{{ stats.lastWritingDate }}</dd>
          </div>
        </dl>
        <p v-else class="text-xs opacity-60">…</p>

        <div class="mt-4 flex items-center justify-between gap-3">
          <label for="set-daily-goal" class="text-sm opacity-80">
            {{ t('settings.dailyGoal') }}
          </label>
          <InputNumber
            input-id="set-daily-goal"
            :model-value="dailyGoal"
            :min="0"
            :step="50"
            show-buttons
            :placeholder="t('settings.dailyGoalNone')"
            class="!w-40"
            @update:model-value="onDailyGoalChange"
          />
        </div>
        <p class="text-xs opacity-55 mt-1">{{ t('settings.dailyGoalHint') }}</p>

        <div class="mt-5">
          <div class="flex items-baseline justify-between mb-2 text-xs">
            <span class="opacity-70">{{ t('settings.last30Days') }}</span>
            <span class="opacity-60">
              {{ t('settings.totalWords', { count: totalWords30d }) }} ·
              {{ t('settings.activeDays', { count: activeDays30d }) }}
            </span>
          </div>
          <SparklineChart
            :series="dailySeries"
            :height="56"
            :aria-label="t('settings.last30DaysAria')"
          />
        </div>
      </section>

      <section>
        <div class="flex items-center justify-between mb-3 gap-3">
          <div>
            <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70">
              {{ t('settings.backupsTitle') }}
            </h2>
            <p class="text-xs opacity-60 mt-1">{{ t('settings.backupsHint') }}</p>
          </div>
          <Button
            :label="t('settings.backupNow')"
            icon="pi pi-database"
            size="small"
            :loading="creatingBackup"
            @click="onCreateBackup"
          />
        </div>
        <div
          v-if="backups.length === 0"
          class="text-xs opacity-60 p-3 rounded border border-surface-200 dark:border-surface-700"
        >
          {{ t('settings.backupsEmpty') }}
        </div>
        <ul
          v-else
          class="rounded border border-surface-200 dark:border-surface-700 divide-y divide-surface-200 dark:divide-surface-700"
        >
          <li
            v-for="b in backups"
            :key="b.id"
            class="flex items-center justify-between gap-3 p-3 text-sm"
          >
            <div class="flex flex-col min-w-0">
              <span class="font-mono text-xs truncate">{{ b.id }}</span>
              <span class="text-xs opacity-60">
                {{ kindLabel(b.kind) }} · {{ formatDate(b.createdAt) }} ·
                {{ formatSize(b.sizeBytes) }}
              </span>
            </div>
            <Button
              :label="t('settings.restore')"
              size="small"
              text
              :loading="restoringId === b.id"
              @click="onRestore(b)"
            />
          </li>
        </ul>
      </section>

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

      <section v-if="premium?.licensingConfigured">
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
          {{ t('settings.premiumTitle') }}
        </h2>
        <p class="text-xs opacity-60 mb-2">{{ t('settings.premiumHint') }}</p>
        <div
          v-if="premium?.active"
          class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm"
        >
          <span>
            {{ t('settings.premiumActive') }}
            <span v-if="premium.holder" class="opacity-60">· {{ premium.holder }}</span>
          </span>
          <Button
            :label="t('settings.premiumDeactivate')"
            size="small"
            text
            @click="onDeactivatePremium"
          />
        </div>
        <div v-else class="flex items-center gap-2">
          <InputText
            v-model="licenseKey"
            class="flex-1 font-mono text-xs"
            :placeholder="t('settings.premiumKeyPlaceholder')"
            :aria-label="t('settings.premiumTitle')"
          />
          <Button
            :label="t('settings.premiumActivate')"
            size="small"
            :loading="activatingPremium"
            :disabled="!licenseKey.trim()"
            @click="onActivatePremium"
          />
        </div>
      </section>

      <section v-if="aiEnabled">
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
          {{ t('settings.aiTitle') }}
        </h2>
        <p class="text-xs opacity-60 mb-2">{{ t('settings.aiKeyHint') }}</p>
        <div
          v-if="aiStatus?.hasKey"
          class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm"
        >
          <span>
            <i class="pi pi-check-circle text-green-500 mr-1" />
            {{ t('settings.aiKeySaved') }}
          </span>
          <Button
            :label="t('settings.aiKeyClear')"
            size="small"
            text
            severity="danger"
            @click="onClearOpenrouterKey"
          />
        </div>
        <div v-else class="flex items-center gap-2">
          <InputText
            v-model="openrouterKey"
            type="password"
            class="flex-1 font-mono text-xs"
            :placeholder="t('settings.aiKeyPlaceholder')"
            :aria-label="t('settings.aiKeyLabel')"
            @keydown.enter="onSaveOpenrouterKey"
          />
          <Button
            :label="t('settings.aiKeySave')"
            size="small"
            :loading="savingKey"
            :disabled="!openrouterKey.trim()"
            @click="onSaveOpenrouterKey"
          />
        </div>
        <a
          class="text-xs underline opacity-60 hover:opacity-100 mt-2 inline-block"
          href="https://openrouter.ai/keys"
          target="_blank"
          rel="noopener noreferrer"
        >
          {{ t('settings.aiKeyGetLink') }}
        </a>

        <div class="mt-4 pt-3 border-t border-surface-200 dark:border-surface-700">
          <div class="flex items-center justify-between gap-2 text-xs">
            <span class="opacity-70">
              {{
                t('settings.aiUsageThisMonth', { sent: aiUsage.sent, received: aiUsage.received })
              }}
            </span>
            <Button
              :label="t('settings.aiUsageReset')"
              size="small"
              text
              severity="secondary"
              @click="aiUsage.reset()"
            />
          </div>
          <a
            class="text-xs underline opacity-60 hover:opacity-100 mt-1 inline-block"
            href="https://openrouter.ai/activity"
            target="_blank"
            rel="noopener noreferrer"
          >
            {{ t('settings.aiUsageCostsLink') }}
          </a>
        </div>
      </section>

      <section v-if="voiceEnabled">
        <h2 class="text-sm font-semibold uppercase tracking-wide opacity-70 mb-2">
          {{ t('settings.voiceTitle') }}
        </h2>
        <p class="text-xs opacity-60 mb-2">{{ t('settings.voiceHint') }}</p>

        <!-- Whisper binary -->
        <div
          class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm mb-3"
        >
          <span>
            <i
              :class="
                voiceStatus?.binaryInstalled
                  ? 'pi pi-check-circle text-green-500'
                  : 'pi pi-exclamation-circle text-amber-500'
              "
              class="mr-1"
            />
            {{
              voiceStatus?.binaryInstalled
                ? t('settings.voiceBinaryInstalled')
                : t('settings.voiceBinaryMissing')
            }}
          </span>
          <Button
            :label="t('settings.voiceImportBinary')"
            size="small"
            text
            :loading="importingBinary"
            @click="onImportBinary"
          />
        </div>

        <!-- Models -->
        <ul
          class="rounded border border-surface-200 dark:border-surface-700 divide-y divide-surface-200 dark:divide-surface-700"
        >
          <li
            v-for="m in voiceModels"
            :key="m.id"
            class="flex items-center justify-between gap-3 p-3 text-sm"
          >
            <div class="min-w-0">
              <span class="font-medium">{{ m.id }}</span>
              <span
                v-if="m.recommended"
                class="ml-2 text-xs px-1.5 py-0.5 rounded bg-primary-100 dark:bg-primary-900/40 text-primary-700 dark:text-primary-300"
              >
                {{ t('settings.voiceRecommended') }}
              </span>
              <span class="block text-xs opacity-60">{{ `${m.sizeMb} MB` }}</span>
            </div>
            <div class="shrink-0">
              <span v-if="downloadPct[m.id] !== undefined" class="text-xs font-mono opacity-70">
                {{ `${downloadPct[m.id]}%` }}
              </span>
              <Button
                v-else-if="m.installed"
                :label="t('settings.voiceModelDelete')"
                size="small"
                text
                severity="danger"
                @click="onDeleteModel(m)"
              />
              <Button
                v-else
                :label="t('settings.voiceModelDownload')"
                icon="pi pi-download"
                size="small"
                text
                @click="onDownloadModel(m)"
              />
            </div>
          </li>
        </ul>

        <!-- Read-aloud: Piper binary + voices -->
        <h3 class="text-xs font-semibold uppercase tracking-wide opacity-60 mt-4 mb-2">
          {{ t('settings.voiceReadAloud') }}
        </h3>
        <div
          class="flex items-center justify-between gap-3 p-3 rounded border border-surface-200 dark:border-surface-700 text-sm mb-3"
        >
          <span>
            <i
              :class="
                voiceStatus?.piperInstalled
                  ? 'pi pi-check-circle text-green-500'
                  : 'pi pi-exclamation-circle text-amber-500'
              "
              class="mr-1"
            />
            {{
              voiceStatus?.piperInstalled
                ? t('settings.voicePiperInstalled')
                : t('settings.voicePiperMissing')
            }}
          </span>
          <Button
            :label="t('settings.voiceImportPiper')"
            size="small"
            text
            :loading="importingPiper"
            @click="onImportPiper"
          />
        </div>
        <ul
          class="rounded border border-surface-200 dark:border-surface-700 divide-y divide-surface-200 dark:divide-surface-700"
        >
          <li
            v-for="v in voiceVoices"
            :key="v.id"
            class="flex items-center justify-between gap-3 p-3 text-sm"
          >
            <div class="min-w-0">
              <span class="font-medium">{{ v.name }}</span>
              <span
                v-if="v.recommended"
                class="ml-2 text-xs px-1.5 py-0.5 rounded bg-primary-100 dark:bg-primary-900/40 text-primary-700 dark:text-primary-300"
              >
                {{ t('settings.voiceRecommended') }}
              </span>
              <span class="block text-xs opacity-60">{{ `${v.sizeMb} MB` }}</span>
            </div>
            <div class="shrink-0">
              <span v-if="downloadPct[v.id] !== undefined" class="text-xs font-mono opacity-70">
                {{ `${downloadPct[v.id]}%` }}
              </span>
              <span v-else-if="v.installed" class="text-xs text-green-600 dark:text-green-400">
                {{ t('settings.voiceInstalled') }}
              </span>
              <Button
                v-else
                :label="t('settings.voiceModelDownload')"
                icon="pi pi-download"
                size="small"
                text
                @click="onDownloadVoice(v)"
              />
            </div>
          </li>
        </ul>
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

    <LegalDialog v-model:visible="legalVisible" :kind="legalKind" />
  </section>
</template>
