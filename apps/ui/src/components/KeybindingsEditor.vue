<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import Button from 'primevue/button';
import {
  formatCombo,
  SHORTCUT_ACTIONS,
  useKeybindingsStore,
  type ShortcutAction,
} from '@/stores/keybindings';

const { t } = useI18n();
const store = useKeybindingsStore();

const capturing = ref<ShortcutAction | null>(null);

onMounted(() => void store.load());

function startCapture(action: ShortcutAction) {
  capturing.value = action;
}

function cancelCapture() {
  capturing.value = null;
}

/** Listens for the next non-modifier key press while `capturing` is set,
 *  saves it as the new combo and exits capture mode. */
function onCaptureKey(e: KeyboardEvent) {
  if (!capturing.value) return;
  e.preventDefault();
  e.stopPropagation();
  const combo = formatCombo(e);
  // Reject modifier-only events (formatCombo returns "ctrl" alone).
  if (combo === 'ctrl' || combo === 'shift' || combo === 'alt' || combo === '') return;
  if (combo === 'escape') {
    cancelCapture();
    return;
  }
  void store.set(capturing.value, combo);
  capturing.value = null;
}

async function onReset(action: ShortcutAction) {
  await store.reset(action);
}
</script>

<template>
  <div class="flex flex-col gap-1" tabindex="0" @keydown.capture="onCaptureKey">
    <div
      v-for="action in SHORTCUT_ACTIONS"
      :key="action"
      class="flex items-center justify-between gap-3 py-2 border-b border-surface-100 dark:border-surface-800 last:border-b-0"
    >
      <div class="text-sm">{{ t(`shortcuts.${action}`) }}</div>
      <div class="flex items-center gap-2">
        <code
          class="font-mono text-xs px-2 py-1 rounded bg-surface-100 dark:bg-surface-800 min-w-24 text-center"
        >
          {{ store.bindings[action] }}
        </code>
        <Button
          v-if="capturing !== action"
          :label="t('shortcuts.rebind')"
          size="small"
          text
          @click="startCapture(action)"
        />
        <span v-else class="text-xs italic opacity-70 px-2">
          {{ t('shortcuts.capturing') }}
        </span>
        <Button
          :label="t('shortcuts.reset')"
          size="small"
          text
          severity="secondary"
          @click="onReset(action)"
        />
      </div>
    </div>
  </div>
</template>
