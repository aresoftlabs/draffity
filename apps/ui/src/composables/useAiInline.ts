import { computed, onMounted, onUnmounted, ref, type Ref } from 'vue';
import type { Editor } from '@tiptap/vue-3';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { ipc, type AiActionRequestInput, type AiDeltaEvent } from '@/services/ipc';
import { useAiUsageStore } from '@/stores/aiUsage';

/**
 * Inline AI orchestration (F-06/F-07 + F-08..F-11). Owns the state machine for
 * one in-flight action: gather selection/context → stream the result live →
 * preview with accept/reject. The floating panel renders this state; the
 * editor is only mutated on `accept()`.
 *
 * Streaming arrives as `ai.suggestion.received` events tagged with the request
 * id (the backend runs the blocking HTTP off-thread). `reject()` cancels via
 * `ai_cancel` so the backend stops feeding deltas.
 */
export type AiActionKind = 'continue' | 'expand' | 'rewrite' | 'describe';
export type RewriteSubMode =
  | 'rephrase'
  | 'shorter'
  | 'vivid'
  | 'show_not_tell'
  | 'inner_conflict'
  | 'custom';

export type AiPhase = 'idle' | 'menu' | 'streaming' | 'preview' | 'error';

interface Options {
  editor: Ref<Editor | null>;
  projectId: Ref<string | null>;
  docId: Ref<string | null>;
}

/** How much text before the selection we send as continuity context. */
const CONTEXT_CHARS = 4000;

interface RunMeta {
  action: AiActionKind;
  subMode?: RewriteSubMode;
  from: number;
  to: number;
  actionLabel: string; // for history, e.g. "rewrite:vivid"
}

export function useAiInline(opts: Options) {
  const usageStore = useAiUsageStore();
  const phase = ref<AiPhase>('idle');
  const anchorRect = ref<DOMRect | null>(null);
  const streamedText = ref('');
  const originalText = ref('');
  const errorMsg = ref<string | null>(null);
  const usage = ref<{ prompt: number; completion: number } | null>(null);

  let currentRequestId: string | null = null;
  let meta: RunMeta | null = null;
  let unlisten: UnlistenFn | null = null;

  const isBusy = computed(() => phase.value === 'streaming');
  /** Diff makes sense only when there was an original to compare against. */
  const showsDiff = computed(() => meta?.action === 'rewrite' || meta?.action === 'expand');

  onMounted(async () => {
    unlisten = await listen<AiDeltaEvent>('ai.suggestion.received', (event) => {
      if (event.payload.requestId !== currentRequestId) return;
      streamedText.value += event.payload.delta;
    });
  });

  onUnmounted(() => {
    unlisten?.();
    if (currentRequestId) void ipc.aiCancel(currentRequestId);
  });

  function showMenuAt(rect: DOMRect) {
    if (phase.value === 'streaming' || phase.value === 'preview') return;
    anchorRect.value = rect;
    phase.value = 'menu';
  }

  function hideMenu() {
    if (phase.value === 'menu') reset();
  }

  function reset() {
    phase.value = 'idle';
    streamedText.value = '';
    originalText.value = '';
    errorMsg.value = null;
    usage.value = null;
    currentRequestId = null;
    meta = null;
  }

  async function run(action: AiActionKind, subMode?: RewriteSubMode, customPrompt?: string) {
    const ed = opts.editor.value;
    const projectId = opts.projectId.value;
    if (!ed || !projectId) return;

    const { from, to } = ed.state.selection;
    const selectedText = ed.state.doc.textBetween(from, to, '\n', '\n');
    const precedingText = ed.state.doc.textBetween(Math.max(0, to - CONTEXT_CHARS), to, '\n', '\n');

    currentRequestId = crypto.randomUUID();
    streamedText.value = '';
    originalText.value = selectedText;
    errorMsg.value = null;
    usage.value = null;
    phase.value = 'streaming';
    meta = {
      action,
      subMode,
      from,
      to,
      actionLabel: action === 'rewrite' ? `rewrite:${subMode ?? 'rephrase'}` : action,
    };

    const req: AiActionRequestInput = {
      requestId: currentRequestId,
      action,
      subMode,
      projectId,
      docId: opts.docId.value,
      selectedText,
      precedingText,
      customPrompt,
    };

    try {
      const result = await ipc.aiRunAction(req);
      if (result.cancelled) {
        reset();
        return;
      }
      // The stream events build `streamedText` live; trust the returned text
      // as the canonical final (covers any dropped events).
      streamedText.value = result.text;
      usage.value = {
        prompt: result.promptTokens ?? 0,
        completion: result.completionTokens ?? 0,
      };
      // Tokens are spent on generation regardless of accept/reject (F-13).
      usageStore.record(usage.value.prompt, usage.value.completion);
      phase.value = 'preview';
    } catch (e) {
      errorMsg.value = String((e as { message?: string })?.message ?? e);
      phase.value = 'error';
    }
  }

  function accept() {
    const ed = opts.editor.value;
    if (!ed || !meta) {
      reset();
      return;
    }
    const html = textToHtml(streamedText.value);
    const { action, from, to } = meta;
    if (action === 'expand' || action === 'rewrite') {
      // Replace the original selection.
      ed.chain().focus().insertContentAt({ from, to }, html).run();
    } else {
      // continue / describe: insert at the end of the selection/cursor.
      ed.chain().focus().insertContentAt(to, html).run();
    }
    // Persist the accepted generation (best-effort).
    const projectId = opts.projectId.value;
    if (projectId) {
      void ipc.aiRecordAccepted({
        projectId,
        docId: opts.docId.value,
        action: meta.actionLabel,
        response: streamedText.value,
      });
    }
    reset();
  }

  function reject() {
    if (currentRequestId) void ipc.aiCancel(currentRequestId);
    reset();
  }

  return {
    phase,
    anchorRect,
    streamedText,
    originalText,
    errorMsg,
    usage,
    isBusy,
    showsDiff,
    showMenuAt,
    hideMenu,
    run,
    accept,
    reject,
    reset,
  };
}

/** Convert plain AI text to safe paragraph HTML: blank lines split paragraphs,
 * single newlines become <br>, and HTML metacharacters are escaped. */
export function textToHtml(text: string): string {
  const escape = (s: string) =>
    s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  return text
    .trim()
    .split(/\n{2,}/)
    .map((para) => `<p>${escape(para).replace(/\n/g, '<br>')}</p>`)
    .join('');
}
