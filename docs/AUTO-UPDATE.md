# Auto-update

> **Estado: IMPLEMENTADO.** Auto-update activo para Windows (NSIS per-user) y
> Linux (AppImage), sirviendo manifiesto + instaladores firmados desde
> `bins.draffity.com/app/`. Pendiente: macOS y firma OS-level (diferidos).
> Repo open-source público: `aresoftlabs/draffity`.

## Cómo funciona el auto-update en Tauri 2

1. La app incluye `tauri-plugin-updater` (Rust) + `@tauri-apps/plugin-updater` (JS).
2. `tauri.conf.json → plugins.updater` define:
   - `endpoints`: una o más URLs a un **manifiesto JSON** (p. ej. `latest.json`).
   - `pubkey`: la **clave pública** con la que la app verifica la firma del artefacto.
3. Al chequear updates, la app baja el manifiesto, compara versión, y si hay una
   nueva descarga el instalador **firmado** y verifica su `.sig` contra `pubkey`.
4. El manifiesto lista, por plataforma, `{ version, notes, pub_date, platforms: { "<target>": { signature, url } } }`.

El instalador y el manifiesto deben ser **accesibles públicamente sin autenticación**
(la app los baja con un cliente HTTP sin token). Ese es el prerequisito duro.

## Cómo está implementado en Draffity

- **Endpoint en R2 (no GitHub Releases).** El tag `v*` dispara `release.yml`, que
  buildea + firma (minisign) los instaladores y los sube a Cloudflare R2 (bucket
  `bins-draffity`), versionados en `app/<version>/`. Un job de agregación arma
  `app/stable/latest.json`. La app lee ese manifiesto **público** (sin auth) servido
  desde un dominio propio sobre R2. Servir desde R2 nos da control total del path,
  evita el baile de draft/prerelease de GitHub Releases y reusa el mismo patrón que
  el vendoreo de binarios whisper/voces.
- **Manifiesto propio.** En vez de `includeUpdaterJson` de `tauri-action`, el
  manifiesto se arma con `scripts/build-update-manifest.mjs` (subcomandos
  `fragment` por plataforma y `assemble`), con pruebas en
  `scripts/build-update-manifest.test.mjs`.
- **Firma minisign.** Pública en `tauri.conf.json → plugins.updater.pubkey`; privada +
  password como secrets de CI (`TAURI_SIGNING_PRIVATE_KEY`,
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`). La privada **no** se commitea ni se rota sin
  re-firmar todo.
- **Windows NSIS per-user** (`bundle.windows.nsis.installMode: currentUser` + updater
  `windows.installMode: passive`): el auto-update se aplica **sin prompt de UAC**.
  **Linux AppImage** se reemplaza in-place.
- **UI:** `useUpdater` (composable, máquina de estados) + `UpdateBanner` (aviso no
  intrusivo, chequeo silencioso al inicio) + `SettingsUpdates` (chequeo manual +
  versión actual).

## Checklist de implementación

- [x] Decidir y ejecutar **releases públicas** (host público `bins.draffity.com/app/`).
- [x] `tauri signer generate`; cargar privada+password como secrets de CI; pública en config.
- [x] Agregar `tauri-plugin-updater` (Rust) y `@tauri-apps/plugin-updater` (JS).
- [x] `plugins.updater` en `tauri.conf.json` (`endpoints` + `pubkey`).
- [x] UI mínima: chequeo de updates, descarga, "reiniciar para actualizar".
- [x] Script de manifiesto (`scripts/build-update-manifest.mjs`) y pruebas.
- [ ] Sumar **macOS** a `release.yml` + firma OS-level (diferido).
- [ ] Probar el ciclo end-to-end (vX → vX+1) en las 3 plataformas (pendiente macOS).

## Pendientes diferidos

macOS y la firma de código a nivel OS (SmartScreen en Windows, Gatekeeper /
notarization en macOS) quedan diferidos: el auto-update ya funciona sin ellos en
Windows y Linux. Hasta entonces, los instaladores son **sin firma OS-level** y el
usuario puede ver un aviso de "editor desconocido" en la primera instalación.
