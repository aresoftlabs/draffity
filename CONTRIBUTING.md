# Contributing to Draffity

## Workflow

1. Fork / branch desde `main` con un nombre descriptivo (`feat/binder-drag`, `fix/autosave-debounce`).
2. Commits siguen [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `chore:`, `docs:`, `refactor:`, `test:`).
3. Ejecuta antes de PR:
   - `pnpm fmt`
   - `pnpm lint`
   - `pnpm typecheck`
   - `pnpm test`
4. PR pequeño, foco único. Si una historia toca >5 archivos, considéralo un indicio de que se puede dividir.

## Reglas de oro del proyecto

1. **Premium es aditivo, no invasivo.** Cualquier abstracción nueva debe seguir el patrón `trait + impl NoOp`. Si tu PR requiere "refactorizar X para añadir premium luego", la arquitectura ya falló — corrígela primero.
2. **Sin premium leakage en la UI del MVP.** Ningún botón, banner o mensaje habla de premium. Los gates simplemente devuelven `false` y la UI se comporta acorde.
3. **Sin pop-ups de venta agresivos.** Nunca.
4. **No infra propia de IA.** Premium usa BYOK (OpenRouter). No se hostea ningún LLM.
5. **Free es completamente funcional.** Limitar features básicas en free es contrario a la propuesta del producto. La única limitación del free es: 1 proyecto activo + N archivados read-only.

## Estructura del código

- **`apps/desktop/src/domain/`** — entidades + invariantes puras, sin dependencias de SQLite ni Tauri. Testeable aislado.
- **`apps/desktop/src/services/`** — traits + implementaciones (storage, exporter, ai, sync, asr, tier).
- **`apps/desktop/src/commands/`** — sólo IPC commands. No lógica de negocio aquí, sólo orquestación.
- **`apps/desktop/src/capabilities.rs`** — única fuente de verdad de feature gates.
- **`apps/ui/src/composables/useCapability.ts`** — única forma en que la UI consulta capabilities.

## Cobertura de tests

La cobertura **no es gating** en CI todavía (llega en Sprint 8 del backlog v2), pero se publica para tracking.

- **Rust** (`domain` + `services`): requiere `cargo-llvm-cov` (`cargo install cargo-llvm-cov`).
  - HTML report local: `pnpm coverage:rs` → abrir `apps/desktop/target/llvm-cov/html/index.html`
  - Resumen en consola: `pnpm coverage:rs:summary`
- **TypeScript** (`composables` + `stores`): `pnpm coverage:ts`

Objetivos (target Sprint 8): Rust ≥80% en `domain`+`services`, TS ≥70% en `composables`+`stores`.

## Architecture Decision Records

Decisiones arquitectónicas importantes van en `docs/ADR/`. Usar la plantilla [`docs/ADR/0000-template.md`](./docs/ADR/0000-template.md). Ver criterios de "cuándo escribir un ADR" en [`docs/ADR/README.md`](./docs/ADR/README.md).

## Issues

Etiqueta tus issues con `sprint-0`...`sprint-8`, `bug`, `enhancement`, `premium-ready` (cuando toca abstracciones que premium reutilizará), `arch` (cuando toca límites de módulos).
