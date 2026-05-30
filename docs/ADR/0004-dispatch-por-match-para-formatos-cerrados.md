# ADR-0004 — Dispatch por `match` para sets de formato cerrados

- **Estado**: Accepted
- **Fecha**: 2026-05-30
- **Sprint / Épica**: Remediación de auditoría (AUD-25)

## Contexto

`CLAUDE.md §2` prescribe el patrón Strategy como `HashMap<Format, Box<dyn Strategy>>`
para export/import. La implementación real (`exporter/mod.rs`, `importer/mod.rs`)
usa en cambio un `match format { ... }` que delega a funciones libres por formato
(`markdown::render`, `docx::render`, …). La auditoría marcó la contradicción entre
código y doc.

El conjunto de formatos es **cerrado y conocido en compile-time**: es un `enum`
(`ExportFormat`, `ImportFormat`) que solo cambia editando el código del propio
crate. No hay formatos de terceros ni carga dinámica.

## Decisión

> Para un **set cerrado** de variantes (un `enum` exhaustivo), el dispatch idiomático
> es `match`, no un registry de trait objects. Reservamos el patrón
> `HashMap<Key, Box<dyn Strategy>>` para sets **abiertos/extensibles** (plugins de
> terceros, registro dinámico en runtime).

Concretamente:

1. `exporter` e `importer` mantienen el `match format` a funciones de render/parse
   por formato. Cada formato sigue siendo un módulo independiente (la separación que
   buscaba el patrón Strategy ya existe a nivel de módulo).
2. `CLAUDE.md §2` se enmienda para reflejar esta regla (set cerrado → `match`; set
   abierto → registry).

## Consecuencias

### Positivas

- **Exhaustividad en compile-time**: añadir una variante al `enum` sin manejarla es
  un error de compilación. El registry lo detectaría recién en runtime (o nunca).
- Sin dispatch dinámico ni allocation de `Box<dyn>` para un set fijo.
- Menos boilerplate: no hay que registrar cada strategy ni mantener el mapa.
- Código y doc dejan de contradecirse (cierra AUD-25).

### Negativas / costos

- Si en el futuro aparecen formatos de terceros/plugins, habrá que migrar a un
  registry. El coste es bajo: cada formato ya es un módulo con la misma firma, así
  que envolverlos en strategies es mecánico.

## Alternativas consideradas

### Implementar el registry `HashMap<Format, Box<dyn Strategy>>`

Cumpliría la letra del `CLAUDE.md` original.

**Descartado** porque añade dispatch dinámico, allocation y boilerplate de registro
para un set que no es extensible, y pierde la verificación de exhaustividad que da
el `match` — un peor trade-off para el caso real.

## Referencias

- Dispatch de export: [`exporter/mod.rs`](../../apps/desktop/src/services/exporter/mod.rs)
- Dispatch de import: [`importer/mod.rs`](../../apps/desktop/src/services/importer/mod.rs)
- Regla de origen: `CLAUDE.md §2` (Strategy)
