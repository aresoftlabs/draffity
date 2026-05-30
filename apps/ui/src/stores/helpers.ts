/** Shared helpers for the Pinia stores (client-side cache of Rust truth). */

/**
 * Replace the element whose `id` matches in place, keeping array identity.
 * No-op when no element matches. Centralizes the `findIndex` + assign pattern
 * the stores repeated ~16 times (AUD-36).
 */
export function replaceById<T extends { id: string }>(list: T[], id: string, updated: T): void {
  const idx = list.findIndex((x) => x.id === id);
  if (idx !== -1) list[idx] = updated;
}
