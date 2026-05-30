import { ref, type Ref } from 'vue';

/** Visibilidad módulo-compartida de la paleta ⌘K, para abrirla desde cualquier
 *  componente (atajo de teclado o botón) sin prop drilling. */
const visible = ref(false);

const RECENT_KEY = 'draffity.ui.recentCommands';
const RECENT_CAP = 8;

function loadRecent(): string[] {
  if (typeof localStorage === 'undefined') return [];
  try {
    const raw = localStorage.getItem(RECENT_KEY);
    const parsed = raw ? (JSON.parse(raw) as unknown) : [];
    return Array.isArray(parsed) ? parsed.filter((x): x is string => typeof x === 'string') : [];
  } catch {
    return [];
  }
}

/** Ids de los últimos comandos ejecutados, más reciente primero. Persistido
 *  para alimentar la sección "Recientes" del estado vacío de la paleta. */
const recentIds = ref<string[]>(loadRecent());

function pushRecent(id: string): void {
  recentIds.value = [id, ...recentIds.value.filter((x) => x !== id)].slice(0, RECENT_CAP);
  if (typeof localStorage === 'undefined') return;
  try {
    localStorage.setItem(RECENT_KEY, JSON.stringify(recentIds.value));
  } catch {
    // localStorage lleno o no disponible: la sección Recientes es best-effort.
  }
}

export function useCommandPalette(): {
  visible: Ref<boolean>;
  recentIds: Ref<string[]>;
  open: () => void;
  close: () => void;
  toggle: () => void;
  pushRecent: (id: string) => void;
} {
  return {
    visible,
    recentIds,
    open: () => {
      visible.value = true;
    },
    close: () => {
      visible.value = false;
    },
    toggle: () => {
      visible.value = !visible.value;
    },
    pushRecent,
  };
}
