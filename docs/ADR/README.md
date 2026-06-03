# Architecture Decision Records

> Las decisiones arquitectónicas importantes del proyecto se registran aquí como ADRs (Architecture Decision Records).
>
> Formato basado en [MADR](https://adr.github.io/madr/) ligero.

## Por qué

- **Memoria del proyecto**: explican _por qué_ tomamos una decisión, no solo _qué_ hicimos.
- **Onboarding**: un dev nuevo (o un agente) entiende el porqué sin tener que reconstruirlo desde git blame.
- **Reversibilidad informada**: si una decisión envejece mal, sabemos contra qué la estamos revirtiendo.

## Cuándo escribir un ADR

Cualquier decisión que cumpla **al menos uno** de estos criterios:

- Elige entre dos o más alternativas con trade-offs no obvios.
- Afecta a >1 módulo o cruza límites de capa.
- Reverso costoso (storage layout, public API, framework choice).
- Diverge de lo que un desarrollador con experiencia previa esperaría por defecto.

**No** son ADRs: convenciones de naming, formateo, elección entre dos libs equivalentes.

## Estructura de un ADR

Usar [`0000-template.md`](./0000-template.md) como base. Mantener:

- **Contexto**: el problema, no la solución.
- **Decisión**: lo que se hace, en imperativo.
- **Consecuencias**: positivas y negativas, con sinceridad.
- **Alternativas consideradas**: con razón de descarte.

## Estados

- `Proposed` — abierto a discusión, todavía no aplicado.
- `Accepted` — vigente, el código lo refleja.
- `Deprecated` — ya no se aplica pero se mantiene por historia.
- `Superseded by ADR-XXXX` — reemplazado, con referencia al nuevo.

## Índice

| #                                                           | Título                                                 | Estado                 |
| ----------------------------------------------------------- | ------------------------------------------------------ | ---------------------- |
| [0001](./0001-tauri-sobre-electron.md)                      | Tauri 2 sobre Electron                                 | Accepted               |
| [0002](./0002-sqlite-canonico-vs-por-proyecto.md)           | SQLite canónica única vs por proyecto                  | Accepted               |
| [0003](./0003-premium-aditivo-via-traits.md)                | Premium aditivo vía traits                             | Superseded by ADR-0006 |
| [0004](./0004-dispatch-por-match-para-formatos-cerrados.md) | Dispatch por `match` para formatos cerrados            | Accepted               |
| [0005](./0005-storage-service-trait-monolitico.md)          | `StorageService` permanece como trait único            | Accepted               |
| [0006](./0006-draffity-100-gratis-sin-premium.md)          | Draffity es 100% gratis: se elimina el modelo premium | Accepted               |
