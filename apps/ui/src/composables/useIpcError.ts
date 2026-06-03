import { useToast } from 'primevue/usetoast';
import type { WireError } from '@draffity/shared-types';

/**
 * Wrap an IPC promise and surface failures as a toast. Returns the result on
 * success, or `null` on failure. Logs structured errors to the console for
 * future telemetry hooks.
 */
export function isWireError(e: unknown): e is WireError {
  return !!e && typeof e === 'object' && 'code' in e && 'message' in e;
}

export function useIpcError() {
  const toast = useToast();

  async function run<T>(label: string, op: () => Promise<T>): Promise<T | null> {
    try {
      return await op();
    } catch (e) {
      const detail = isWireError(e) ? `${e.code}: ${e.message}` : String(e);
      console.error('[ipc]', label, e);
      toast.add({
        severity: 'error',
        summary: label,
        detail,
        life: 6000,
      });
      return null;
    }
  }

  return { run };
}
