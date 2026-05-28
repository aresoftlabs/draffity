<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import Dialog from 'primevue/dialog';
import Stepper from 'primevue/stepper';
import StepList from 'primevue/steplist';
import StepPanels from 'primevue/steppanels';
import Step from 'primevue/step';
import StepPanel from 'primevue/steppanel';
import Button from 'primevue/button';
import type { MetadataField, ProjectInput, Template, TemplateNode } from '@draffity/shared-types';
import { ipc } from '@/services/ipc';
import { useIpcError } from '@/composables/useIpcError';
import WizardStepTemplate from '@/components/WizardStepTemplate.vue';
import WizardStepMetadata from '@/components/WizardStepMetadata.vue';
import WizardStepConfirm from '@/components/WizardStepConfirm.vue';

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
      reset();
      void load();
    }
  },
);

onMounted(() => {
  if (props.visible) void load();
});

function reset() {
  step.value = 1;
  selectedTemplate.value = null;
  projectTitle.value = '';
  metadataValues.value = {};
}

async function load() {
  loading.value = true;
  const list = await run(t('errors.loadProjects'), () => ipc.listTemplates());
  templates.value = list ?? [];
  loading.value = false;
}

function pickTemplate(tpl: Template) {
  selectedTemplate.value = tpl;
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

const canAdvanceFromStep2 = computed(() => {
  if (projectTitle.value.trim().length === 0) return false;
  if (!selectedTemplate.value) return false;
  return !hasMissingRequired(selectedTemplate.value.metadataFields, metadataValues.value);
});

const previewNodeCount = computed(() =>
  selectedTemplate.value ? countNodes(selectedTemplate.value.structure) : 0,
);

function close() {
  emit('update:visible', false);
}

function submit() {
  if (!selectedTemplate.value || !canAdvanceFromStep2.value) return;
  const cleaned = cleanMetadata(selectedTemplate.value.metadataFields, metadataValues.value);
  emit('submit', {
    title: projectTitle.value.trim(),
    templateId: selectedTemplate.value.id,
    metadata: Object.keys(cleaned).length ? cleaned : null,
  });
  close();
}

function hasMissingRequired(fields: MetadataField[], values: Record<string, unknown>): boolean {
  return fields.some((f) => {
    if (!f.required) return false;
    const v = values[f.key];
    if (v === undefined || v === null) return true;
    if (typeof v === 'string' && v.trim() === '') return true;
    return false;
  });
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

function countNodes(nodes: TemplateNode[]): number {
  let n = 0;
  for (const node of nodes) {
    n += 1;
    if (node.children?.length) n += countNodes(node.children);
  }
  return n;
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
        <StepPanel :value="1">
          <WizardStepTemplate
            :templates="templates"
            :selected-id="selectedTemplate?.id ?? null"
            :loading="loading"
            @select="pickTemplate"
          />
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

        <StepPanel :value="2">
          <WizardStepMetadata
            :template="selectedTemplate"
            :title="projectTitle"
            :values="metadataValues"
            @update:title="(v: string) => (projectTitle = v)"
            @update:values="(v: Record<string, unknown>) => (metadataValues = v)"
          />
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

        <StepPanel :value="3">
          <WizardStepConfirm
            :template="selectedTemplate"
            :title="projectTitle"
            :values="metadataValues"
            :node-count="previewNodeCount"
          />
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
