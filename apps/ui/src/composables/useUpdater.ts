/** Auto-update state machine (Tauri 2). Singleton module state so the startup
 *  banner and the Settings panel share one source of truth. Wraps
 *  `@tauri-apps/plugin-updater` + `@tauri-apps/plugin-process`. */
import { computed, ref } from 'vue';
import { check as checkUpdate, type Update, type DownloadEvent } from '@tauri-apps/plugin-updater';
import { relaunch } from '@tauri-apps/plugin-process';

export type UpdaterStatus =
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'ready'
  | 'uptodate'
  | 'error';

const status = ref<UpdaterStatus>('idle');
const availableVersion = ref<string | null>(null);
const notes = ref<string | null>(null);
const progress = ref(0); // 0..100
const errorMessage = ref<string | null>(null);
const dismissed = ref(false); // dismissed for this session only

let pending: Update | null = null;

const bannerVisible = computed(() => status.value === 'available' && !dismissed.value);

export function useUpdater() {
  /** Check the endpoint. `silent` swallows errors (startup / offline). */
  async function check({ silent }: { silent: boolean }) {
    if (status.value === 'checking' || status.value === 'downloading') return;
    status.value = 'checking';
    errorMessage.value = null;
    try {
      const update = await checkUpdate();
      if (update) {
        pending = update;
        availableVersion.value = update.version;
        notes.value = update.body ?? null;
        dismissed.value = false;
        status.value = 'available';
      } else {
        pending = null;
        availableVersion.value = null;
        notes.value = null;
        status.value = 'uptodate';
      }
    } catch (e) {
      status.value = 'error';
      errorMessage.value = e instanceof Error ? e.message : String(e);
      // `silent` is intentional: a failed startup check must not nag the user.
      void silent;
    }
  }

  /** Download + install the pending update, tracking progress 0..100. */
  async function downloadAndInstall() {
    if (!pending) return;
    status.value = 'downloading';
    progress.value = 0;
    let total = 0;
    let received = 0;
    await pending.downloadAndInstall((event: DownloadEvent) => {
      switch (event.event) {
        case 'Started':
          total = event.data.contentLength ?? 0;
          break;
        case 'Progress':
          received += event.data.chunkLength ?? 0;
          progress.value = total > 0 ? Math.min(100, Math.round((received / total) * 100)) : 0;
          break;
        case 'Finished':
          progress.value = 100;
          break;
      }
    });
    status.value = 'ready';
  }

  /** Restart the app to run the freshly installed version. */
  async function relaunchApp() {
    await relaunch();
  }

  /** Hide the banner until the next launch. */
  function dismiss() {
    dismissed.value = true;
  }

  return {
    status,
    availableVersion,
    notes,
    progress,
    errorMessage,
    bannerVisible,
    check,
    downloadAndInstall,
    relaunchApp,
    dismiss,
  };
}
