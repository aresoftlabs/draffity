<script setup lang="ts">
import { computed, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import type { CatalogLang } from '@/services/ipc';

const props = defineProps<{
  catalog: CatalogLang[];
  downloadPct: Record<string, number>;
  testingVoiceId?: string | null;
}>();

const emit = defineEmits<{
  download: [id: string];
  delete: [id: string];
  test: [id: string];
}>();

const { t } = useI18n();

const query = ref('');
const showAll = ref(false);

const STAR = '★';
const GLOBE = '🌐';
const chevron = computed(() => (showAll.value ? '▾' : '▸'));

function voiceMeta(quality: string, sizeMb: number): string {
  return `· ${quality} · ${sizeMb} MB`;
}

const filtered = computed<CatalogLang[]>(() => {
  const q = query.value.trim().toLowerCase();
  if (!q) return props.catalog;
  return props.catalog
    .map((g) => ({
      ...g,
      voices: g.voices.filter(
        (v) => v.name.toLowerCase().includes(q) || g.langName.toLowerCase().includes(q),
      ),
    }))
    .filter((g) => g.voices.length > 0);
});

const featured = computed(() => filtered.value.filter((g) => g.featured));
const others = computed(() => filtered.value.filter((g) => !g.featured));
const otherCount = computed(() => others.value.reduce((n, g) => n + g.voices.length, 0));
</script>

<template>
  <div>
    <InputText
      v-model="query"
      class="w-full mb-3"
      :placeholder="t('settings.voiceCatalogSearch')"
      :aria-label="t('settings.voiceCatalogSearch')"
    />

    <template v-for="group in featured" :key="group.lang">
      <h4 class="text-xs font-semibold opacity-70 mt-3 mb-1">
        {{ STAR }} {{ group.langName }} <span class="opacity-50">· {{ group.voices.length }}</span>
      </h4>
      <div
        v-for="v in group.voices"
        :key="v.id"
        class="flex items-center justify-between gap-2 py-1.5 text-sm border-t border-surface-200 dark:border-surface-700 min-w-0"
      >
        <span class="min-w-0 truncate">
          {{ v.name }}
          <span class="text-xs opacity-50">{{ voiceMeta(v.quality, v.sizeMb) }}</span>
        </span>
        <div class="shrink-0 flex items-center gap-1">
          <template v-if="downloadPct[v.id] !== undefined">
            <span class="text-xs font-mono opacity-70">{{ downloadPct[v.id] }}%</span>
          </template>
          <template v-else-if="v.installed">
            <Button
              :label="t('settings.voiceTestPlay')"
              size="small"
              text
              icon="pi pi-play"
              :loading="testingVoiceId === v.id"
              @click="emit('test', v.id)"
            />
            <Button
              icon="pi pi-trash"
              size="small"
              text
              severity="danger"
              :aria-label="t('settings.voiceDelete')"
              :data-test="`delete-${v.id}`"
              @click="emit('delete', v.id)"
            />
          </template>
          <Button
            v-else
            :label="t('settings.voiceModelDownload')"
            size="small"
            text
            icon="pi pi-download"
            :data-test="`download-${v.id}`"
            @click="emit('download', v.id)"
          />
        </div>
      </div>
    </template>

    <div v-if="others.length" class="mt-3">
      <button class="text-xs opacity-70 hover:opacity-100" @click="showAll = !showAll">
        {{ GLOBE }} {{ t('settings.voiceCatalogAllLangs', { count: String(otherCount) }) }}
        {{ chevron }}
      </button>
      <template v-for="group in others" :key="group.lang">
        <h4 class="text-xs font-semibold opacity-70 mt-3 mb-1">
          {{ group.langName }} <span class="opacity-50">· {{ group.voices.length }}</span>
        </h4>
        <template v-if="showAll || query">
          <div
            v-for="v in group.voices"
            :key="v.id"
            class="flex items-center justify-between gap-2 py-1.5 text-sm border-t border-surface-200 dark:border-surface-700 min-w-0"
          >
            <span class="min-w-0 truncate">
              {{ v.name }}
              <span class="text-xs opacity-50">{{ voiceMeta(v.quality, v.sizeMb) }}</span>
            </span>
            <div class="shrink-0 flex items-center gap-1">
              <span v-if="downloadPct[v.id] !== undefined" class="text-xs font-mono opacity-70"
                >{{ downloadPct[v.id] }}%</span
              >
              <template v-else-if="v.installed">
                <Button
                  :label="t('settings.voiceTestPlay')"
                  size="small"
                  text
                  icon="pi pi-play"
                  :loading="testingVoiceId === v.id"
                  @click="emit('test', v.id)"
                />
                <Button
                  icon="pi pi-trash"
                  size="small"
                  text
                  severity="danger"
                  :aria-label="t('settings.voiceDelete')"
                  :data-test="`delete-${v.id}`"
                  @click="emit('delete', v.id)"
                />
              </template>
              <Button
                v-else
                :label="t('settings.voiceModelDownload')"
                size="small"
                text
                icon="pi pi-download"
                :data-test="`download-${v.id}`"
                @click="emit('download', v.id)"
              />
            </div>
          </div>
        </template>
      </template>
    </div>
  </div>
</template>
