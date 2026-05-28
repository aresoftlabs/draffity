import { defineStore } from 'pinia';
import { ref } from 'vue';
import { ipc } from '@/services/ipc';

/**
 * In-memory cache of `mediaId → Blob URL`. The Image node-view queries this
 * store to render the live `<img>` without going through IPC every time.
 *
 * Blob URLs live as long as the document is open; on `reset()` (project
 * switch) we revoke them so the WebView doesn't hold byte buffers we'll
 * never use again.
 */
export const useMediaStore = defineStore('media', () => {
  const urls = ref<Map<string, string>>(new Map());
  const mimeById = ref<Map<string, string>>(new Map());
  const pending = new Map<string, Promise<string | null>>();

  /** Resolve a Blob URL for `id`, fetching bytes if not cached. Returns null
   *  when the asset row is gone (stale reference). */
  async function resolve(id: string): Promise<string | null> {
    if (!id) return null;
    const cached = urls.value.get(id);
    if (cached) return cached;
    const inFlight = pending.get(id);
    if (inFlight) return inFlight;

    const promise = (async () => {
      try {
        const asset = await ipc.getMediaAsset(id);
        if (!asset) return null;
        const bytes = await ipc.readMediaBytes(id);
        const blob = new Blob([new Uint8Array(bytes)], { type: asset.mime });
        const url = URL.createObjectURL(blob);
        urls.value.set(id, url);
        mimeById.value.set(id, asset.mime);
        return url;
      } catch {
        return null;
      } finally {
        pending.delete(id);
      }
    })();
    pending.set(id, promise);
    return promise;
  }

  /** Drop the cache + revoke every URL. Called on project switch. */
  function reset() {
    for (const url of urls.value.values()) {
      URL.revokeObjectURL(url);
    }
    urls.value = new Map();
    mimeById.value = new Map();
    pending.clear();
  }

  /** Invalidate a single id (e.g. after delete). */
  function forget(id: string) {
    const url = urls.value.get(id);
    if (url) {
      URL.revokeObjectURL(url);
      urls.value.delete(id);
    }
    mimeById.value.delete(id);
  }

  function mime(id: string): string | undefined {
    return mimeById.value.get(id);
  }

  return { urls, resolve, reset, forget, mime };
});
