/** Umbral por encima del cual descartar pide confirmación (spec §11.3). */
export const LONG_RECORDING_MS = 30_000;

/** ¿Conviene confirmar antes de descartar una grabación de `elapsedMs`? */
export function shouldConfirmDiscard(elapsedMs: number): boolean {
  return elapsedMs >= LONG_RECORDING_MS;
}
