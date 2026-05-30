<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { useCommandRegistry, filterCommands, type Command } from '@/composables/useCommandRegistry';
import { useCommandPalette } from '@/composables/useCommandPalette';
import { useKeybindingsStore, type ShortcutAction } from '@/stores/keybindings';

const { t } = useI18n();
const { commands } = useCommandRegistry();
const { visible, recentIds, close, pushRecent } = useCommandPalette();
const keybindings = useKeybindingsStore();

const query = ref('');
const activeIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);

/** Command ids whose run maps to a rebindable shortcut, so the palette can
 *  surface the *live* combo (configurable) on the right of each row. */
const COMMAND_SHORTCUTS: Record<string, ShortcutAction> = {
  'project.search': 'searchProject',
  'project.newChapter': 'newChapter',
  'project.focus': 'focusMode',
};

/** A section of rows with a heading (recents, or a command group). */
interface Section {
  key: string;
  label: string;
  items: Command[];
}

function groupByGroup(list: Command[]): Section[] {
  const out: Section[] = [];
  for (const c of list) {
    let g = out.find((x) => x.key === c.group);
    if (!g) {
      g = { key: c.group, label: c.group, items: [] };
      out.push(g);
    }
    g.items.push(c);
  }
  return out;
}

/** Last-run commands that are still registered, most-recent first. */
const recentCommands = computed<Command[]>(() => {
  const byId = new Map(commands.value.map((c) => [c.id, c]));
  return recentIds.value
    .map((id) => byId.get(id))
    .filter((c): c is Command => c !== undefined)
    .slice(0, 5);
});

/** Sections to render. Empty query → Recientes (if any) + every command
 *  grouped (nothing hidden). Non-empty → filtered, grouped. */
const sections = computed<Section[]>(() => {
  const q = query.value.trim();
  if (q) return groupByGroup(filterCommands(commands.value, q));

  const out: Section[] = [];
  if (recentCommands.value.length) {
    out.push({ key: '__recent', label: t('commandPalette.recent'), items: recentCommands.value });
  }
  const recentSet = new Set(recentCommands.value.map((c) => c.id));
  out.push(...groupByGroup(commands.value.filter((c) => !recentSet.has(c.id))));
  return out;
});

/** Flattened rows in display order — the index space for keyboard nav. */
const displayed = computed<Command[]>(() => sections.value.flatMap((s) => s.items));

function shortcutFor(id: string): string | null {
  const action = COMMAND_SHORTCUTS[id];
  if (!action) return null;
  return formatCombo(keybindings.bindings[action]);
}

/** "ctrl+shift+f" → "Ctrl ⇧ F". Honest display of the live binding. */
function formatCombo(combo: string): string {
  return combo
    .split('+')
    .map((part) => {
      if (part === 'ctrl') return 'Ctrl';
      if (part === 'shift') return '⇧';
      if (part === 'alt') return 'Alt';
      return part.length === 1 ? part.toUpperCase() : part.charAt(0).toUpperCase() + part.slice(1);
    })
    .join(' ');
}

function clampActive() {
  if (activeIndex.value >= displayed.value.length)
    activeIndex.value = Math.max(0, displayed.value.length - 1);
}

watch(query, () => {
  activeIndex.value = 0;
});

watch(visible, async (v) => {
  if (v) {
    query.value = '';
    activeIndex.value = 0;
    void keybindings.load();
    await nextTick();
    inputRef.value?.focus();
  }
});

onMounted(() => void keybindings.load());

function runAt(index: number) {
  const cmd = displayed.value[index];
  if (!cmd) return;
  pushRecent(cmd.id);
  close();
  cmd.run();
}

function onKeydown(e: KeyboardEvent) {
  if (e.key === 'ArrowDown') {
    e.preventDefault();
    if (displayed.value.length)
      activeIndex.value = (activeIndex.value + 1) % displayed.value.length;
  } else if (e.key === 'ArrowUp') {
    e.preventDefault();
    if (displayed.value.length)
      activeIndex.value = (activeIndex.value - 1 + displayed.value.length) % displayed.value.length;
  } else if (e.key === 'Enter') {
    e.preventDefault();
    runAt(activeIndex.value);
  } else if (e.key === 'Escape') {
    e.preventDefault();
    close();
  }
}

/** Flat index of a command within `displayed` (to highlight across sections). */
function flatIndex(cmd: Command): number {
  return displayed.value.indexOf(cmd);
}

watch(displayed, clampActive);
</script>

<template>
  <Dialog
    :visible="visible"
    modal
    :show-header="false"
    :dismissable-mask="true"
    :style="{ width: '40rem' }"
    :pt="{ content: { class: '!p-0' }, root: { class: '!rounded-2xl overflow-hidden' } }"
    :aria-label="t('commandPalette.open')"
    @update:visible="
      (v: boolean) => {
        if (!v) close();
      }
    "
  >
    <div
      class="flex items-center gap-3 px-5 py-4 border-b border-surface-200 dark:border-surface-700"
    >
      <i class="pi pi-search text-lg opacity-50" />
      <input
        ref="inputRef"
        v-model="query"
        class="flex-1 bg-transparent outline-none text-lg text-surface-900 dark:text-surface-50 placeholder:text-surface-400 dark:placeholder:text-surface-500"
        :placeholder="t('commandPalette.placeholder')"
        :aria-label="t('commandPalette.placeholder')"
        role="combobox"
        aria-autocomplete="list"
        aria-controls="cmd-palette-listbox"
        :aria-activedescendant="
          displayed[activeIndex] ? 'cmd-opt-' + displayed[activeIndex].id : undefined
        "
        @keydown="onKeydown"
      />
    </div>

    <div id="cmd-palette-listbox" role="listbox" class="max-h-[26rem] overflow-auto py-2">
      <p v-if="displayed.length === 0" class="px-5 py-10 text-center text-surface-500">
        {{ t('commandPalette.noResults') }}
      </p>
      <template v-for="s in sections" :key="s.key">
        <p
          class="px-5 pt-3 pb-1 text-[0.7rem] font-semibold uppercase tracking-wider text-surface-400 dark:text-surface-500"
        >
          {{ s.label }}
        </p>
        <button
          v-for="cmd in s.items"
          :id="'cmd-opt-' + cmd.id"
          :key="cmd.id"
          data-test="command-item"
          type="button"
          role="option"
          :aria-selected="flatIndex(cmd) === activeIndex"
          class="w-full flex items-center gap-3 px-5 py-2.5 text-left text-surface-800 dark:text-surface-100"
          :class="
            flatIndex(cmd) === activeIndex
              ? 'bg-primary-50 dark:bg-primary-500/10 text-primary-700 dark:text-primary-300'
              : ''
          "
          @mousemove="activeIndex = flatIndex(cmd)"
          @click="runAt(flatIndex(cmd))"
        >
          <i v-if="cmd.icon" :class="cmd.icon" class="text-base opacity-70 w-5 text-center" />
          <i v-else class="pi pi-angle-right text-base opacity-30 w-5 text-center" />
          <span class="flex-1 truncate">{{ cmd.label }}</span>
          <kbd
            v-if="shortcutFor(cmd.id)"
            class="font-sans text-[0.7rem] tracking-wide px-2 py-0.5 rounded-md bg-surface-100 dark:bg-surface-800 text-surface-500 dark:text-surface-400 border border-surface-200 dark:border-surface-700"
          >
            {{ shortcutFor(cmd.id) }}
          </kbd>
        </button>
      </template>
    </div>

    <div
      class="flex items-center gap-4 px-5 py-2.5 border-t border-surface-200 dark:border-surface-700 text-xs text-surface-500"
    >
      <span>{{ t('commandPalette.hintNavigate') }}</span>
      <span>{{ t('commandPalette.hintRun') }}</span>
      <span>{{ t('commandPalette.hintClose') }}</span>
    </div>
  </Dialog>
</template>
