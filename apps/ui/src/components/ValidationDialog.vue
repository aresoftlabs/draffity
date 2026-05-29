<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Button from 'primevue/button';
import Checkbox from 'primevue/checkbox';
import { useIpcError } from '@/composables/useIpcError';
import {
  ipc,
  type AiValidation,
  type CoverageReport,
  type ValidationFinding,
} from '@/services/ipc';

const props = defineProps<{
  visible: boolean;
  projectId: string;
  documentId: string | null;
}>();
const emit = defineEmits<{ 'update:visible': [boolean]; locate: [string] }>();

const { t } = useI18n();
const { run: ipcRun } = useIpcError();

const VALIDATORS = ['character', 'voice', 'repetition', 'plot', 'style'] as const;
const STORAGE_KEY = 'draffity.validators.selected';

function loadSelected(): string[] {
  try {
    const r = localStorage.getItem(STORAGE_KEY);
    if (r) return JSON.parse(r) as string[];
  } catch {
    // ignore
  }
  return [...VALIDATORS];
}

const selected = ref<string[]>(loadSelected());
watch(
  selected,
  (v) => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(v));
    } catch {
      // ignore
    }
  },
  { deep: true },
);

const coverage = ref<CoverageReport | null>(null);
const reports = ref<AiValidation[]>([]);
const running = ref(false);

const visibleModel = computed({
  get: () => props.visible,
  set: (v: boolean) => emit('update:visible', v),
});

const isSparse = computed(() => {
  const c = coverage.value;
  return !!c && c.candidates >= 3 && c.covered / c.candidates < 0.5;
});

/** Latest report per validator, with its findings parsed. */
const latest = computed(() => {
  const seen = new Set<string>();
  const out: { name: string; summary: string; findings: ValidationFinding[] }[] = [];
  for (const r of reports.value) {
    if (seen.has(r.validatorName)) continue;
    seen.add(r.validatorName);
    let findings: ValidationFinding[] = [];
    try {
      findings = JSON.parse(r.resultsJson) as ValidationFinding[];
    } catch {
      findings = [];
    }
    out.push({ name: r.validatorName, summary: r.severitySummary, findings });
  }
  return out;
});

async function load() {
  if (!props.documentId) return;
  coverage.value =
    (await ipcRun(t('ai.validators.error'), () =>
      ipc.checkCodexCoverage(props.projectId, props.documentId!),
    )) ?? null;
  reports.value =
    (await ipcRun(t('ai.validators.error'), () => ipc.listValidations(props.documentId!))) ?? [];
}

watch(
  () => props.visible,
  (v) => {
    if (v) void load();
  },
);

async function onRun() {
  if (!props.documentId || selected.value.length === 0) return;
  running.value = true;
  const res = await ipcRun(t('ai.validators.error'), () =>
    ipc.runValidators(props.projectId, props.documentId!, selected.value),
  );
  running.value = false;
  if (res) await load();
}

function severityDot(s: string): string {
  if (s === 'critical') return 'bg-red-500';
  if (s === 'warning') return 'bg-amber-500';
  return 'bg-sky-500';
}
</script>

<template>
  <Dialog
    v-model:visible="visibleModel"
    modal
    :header="t('ai.validators.title')"
    :style="{ width: '40rem', maxWidth: '95vw' }"
  >
    <div class="space-y-4">
      <!-- Codex coverage pre-check (G-03) -->
      <div
        v-if="isSparse"
        class="text-xs p-2 rounded bg-amber-100 dark:bg-amber-900/30 text-amber-900 dark:text-amber-200"
      >
        {{
          t('ai.validators.coverageWarning', {
            covered: coverage?.covered ?? 0,
            total: coverage?.candidates ?? 0,
          })
        }}
      </div>

      <!-- Validator toggles (G-10) -->
      <div class="flex flex-wrap gap-3">
        <div v-for="v in VALIDATORS" :key="v" class="flex items-center gap-1.5">
          <Checkbox v-model="selected" :input-id="`val-${v}`" :value="v" />
          <label :for="`val-${v}`" class="text-sm cursor-pointer">{{
            t(`ai.validators.${v}`)
          }}</label>
        </div>
      </div>

      <div class="flex justify-end">
        <Button
          :label="t('ai.validators.run')"
          icon="pi pi-sparkles"
          size="small"
          :loading="running"
          :disabled="running || selected.length === 0 || !documentId"
          @click="onRun"
        />
      </div>

      <!-- Reports (G-09) -->
      <div v-if="latest.length === 0" class="text-sm opacity-60 text-center py-4">
        {{ t('ai.validators.empty') }}
      </div>
      <div v-else class="space-y-4 max-h-[50vh] overflow-auto">
        <section v-for="group in latest" :key="group.name">
          <h3 class="text-xs font-semibold uppercase tracking-wide opacity-70 mb-1">
            {{ t(`ai.validators.${group.name}`) }} ·
            <span class="font-normal normal-case opacity-80">{{ group.summary }}</span>
          </h3>
          <ul class="space-y-2">
            <li
              v-for="(f, i) in group.findings"
              :key="i"
              class="text-sm rounded border border-surface-200 dark:border-surface-700 p-2"
            >
              <div class="flex items-start gap-2">
                <span class="mt-1 w-2 h-2 rounded-full shrink-0" :class="severityDot(f.severity)" />
                <div class="min-w-0 flex-1">
                  <p class="font-medium">{{ f.title }}</p>
                  <p class="opacity-80">{{ f.detail }}</p>
                  <p
                    v-if="f.excerpt"
                    class="mt-1 text-xs font-mono opacity-70 bg-surface-100 dark:bg-surface-800 rounded px-1.5 py-0.5 break-words"
                  >
                    {{ `«${f.excerpt}»` }}
                  </p>
                  <p v-if="f.suggestion" class="mt-1 text-xs italic opacity-80">
                    {{ `→ ${f.suggestion}` }}
                  </p>
                </div>
                <Button
                  v-if="f.excerpt"
                  v-tooltip.left="t('ai.validators.locate')"
                  icon="pi pi-arrow-right"
                  text
                  size="small"
                  :aria-label="t('ai.validators.locate')"
                  @click="emit('locate', f.excerpt)"
                />
              </div>
            </li>
          </ul>
        </section>
      </div>
    </div>
  </Dialog>
</template>
