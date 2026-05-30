# ADR-0005 — `StorageService` permanece como trait único

- **Estado**: Accepted
- **Fecha**: 2026-05-30
- **Sprint / Épica**: Remediación de auditoría (AUD-29)

## Contexto

`StorageService` es un trait grande (~64 métodos, ~826 líneas de declaración +
impl thin). La auditoría (AUD-29) lo señaló como un "god-trait" difícil de mockear
por dominio y sugirió evaluar partirlo en sub-traits (`StorageReader`,
`ProjectStore`, `DocumentStore`, `CodexStore`, …).

Contexto relevante:

- La **implementación** ya está modularizada por submódulos de dominio
  (`storage/projects.rs`, `documents.rs`, `codex.rs`, `snapshots.rs`, `stats.rs`,
  `search.rs`, …). El trait monolítico es la única capa "plana"; debajo hay cohesión.
- El trait lo consume `AppState` como un `Arc<dyn StorageService>` único, y cada
  comando lo usa puntualmente. Partir el trait obliga a propagar varios
  `Arc<dyn XStore>` por `AppState` y por cada caller.
- `CLAUDE.md §7` exige **parar y discutir** antes de un refactor invasivo (>3
  archivos no relacionados con una feature).

## Decisión

> Mantener `StorageService` como un único trait por ahora. **No** dividirlo en
> sub-traits en esta ronda.

La cohesión por dominio ya existe a nivel de submódulos de implementación; el costo
de partir el trait (tocar `AppState` y todos los comandos) supera el beneficio
actual de mockabilidad granular, que hoy se cubre con `LocalStorageService::open_in_memory()`
en los tests de integración.

## Consecuencias

### Positivas

- Cero churn invasivo ahora; los comandos y `AppState` no cambian.
- La superficie pública de storage sigue siendo un solo punto de inyección.

### Negativas / costos

- El trait sigue siendo grande: un mock manual completo es tedioso (mitigado porque
  los tests usan la impl real in-memory, no mocks).
- Si más adelante un consumidor necesita solo lectura (p. ej. un servicio premium de
  sync que no debe escribir), convendrá extraer al menos un `StorageReader`. Este ADR
  no lo prohíbe; lo difiere hasta que haya un consumidor que lo justifique.

## Disparadores de revisión

Reconsiderar la división si:

1. Aparece un consumidor que necesita un subconjunto acotado (read-only, solo
   proyectos) y mezclar todo el trait lo obliga a depender de métodos que no usa.
2. El trait supera un umbral que haga inmanejable su mantenimiento (heurística:
   si añadir un método de dominio nuevo obliga a tocar >1 archivo de wiring).

## Referencias

- Trait: [`storage/mod.rs`](../../apps/desktop/src/services/storage/mod.rs)
- Submódulos de dominio: [`storage/`](../../apps/desktop/src/services/storage/)
- Regla "parar y discutir": `CLAUDE.md §7`
