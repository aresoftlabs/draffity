# ADR-0001 — Tauri 2 sobre Electron

- **Estado**: Accepted
- **Fecha**: 2026-05-08 (retroactivo)
- **Sprint / Épica**: Fase 0 del backlog v1

## Contexto

Draffity necesita una shell desktop multi-plataforma (Windows + Linux en MVP, Mac post-MVP) que permita:

1. Distribuir un instalable nativo por OS, sin asumir Node ni runtime en el equipo del usuario.
2. Hacer procesamiento intensivo de texto (exporters DOCX/EPUB/PDF, parsing de plantillas, indexado FTS) sin penalizar el tiempo de respuesta de la UI.
3. Mantener el tamaño del instalable razonable — el segmento de "escritores" tolera mal apps de 200 MB para escribir texto.
4. Tener una superficie de ataque pequeña: los datos del usuario son su manuscrito, no admite filtraciones por dependencias laxas de Node.

Las opciones realistas en 2026: **Electron** (estándar de facto, ecosistema enorme), **Tauri 2** (Rust + WebView nativo), **wails** (Go + WebView, menos maduro), **Flutter desktop** (descarta porque requiere reescribir todo en Dart).

## Decisión

Adoptar **Tauri 2** como shell desktop con backend en Rust y frontend Vue 3 + Vite.

## Consecuencias

### Positivas

- Bundles ~5–10× más livianos que Electron equivalente. Target alpha: MSI Windows < 15 MB, AppImage Linux < 20 MB (cumplido en `v0.1.0-alpha`).
- IPC seguro tipado entre frontend y backend (`#[tauri::command]` + `invoke`). Permite enforcement de permisos granular vía `apps/desktop/capabilities/`.
- Sin Node runtime en el equipo del usuario → menor superficie de ataque + cold start más rápido.
- Backend Rust → robustez en parsing de plantillas, exporters nativos (sin dependencia de Node libs como `docx`/`epub-gen`), tests unitarios rápidos.
- Ecosistema Rust ya tiene libs maduras para todo lo del MVP: `rusqlite`, `docx-rs`, `epub-builder`, `tracing`, `serde`.

### Negativas / costos

- WebView nativo varía por OS: `WebView2` en Windows, `WebKitGTK` en Linux, `WKWebView` en Mac. Algunos APIs experimentales del navegador funcionan distinto o no funcionan. Para un editor TipTap (DOM estándar) esto no es problema, pero limita features como WebGPU o algunos polyfills.
- En Linux requiere instalar `libwebkit2gtk-4.1-dev` y dependencias del sistema. Mitigado con AppImage que bundlea lo necesario.
- Menos developers familiarizados con Rust que con Node → curva de aprendizaje para contribuidores futuros.
- Mac requiere Apple Developer ID + notarización (no es problema exclusivo de Tauri, pero suma).

### Neutras

- Mantiene la puerta abierta a sidecars (`tauri-plugin-shell`) para integrar `whisper.cpp` sin pelear con el runtime principal.
- Permite emitir tipos compartidos Rust↔TS vía `specta` (planificado para Sprint 5 v2).

## Alternativas consideradas

### Electron

Estándar de facto, ecosistema masivo, mejor documentación, mejor portabilidad de features web modernas.

**Descartado** porque: bundles típicos de 80–150 MB (incluso "hello world" parte de ~80 MB), RAM baseline alto (>200 MB para una app vacía), Node runtime expuesto en el cliente con todo lo que implica de auditoría de dependencias. El segmento "escritor" es sensible a apps pesadas — Scrivener pesa ~50 MB, NovelCrafter (web) pesa lo que pesa Chrome. Electron nos pondría en el peor de los dos mundos.

### wails (Go + WebView)

Go es más simple que Rust como backend, ecosystem de exporters menos maduro pero existente.

**Descartado** porque: comunidad mucho menor que Tauri, menos features (no hay equivalente directo a `capabilities/`), libs de processing de texto en Go son menos completas que las de Rust (`docx-rs`, `epub-builder` no tienen pares directos en Go). Y nadie del proyecto tiene experiencia con Go suficiente como para apostar a la apuesta.

### Flutter desktop

Una sola codebase para mobile + desktop, UI consistente.

**Descartado** porque: requiere reescribir todo en Dart, perdemos el ecosystem de Vue/PrimeVue, y para una app cuyo 80% es un editor de texto (TipTap) el modelo de widget de Flutter es subóptimo. Mobile no es target del MVP ni del v1.

## Referencias

- Bootstrap: `apps/desktop/src/lib.rs:25-104`
- Capabilities granulares: `apps/desktop/capabilities/default.json`
- Documentación Tauri 2: https://tauri.app/
