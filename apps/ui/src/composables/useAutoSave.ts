import { onBeforeUnmount } from 'vue';

/**
 * Schedule a debounced save callback. Returns a `trigger` function — call it
 * on every change, the underlying save fn runs once after `delay` ms of quiet.
 *
 * `delay` may be a getter so a reactive setting (e.g. the user's autosave
 * interval) is read fresh on each trigger rather than frozen at creation.
 *
 * `flush()` runs immediately, used on blur or app exit. `cancel()` discards a
 * pending invocation. `pending()` reports whether a save is queued.
 */
export function useAutoSave(
  saveFn: () => void | Promise<void>,
  delay: number | (() => number) = 500,
) {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let pendingFlag = false;

  function trigger() {
    pendingFlag = true;
    if (timer) clearTimeout(timer);
    const ms = typeof delay === 'function' ? delay() : delay;
    timer = setTimeout(async () => {
      timer = null;
      pendingFlag = false;
      await saveFn();
    }, ms);
  }

  async function flush() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    if (pendingFlag) {
      pendingFlag = false;
      await saveFn();
    }
  }

  function cancel() {
    if (timer) {
      clearTimeout(timer);
      timer = null;
    }
    pendingFlag = false;
  }

  function pending() {
    return pendingFlag;
  }

  onBeforeUnmount(() => {
    void flush();
  });

  return { trigger, flush, cancel, pending };
}
