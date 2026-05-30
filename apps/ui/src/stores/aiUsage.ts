import { computed, ref } from 'vue';
import { defineStore } from 'pinia';

/**
 * Token cost meter (F-13). Accumulates the real token usage reported by
 * OpenRouter (so it matches their dashboard by construction) for the current
 * calendar month, persisted to localStorage. We track tokens, not USD: pricing
 * is per-model and per-user on OpenRouter, so the honest surface is the token
 * count plus a link out to their usage page.
 */
const STORAGE_KEY = 'draffity.aiUsage';

interface MonthUsage {
  month: string; // YYYY-MM
  sent: number;
  received: number;
}

function currentMonth(): string {
  // Local YYYY-MM (not UTC): the backend stats use `chrono::Local`, so a UTC
  // `toISOString()` would mislabel the month near its boundary in far-from-UTC
  // time zones (AUD-22).
  const d = new Date();
  const m = String(d.getMonth() + 1).padStart(2, '0');
  return `${d.getFullYear()}-${m}`;
}

function load(): MonthUsage {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (raw) return JSON.parse(raw) as MonthUsage;
  } catch {
    // ignore corrupt/absent storage
  }
  return { month: currentMonth(), sent: 0, received: 0 };
}

export const useAiUsageStore = defineStore('aiUsage', () => {
  const data = ref<MonthUsage>(load());

  function persist() {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(data.value));
    } catch {
      // best-effort
    }
  }

  /** Reset the counters when the month rolls over. Call before reading. */
  function rollIfNeeded() {
    const m = currentMonth();
    if (data.value.month !== m) {
      data.value = { month: m, sent: 0, received: 0 };
      persist();
    }
  }

  function record(sent: number, received: number) {
    rollIfNeeded();
    data.value.sent += sent;
    data.value.received += received;
    persist();
  }

  function reset() {
    data.value = { month: currentMonth(), sent: 0, received: 0 };
    persist();
  }

  const sent = computed(() => data.value.sent);
  const received = computed(() => data.value.received);
  const month = computed(() => data.value.month);

  return { record, reset, rollIfNeeded, sent, received, month };
});
