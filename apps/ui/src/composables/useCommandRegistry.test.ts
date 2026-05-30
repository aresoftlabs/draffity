import { describe, expect, it, beforeEach } from 'vitest';
import {
  useCommandRegistry,
  registerCommands,
  filterCommands,
  type Command,
} from './useCommandRegistry';

function cmd(id: string, label: string, keywords: string[] = []): Command {
  return { id, label, group: 'test', run: () => {}, keywords };
}

describe('useCommandRegistry', () => {
  beforeEach(() => {
    const { commands, clearAll } = useCommandRegistry();
    clearAll();
    expect(commands.value.length).toBe(0);
  });

  it('registra comandos y los expone reactivamente', () => {
    const { commands } = useCommandRegistry();
    registerCommands([cmd('a', 'Exportar'), cmd('b', 'Buscar')]);
    expect(commands.value.map((c) => c.id)).toEqual(['a', 'b']);
  });

  it('la función de desuscripción retira exactamente esos comandos', () => {
    const { commands } = useCommandRegistry();
    const off = registerCommands([cmd('a', 'Exportar')]);
    registerCommands([cmd('b', 'Buscar')]);
    off();
    expect(commands.value.map((c) => c.id)).toEqual(['b']);
  });

  it('ignora ids duplicados re-registrados (último gana, sin duplicar)', () => {
    const { commands } = useCommandRegistry();
    registerCommands([cmd('a', 'Exportar')]);
    registerCommands([cmd('a', 'Exportar v2')]);
    expect(commands.value.filter((c) => c.id === 'a').length).toBe(1);
    expect(commands.value.find((c) => c.id === 'a')?.label).toBe('Exportar v2');
  });
});

describe('filterCommands', () => {
  const list: Command[] = [
    cmd('export', 'Exportar manuscrito', ['compilar', 'docx']),
    cmd('focus', 'Modo foco'),
    cmd('search', 'Buscar en el proyecto'),
  ];

  it('sin query devuelve todo en orden', () => {
    expect(filterCommands(list, '').map((c) => c.id)).toEqual(['export', 'focus', 'search']);
  });

  it('matchea por etiqueta sin distinguir mayúsculas ni acentos', () => {
    expect(filterCommands(list, 'foco').map((c) => c.id)).toEqual(['focus']);
    expect(filterCommands(list, 'BUSCAR').map((c) => c.id)).toEqual(['search']);
  });

  it('matchea por keyword además de la etiqueta', () => {
    expect(filterCommands(list, 'docx').map((c) => c.id)).toEqual(['export']);
  });

  it('query sin coincidencias devuelve vacío', () => {
    expect(filterCommands(list, 'zzz')).toEqual([]);
  });
});
