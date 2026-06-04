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

1. **Toda abstracción nueva sigue el patrón `trait + impl local`.** Si tu PR requiere una clase/struct concreta en la firma pública, la arquitectura ya falló — definí el trait primero.
2. **Sin pop-ups ni banners de venta.** Nunca.
3. **No infra propia de IA.** La IA funciona vía BYOK: el usuario aporta su propia clave de OpenRouter. No se hostea ningún LLM.
4. **Voz por binarios locales.** ASR (Whisper) y TTS (Piper) corren como sidecars en la máquina del usuario. Sin APIs de pago ni suscripciones de terceros.
5. **Draffity es 100 % gratis.** Todas las features son accesibles para todos los usuarios. La única regla de negocio es: 1 proyecto activo + N archivados read-only (invariante de foco del escritor).

## Estructura del código

- **`apps/desktop/src/domain/`** — entidades + invariantes puras, sin dependencias de SQLite ni Tauri. Testeable aislado.
- **`apps/desktop/src/services/`** — traits + implementaciones (storage, exporter, ai, asr, tts).
- **`apps/desktop/src/commands/`** — sólo IPC commands. No lógica de negocio aquí, sólo orquestación.

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

Etiqueta tus issues con `sprint-0`...`sprint-8`, `bug`, `enhancement`, `arch` (cuando toca límites de módulos).

## Developer Certificate of Origin (DCO)

Este proyecto usa DCO. Cada commit debe ir firmado con `git commit -s`, que agrega
una línea `Signed-off-by: Tu Nombre <tu@email>`. Con eso certificás que tenés
derecho a aportar ese código bajo la licencia del proyecto (**GPLv3**). Mantenés tu
copyright; no cedés derechos. Ver https://developercertificate.org/.
