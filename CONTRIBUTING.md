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

## Issues

Etiqueta tus issues con `phase-0`...`phase-5`, `bug`, `enhancement`, `premium-ready` (cuando toca abstracciones que premium reutilizará).
