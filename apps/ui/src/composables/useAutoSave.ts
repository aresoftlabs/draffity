import { onBeforeUnmount } from 'vue';

/**
 * Schedule a debounced save callback. Returns a `trigger` function — call it
 * on every change, the underlying save fn runs once after `delay` ms of quiet.
 *
 * `flush()` runs immediately, used on blur or app exit. `cancel()` discards a
 * pending invocation. `pending()` reports whether a save is queued.
 */
export function useAutoSave(saveFn: () => void | Promise<void>, delay = 500) {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let pendingFlag = false;

  function trigger() {
    pendingFlag = true;
    if (timer) clearTimeout(timer);
    timer = setTimeout(async () => {
      timer = null;
      pendingFlag = false;
      await saveFn();
    }, delay);
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
