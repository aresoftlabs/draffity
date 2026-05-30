import { computed, ref, type ComputedRef } from 'vue';

/** Un comando ejecutable desde la paleta ⌘K. La etiqueta llega ya resuelta por
 *  i18n desde quien registra (tiene acceso a `t()`); el registro no traduce. */
export interface Command {
  /** Id estable y único; re-registrar el mismo id reemplaza al anterior. */
  id: string;
  /** Texto visible, ya traducido. */
  label: string;
  /** Grupo para agrupar en la UI (ya traducido), p.ej. "Proyecto", "Global". */
  group: string;
  /** Icono PrimeIcons opcional, p.ej. "pi pi-file-export". */
  icon?: string;
  /** Términos extra para el match (sinónimos), no se muestran. */
  keywords?: string[];
  /** Acción a ejecutar al elegir el comando. */
  run: () => void;
}

/** Estado módulo-compartido: orden de inserción preservado vía Map. */
const registry = ref<Map<string, Command>>(new Map());

/**
 * Registra un lote de comandos y devuelve una función para retirarlos.
 * Re-registrar un id existente lo reemplaza (último gana) sin duplicar.
 */
export function registerCommands(commands: Command[]): () => void {
  const map = registry.value;
  for (const c of commands) map.set(c.id, c);
  // Forzar reactividad (Map mutado in place).
  registry.value = new Map(map);
  const ids = commands.map((c) => c.id);
  return () => {
    const m = registry.value;
    for (const id of ids) m.delete(id);
    registry.value = new Map(m);
  };
}

const commands: ComputedRef<Command[]> = computed(() => Array.from(registry.value.values()));

/** Normaliza para match: minúsculas + sin diacríticos. */
function norm(s: string): string {
  return s.toLowerCase().normalize('NFD').replace(/[̀-ͯ]/g, '');
}

/** Filtra por etiqueta o keywords (substring, acento-insensible). Query vacía → todo. */
export function filterCommands(list: Command[], query: string): Command[] {
  const q = norm(query.trim());
  if (!q) return list;
  return list.filter((c) => {
    if (norm(c.label).includes(q)) return true;
    return (c.keywords ?? []).some((k) => norm(k).includes(q));
  });
}

/** Accesor reactivo al registro. `clearAll` existe para los tests. */
export function useCommandRegistry(): {
  commands: ComputedRef<Command[]>;
  clearAll: () => void;
} {
  return {
    commands,
    clearAll: () => {
      registry.value = new Map();
    },
  };
}
