<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, toRef, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import type { Editor } from '@tiptap/vue-3';
import { computePosition, offset, flip, shift } from '@floating-ui/dom';
import Button from 'primevue/button';
import InputText from 'primevue/inputtext';
import { useAiInline, type RewriteSubMode } from '@/composables/useAiInline';
import { lineDiff } from '@/composables/useTextDiff';
import { ipc } from '@/services/ipc';

const props = defineProps<{
  editor: Editor | null;
  projectId: string | null;
  docId: string | null;
  disabled?: boolean;
}>();

const { t } = useI18n();

const ai = useAiInline({
  editor: toRef(props, 'editor'),
  projectId: toRef(props, 'projectId'),
  docId: toRef(props, 'docId'),
});

// AI usability (premium + key). Fetched once; the menu never appears without
// it, so free users see nothing (no premium leakage).
const available = ref(false);
async function refreshAvailability() {
  try {
    available.value = (await ipc.getAiStatus()).available;
  } catch {
    available.value = false;
  }
}

const panelRef = ref<HTMLElement | null>(null);
const showRewriteSub = ref(false);
const customPrompt = ref('');

const REWRITE_MODES: { mode: RewriteSubMode; key: string }[] = [
  { mode: 'rephrase', key: 'ai.rewrite.rephrase' },
  { mode: 'shorter', key: 'ai.rewrite.shorter' },
  { mode: 'vivid', key: 'ai.rewrite.vivid' },
  { mode: 'show_not_tell', key: 'ai.rewrite.showNotTell' },
  { mode: 'inner_conflict', key: 'ai.rewrite.innerConflict' },
];

const diffOps = computed(() =>
  ai.showsDiff.value ? lineDiff(ai.originalText.value, ai.streamedText.value) : [],
);

const tokensLabel = computed(() => {
  const u = ai.usage.value;
  if (!u) return '';
  return t('ai.preview.tokens', { sent: u.prompt, received: u.completion });
});

// --- selection detection on the editor ---
function selectionRect(ed: Editor, from: number, to: number): DOMRect {
  const start = ed.view.coordsAtPos(from);
  const end = ed.view.coordsAtPos(to);
  const left = Math.min(start.left, end.left);
  const top = Math.min(start.top, end.top);
  const right = Math.max(start.right, end.right);
  const bottom = Math.max(start.bottom, end.bottom);
  return new DOMRect(left, top, right - left, bottom - top);
}

function onSelectionUpdate() {
  const ed = props.editor;
  if (!ed) return;
  // Freeze the panel while a generation is in flight or under review.
  if (ai.phase.value !== 'idle' && ai.phase.value !== 'menu') return;
  if (props.disabled || !available.value) {
    ai.hideMenu();
    return;
  }
  const { from, to, empty } = ed.state.selection;
  if (empty) {
    ai.hideMenu();
    showRewriteSub.value = false;
    return;
  }
  ai.showMenuAt(selectionRect(ed, from, to));
}

let attached: Editor | null = null;
function attach(ed: Editor | null) {
  if (attached) attached.off('selectionUpdate', onSelectionUpdate);
  attached = ed;
  if (ed) ed.on('selectionUpdate', onSelectionUpdate);
}
watch(() => props.editor, attach, { immediate: true });

// --- floating position ---
async function reposition() {
  const rect = ai.anchorRect.value;
  const el = panelRef.value;
  if (!rect || !el) return;
  const virtual = { getBoundingClientRect: () => rect };
  const { x, y } = await computePosition(virtual, el, {
    placement: 'top-start',
    strategy: 'fixed',
    middleware: [offset(8), flip(), shift({ padding: 8 })],
  });
  Object.assign(el.style, { left: `${x}px`, top: `${y}px` });
}
watch([() => ai.phase.value, ai.anchorRect, () => ai.streamedText.value, showRewriteSub], () => {
  if (ai.phase.value === 'idle') return;
  void nextTick().then(reposition);
});

// --- keyboard: Enter accepts in preview, Esc rejects/cancels ---
function onKeydown(e: KeyboardEvent) {
  const phase = ai.phase.value;
  if (phase === 'preview' && e.key === 'Enter' && !e.shiftKey) {
    e.preventDefault();
    ai.accept();
  } else if (
    e.key === 'Escape' &&
    (phase === 'streaming' || phase === 'preview' || phase === 'error')
  ) {
    e.preventDefault();
    ai.reject();
  }
}

function runRewrite(mode: RewriteSubMode) {
  showRewriteSub.value = false;
  void ai.run('rewrite', mode);
}
function runCustomRewrite() {
  const p = customPrompt.value.trim();
  if (!p) return;
  customPrompt.value = '';
  showRewriteSub.value = false;
  void ai.run('rewrite', 'custom', p);
}

onMounted(() => {
  void refreshAvailability();
  window.addEventListener('keydown', onKeydown);
});
onBeforeUnmount(() => {
  attach(null);
  window.removeEventListener('keydown', onKeydown);
});
</script>

<template>
  <div
    v-if="ai.phase.value !== 'idle'"
    ref="panelRef"
    class="ai-inline-panel fixed z-50 rounded-lg border border-surface-200 dark:border-surface-700 bg-surface-0 dark:bg-surface-900 shadow-lg text-sm"
    style="top: 0; left: 0"
    @mousedown.prevent
  >
    <!-- Action menu -->
    <div v-if="ai.phase.value === 'menu'" class="p-1 flex flex-col min-w-[12rem]">
      <button class="ai-menu-item" @click="ai.run('continue')">
        <i class="pi pi-forward" /> {{ t('ai.menu.continue') }}
      </button>
      <button class="ai-menu-item" @click="ai.run('expand')">
        <i class="pi pi-arrows-h" /> {{ t('ai.menu.expand') }}
      </button>
      <button class="ai-menu-item" @click="showRewriteSub = !showRewriteSub">
        <i class="pi pi-pencil" /> {{ t('ai.menu.rewrite') }}
        <i class="pi pi-angle-right ml-auto" />
      </button>
      <div
        v-if="showRewriteSub"
        class="pl-3 flex flex-col border-l border-surface-200 dark:border-surface-700 ml-2"
      >
        <button
          v-for="r in REWRITE_MODES"
          :key="r.mode"
          class="ai-menu-item"
          @click="runRewrite(r.mode)"
        >
          {{ t(r.key) }}
        </button>
        <div class="flex items-center gap-1 p-1">
          <InputText
            v-model="customPrompt"
            class="flex-1 text-xs"
            :placeholder="t('ai.rewrite.customPlaceholder')"
            @keydown.enter="runCustomRewrite"
          />
          <Button
            icon="pi pi-send"
            size="small"
            text
            :aria-label="t('ai.rewrite.custom')"
            @click="runCustomRewrite"
          />
        </div>
      </div>
      <button class="ai-menu-item" @click="ai.run('describe')">
        <i class="pi pi-eye" /> {{ t('ai.menu.describe') }}
      </button>
    </div>

    <!-- Streaming / preview / error -->
    <div v-else class="p-3 w-[26rem] max-w-[90vw]">
      <div class="flex items-center gap-2 mb-2 text-xs opacity-70">
        <i v-if="ai.isBusy.value" class="pi pi-spin pi-spinner" />
        <i v-else-if="ai.phase.value === 'error'" class="pi pi-exclamation-triangle text-red-500" />
        <i v-else class="pi pi-sparkles text-primary-500" />
        <span>{{ ai.isBusy.value ? t('ai.preview.streaming') : t('ai.preview.ready') }}</span>
        <span v-if="tokensLabel" class="ml-auto font-mono">{{ tokensLabel }}</span>
      </div>

      <p v-if="ai.phase.value === 'error'" class="text-sm text-red-600 dark:text-red-400">
        {{ ai.errorMsg.value }}
      </p>

      <!-- Diff view for rewrite/expand -->
      <div
        v-else-if="ai.showsDiff.value"
        class="max-h-64 overflow-auto rounded border border-surface-200 dark:border-surface-700 p-2 font-serif leading-relaxed whitespace-pre-wrap"
      >
        <template v-for="(op, i) in diffOps" :key="i">
          <div v-if="op.kind === 'remove'" class="ai-diff-remove">{{ op.before }}</div>
          <div v-else-if="op.kind === 'add'" class="ai-diff-add">{{ op.after }}</div>
          <div v-else>{{ op.after }}</div>
        </template>
      </div>

      <!-- Plain streamed text for continue/describe -->
      <div
        v-else
        class="max-h-64 overflow-auto rounded border border-surface-200 dark:border-surface-700 p-2 font-serif leading-relaxed whitespace-pre-wrap"
      >
        {{ ai.streamedText.value || '…' }}
      </div>

      <div class="flex items-center justify-end gap-2 mt-3">
        <template v-if="ai.phase.value === 'preview'">
          <Button
            :label="t('ai.preview.reject')"
            size="small"
            text
            severity="secondary"
            @click="ai.reject()"
          />
          <Button
            :label="t('ai.preview.accept')"
            size="small"
            icon="pi pi-check"
            @click="ai.accept()"
          />
        </template>
        <Button
          v-else
          :label="t('ai.preview.cancel')"
          size="small"
          text
          severity="secondary"
          @click="ai.reject()"
        />
      </div>
    </div>
  </div>
</template>

<style scoped>
.ai-menu-item {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  width: 100%;
  text-align: left;
  padding: 0.4rem 0.6rem;
  border-radius: 0.375rem;
  font-size: 0.875rem;
}
.ai-menu-item:hover {
  background: var(--p-surface-100, #f1f5f9);
}
:global(.app-dark) .ai-menu-item:hover {
  background: var(--p-surface-800, #1e293b);
}
.ai-diff-add {
  background: color-mix(in srgb, var(--p-green-500, #22c55e) 18%, transparent);
}
.ai-diff-remove {
  background: color-mix(in srgb, var(--p-red-500, #ef4444) 18%, transparent);
  text-decoration: line-through;
  opacity: 0.7;
}
</style>
