<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useCommandRegistry, filterCommands, type Command } from '@/composables/useCommandRegistry';
import { useCommandPalette } from '@/composables/useCommandPalette';

const { t } = useI18n();
const { commands } = useCommandRegistry();
const { visible, close } = useCommandPalette();

const query = ref('');
const activeIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

const results = computed<Command[]>(() => filterCommands(commands.value, query.value));

/** Grupos en orden de primera aparición, para encabezados en la lista. */
const groups = computed<{ group: string; items: Command[] }[]>(() => {
  const out: { group: string; items: Command[] }[] = [];
  for (const c of results.value) {
    let g = out.find((x) => x.group === c.group);
    if (!g) {
      g = { group: c.group, items: [] };
      out.push(g);
    }
    g.items.push(c);
  }
  return out;
});

function clampActive() {
  if (activeIndex.value >= results.value.length)
    activeIndex.value = Math.max(0, results.value.length - 1);
}

watch(query, () => {
  activeIndex.value = 0;
});

watch(visible, async (v) => {
  if (v) {
    query.value = '';
    activeIndex.value = 0;
    await nextTick();
    inputRef.value?.focus();
  }
});

function runAt(index: number) {
  const cmd = results.value[index];
  if (!cmd) return;
  close();
  cmd.run();
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    if (results.value.length) activeIndex.value = (activeIndex.value + 1) % results.value.length;
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    if (results.value.length)
      activeIndex.value = (activeIndex.value - 1 + results.value.length) % results.value.length;
  } else if (e.key === 'Enter') {
    e.preventDefault();
    runAt(activeIndex.value);
  } else if (e.key === 'Escape') {
    e.preventDefault();
    close();
  }
}

/** Índice plano de un comando dentro de `results` (para resaltar a través de grupos). */
function flatIndex(cmd: Command): number {
  return results.value.indexOf(cmd);
}

watch(results, clampActive);
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :show-header="false"
    :dismissable-mask="true"
    :style="{ width: '40rem' }"
    :pt="{ content: { class: '!p-0' } }"
    :aria-label="t('commandPalette.open')"
    @update:visible="
      (v: boolean) => {
        if (!v) close();
      }
    "
  >
    <div
      class="flex items-center gap-2 px-4 py-3 border-b border-surface-200 dark:border-surface-700"
    >
      <i class="pi pi-search opacity-60" />
      <input
        ref="inputRef"
        v-model="query"
        class="flex-1 bg-transparent outline-none text-surface-900 dark:text-surface-50"
        :placeholder="t('commandPalette.placeholder')"
        :aria-label="t('commandPalette.placeholder')"
        @keydown="onKeydown"
      />
    </div>

    <div class="max-h-[24rem] overflow-auto py-2">
      <p v-if="results.length === 0" class="px-4 py-6 text-center text-surface-500">
        {{ t('commandPalette.noResults') }}
      </p>
      <template v-for="g in groups" :key="g.group">
        <p class="px-4 pt-2 pb-1 text-xs uppercase tracking-wide text-surface-500">{{ g.group }}</p>
        <button
          v-for="cmd in g.items"
          :key="cmd.id"
          data-test="command-item"
          type="button"
          class="w-full flex items-center gap-3 px-4 py-2 text-left text-surface-800 dark:text-surface-100"
          :class="flatIndex(cmd) === activeIndex ? 'bg-surface-100 dark:bg-surface-800' : ''"
          @mousemove="activeIndex = flatIndex(cmd)"
          @click="runAt(flatIndex(cmd))"
        >
          <i v-if="cmd.icon" :class="cmd.icon" class="opacity-70" />
          <span class="flex-1">{{ cmd.label }}</span>
        </button>
      </template>
    </div>

    <div
      class="flex items-center gap-4 px-4 py-2 border-t border-surface-200 dark:border-surface-700 text-xs text-surface-500"
    >
      <span>{{ t('commandPalette.hintNavigate') }}</span>
      <span>{{ t('commandPalette.hintRun') }}</span>
      <span>{{ t('commandPalette.hintClose') }}</span>
    </div>
  </Dialog>
</template>
