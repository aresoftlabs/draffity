<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Stepper from 'primevue/stepper';
import StepList from 'primevue/steplist';
import StepPanels from 'primevue/steppanels';
import Step from 'primevue/step';
import StepPanel from 'primevue/steppanel';
import InputText from 'primevue/inputtext';
import InputNumber from 'primevue/inputnumber';
import Textarea from 'primevue/textarea';
import DatePicker from 'primevue/datepicker';
import Button from 'primevue/button';
import Message from 'primevue/message';
import Tag from 'primevue/tag';
import type { MetadataField, ProjectInput, Template, TemplateNode } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';

const props = defineProps<{
  visible: boolean;
}>();

const emit = defineEmits<{
  'update:visible': [value: boolean];
  submit: [input: ProjectInput];
}>();

const { t } = useI18n();
const { run } = useIpcError();

const templates = ref<Template[]>([]);
const loading = ref(false);
const step = ref(1);

const selectedTemplate = ref<Template | null>(null);
const projectTitle = ref('');
const metadataValues = ref<Record<string, unknown>>({});

watch(
  () => props.visible,
  (v) => {
    if (v) {
      step.value = 1;
      selectedTemplate.value = null;
      projectTitle.value = '';
      metadataValues.value = {};
      void load();
    }
  },
);

onMounted(() => {
  if (props.visible) void load();
});

async function load() {
  loading.value = true;
  const list = await run(t('errors.loadProjects'), () => ipc.listTemplates());
  templates.value = list ?? [];
  loading.value = false;
}

function pickTemplate(tpl: Template) {
  selectedTemplate.value = tpl;
  // Seed defaults
  metadataValues.value = {};
  for (const f of tpl.metadataFields) {
    if (f.default !== undefined && f.default !== null) {
      metadataValues.value[f.key] = f.default;
    }
  }
}

function nextStep() {
  if (step.value < 3) step.value += 1;
}

function prevStep() {
  if (step.value > 1) step.value -= 1;
}

const canAdvanceFromStep1 = computed(() => selectedTemplate.value !== null);

const requiredMetadataMissing = computed(() => {
  if (!selectedTemplate.value) return false;
  return selectedTemplate.value.metadataFields.some((f) => {
    if (!f.required) return false;
    const v = metadataValues.value[f.key];
    if (v === undefined || v === null) return true;
    if (typeof v === 'string' && v.trim() === '') return true;
    return false;
  });
});

const canAdvanceFromStep2 = computed(
  () => projectTitle.value.trim().length > 0 && !requiredMetadataMissing.value,
);

function close() {
  emit('update:visible', false);
}

function submit() {
  if (!selectedTemplate.value) return;
  if (!canAdvanceFromStep2.value) return;
  const cleaned = cleanMetadata(selectedTemplate.value.metadataFields, metadataValues.value);
  emit('submit', {
    title: projectTitle.value.trim(),
    templateId: selectedTemplate.value.id,
    metadata: Object.keys(cleaned).length ? cleaned : null,
  });
  close();
}

function cleanMetadata(
  fields: MetadataField[],
  values: Record<string, unknown>,
): Record<string, unknown> {
  const out: Record<string, unknown> = {};
  for (const f of fields) {
    const v = values[f.key];
    if (v === undefined || v === null) continue;
    if (typeof v === 'string' && v.trim() === '') continue;
    out[f.key] = v;
  }
  return out;
}

function flattenTree(nodes: TemplateNode[]): { title: string; level: number; type: string }[] {
  const out: { title: string; level: number; type: string }[] = [];
  function walk(items: TemplateNode[], level: number) {
    for (const n of items) {
      out.push({ title: n.title, level, type: n.docType });
      if (n.children?.length) walk(n.children, level + 1);
    }
  }
  walk(nodes, 0);
  return out;
}

const previewNodes = computed(() =>
  selectedTemplate.value ? flattenTree(selectedTemplate.value.structure) : [],
);

function templateBadge(tpl: Template): string {
  return t(`newProject.kind.${tpl.kind}`);
}

function iconForDocType(type: string) {
  switch (type) {
    case 'chapter':
      return 'pi pi-book';
    case 'scene':
      return 'pi pi-bookmark';
    case 'note':
      return 'pi pi-sticky-note';
    case 'folder':
      return 'pi pi-folder';
    case 'manga_page':
      return 'pi pi-image';
    default:
      return 'pi pi-file';
  }
}
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :header="t('newProject.title')"
    :style="{ width: '52rem' }"
    :closable="true"
    @update:visible="(v: boolean) => emit('update:visible', v)"
  >
    <Stepper v-model:value="step" linear>
      <StepList>
        <Step :value="1">{{ t('newProject.step1') }}</Step>
        <Step :value="2">{{ t('newProject.step2') }}</Step>
        <Step :value="3">{{ t('newProject.step3') }}</Step>
      </StepList>

      <StepPanels>
        <!-- Step 1: choose template -->
        <StepPanel :value="1">
          <div class="flex flex-col gap-3 min-h-[20rem]">
            <p v-if="loading" class="text-sm opacity-60">…</p>
            <div v-else class="grid grid-cols-1 md:grid-cols-2 gap-3">
              <button
                v-for="tpl in templates"
                :key="tpl.id"
                type="button"
                class="text-left p-4 rounded-lg border transition-shadow hover:shadow-sm"
                :class="
                  selectedTemplate?.id === tpl.id
                    ? 'border-primary-500 ring-2 ring-primary-200 dark:ring-primary-900 bg-primary-50/40 dark:bg-primary-900/10'
                    : 'border-surface-200 dark:border-surface-700'
                "
                @click="pickTemplate(tpl)"
              >
                <div class="flex items-start justify-between gap-2 mb-2">
                  <h3 class="font-serif font-semibold text-base">{{ tpl.name }}</h3>
                  <Tag :value="templateBadge(tpl)" severity="info" />
                </div>
                <p v-if="tpl.description" class="text-sm opacity-70 leading-relaxed">
                  {{ tpl.description }}
                </p>
              </button>
            </div>

            <div v-if="selectedTemplate" class="mt-4">
              <h4 class="text-xs font-semibold uppercase tracking-wide opacity-60 mb-2">
                {{ t('newProject.preview') }}
              </h4>
              <ul
                class="text-sm border border-surface-200 dark:border-surface-700 rounded-md p-3 bg-surface-50 dark:bg-surface-900 max-h-48 overflow-auto space-y-1"
              >
                <li
                  v-for="(n, i) in previewNodes"
                  :key="i"
                  class="flex items-center gap-2"
                  :style="{ paddingLeft: `${n.level * 16}px` }"
                >
                  <i :class="iconForDocType(n.type)" class="text-xs opacity-60" />
                  <span>{{ n.title }}</span>
                </li>
                <li v-if="previewNodes.length === 0" class="opacity-60">
                  {{ t('newProject.previewEmpty') }}
                </li>
              </ul>
            </div>
          </div>

          <div class="flex justify-end gap-2 mt-6">
            <Button :label="t('actions.cancel')" text severity="secondary" @click="close" />
            <Button
              :label="t('newProject.next')"
              icon="pi pi-arrow-right"
              icon-pos="right"
              :disabled="!canAdvanceFromStep1"
              @click="nextStep"
            />
          </div>
        </StepPanel>

        <!-- Step 2: metadata -->
        <StepPanel :value="2">
          <div class="flex flex-col gap-4 min-h-[20rem]">
            <div class="flex flex-col gap-1">
              <label for="np-title" class="text-sm font-medium">
                {{ t('newProject.name') }}
                <span class="text-red-500">*</span>
              </label>
              <InputText
                id="np-title"
                v-model="projectTitle"
                :placeholder="t('newProject.namePlaceholder')"
                autofocus
              />
            </div>

            <div
              v-for="f in selectedTemplate?.metadataFields ?? []"
              :key="f.key"
              class="flex flex-col gap-1"
            >
              <label :for="`np-meta-${f.key}`" class="text-sm font-medium">
                {{ f.label }}
                <span v-if="f.required" class="text-red-500">*</span>
              </label>
              <InputText
                v-if="f.type === 'string'"
                :id="`np-meta-${f.key}`"
                :model-value="(metadataValues[f.key] as string) ?? ''"
                @update:model-value="(v) => (metadataValues[f.key] = v)"
              />
              <Textarea
                v-else-if="f.type === 'text'"
                :id="`np-meta-${f.key}`"
                :model-value="(metadataValues[f.key] as string) ?? ''"
                rows="3"
                auto-resize
                @update:model-value="(v) => (metadataValues[f.key] = v)"
              />
              <InputNumber
                v-else-if="f.type === 'number'"
                :id="`np-meta-${f.key}`"
                :model-value="(metadataValues[f.key] as number) ?? null"
                @update:model-value="(v) => (metadataValues[f.key] = v)"
              />
              <DatePicker
                v-else-if="f.type === 'date'"
                :id="`np-meta-${f.key}`"
                :model-value="(metadataValues[f.key] as Date) ?? null"
                show-icon
                @update:model-value="(v) => (metadataValues[f.key] = v)"
              />
            </div>

            <Message v-if="requiredMetadataMissing" severity="warn" :closable="false">
              {{ t('newProject.fillRequired') }}
            </Message>
          </div>

          <div class="flex justify-between gap-2 mt-6">
            <Button
              :label="t('newProject.back')"
              icon="pi pi-arrow-left"
              text
              severity="secondary"
              @click="prevStep"
            />
            <div class="flex gap-2">
              <Button :label="t('actions.cancel')" text severity="secondary" @click="close" />
              <Button
                :label="t('newProject.next')"
                icon="pi pi-arrow-right"
                icon-pos="right"
                :disabled="!canAdvanceFromStep2"
                @click="nextStep"
              />
            </div>
          </div>
        </StepPanel>

        <!-- Step 3: confirm -->
        <StepPanel :value="3">
          <div class="flex flex-col gap-3 min-h-[20rem]">
            <h3 class="text-sm font-semibold uppercase tracking-wide opacity-60">
              {{ t('newProject.summary') }}
            </h3>
            <dl class="text-sm space-y-1">
              <div class="flex justify-between gap-2">
                <dt class="opacity-60">{{ t('newProject.name') }}</dt>
                <dd class="font-medium">{{ projectTitle.trim() }}</dd>
              </div>
              <div class="flex justify-between gap-2">
                <dt class="opacity-60">{{ t('newProject.template') }}</dt>
                <dd class="font-medium">{{ selectedTemplate?.name }}</dd>
              </div>
              <div
                v-for="f in selectedTemplate?.metadataFields ?? []"
                :key="f.key"
                class="flex justify-between gap-2"
              >
                <dt class="opacity-60">{{ f.label }}</dt>
                <dd>{{ metadataValues[f.key] || '—' }}</dd>
              </div>
            </dl>

            <p class="text-xs opacity-70 italic mt-2">
              {{
                t('newProject.willCreate', {
                  count: previewNodes.length,
                })
              }}
            </p>
          </div>

          <div class="flex justify-between gap-2 mt-6">
            <Button
              :label="t('newProject.back')"
              icon="pi pi-arrow-left"
              text
              severity="secondary"
              @click="prevStep"
            />
            <div class="flex gap-2">
              <Button :label="t('actions.cancel')" text severity="secondary" @click="close" />
              <Button :label="t('actions.create')" icon="pi pi-check" @click="submit" />
            </div>
          </div>
        </StepPanel>
      </StepPanels>
    </Stepper>
  </Dialog>
</template>
