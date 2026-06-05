# Auto-update con Tauri 2 sobre Cloudflare R2 — Diseño

> Estado: **diseño aprobado**, pendiente de plan de implementación.
> Fecha: 2026-06-05. Repo: `aresoftlabs/draffity`.

## Objetivo

Implementar actualización automática para la app desktop (Tauri 2) en **Windows y
Linux**, reusando el patrón "build en GitHub → vendor a Cloudflare R2" que ya se usa
para los binarios de whisper y el catálogo de voces. El updater consume los artefactos
desde `bins.draffity.com` (público, sin auth), **no** desde GitHub Releases.

Esto desbloquea de paso el problema que documenta [`docs/AUTO-UPDATE.md`](../AUTO-UPDATE.md):
mientras el repo siga privado, los assets de release dan 404 sin token. Sirviendo desde
R2 (público) el updater no depende de que el repo sea público.

## Alcance

**Dentro:**

- Auto-update para Windows (instalador NSIS per-user) y Linux (AppImage).
- Firma del updater con minisign (obligatoria para que Tauri verifique el artefacto).
- Vendoreo de instaladores + manifiesto a R2 desde el pipeline de release existente.
- UX: chequeo al inicio + botón manual en Ajustes, aviso no intrusivo, descarga y
  reinicio iniciados por el usuario.

**Fuera (diferido):**

- macOS (build + firma Apple Developer ID + notarización) — pospuesto al post
  open-source prep, según nota de proyecto.
- Firma OS-level de Windows (SmartScreen) / Linux. El updater funciona sin ella.
- Canal beta, rollout gradual, telemetría de updates, delta updates. El layout R2 deja
  lugar (`app/beta/`) pero no se implementa ahora.
- Hacer público el GitHub Release (parte del open-source prep; no bloquea el updater).

## Decisiones tomadas

| Tema                    | Decisión                                             | Motivo                                                                                                                                  |
| ----------------------- | ---------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------- |
| Plataformas             | Windows + Linux                                      | Las que ya buildea el pipeline; macOS diferido.                                                                                         |
| Endpoint del manifiesto | Estático en R2 (`bins.draffity.com`)                 | Reusa el patrón whisper/voces; público sin auth; control total del path; evita el lío draft/prerelease y el bloqueante de repo privado. |
| Instalador Windows      | NSIS per-user (`currentUser`)                        | Auto-update sin prompt de UAC; modo `passive`; recomendado por Tauri para el updater. Reemplaza el target `msi`.                        |
| UX                      | Chequeo al inicio + botón manual; aviso no intrusivo | App de escritura: nunca reiniciar sin permiso del usuario.                                                                              |
| Firma                   | minisign vía secrets de CI                           | Obligatoria para el updater; independiente de la firma OS-level (diferida).                                                             |

## Arquitectura

```
push tag v0.13.0
   └─> release.yml (matrix win + linux)
        ├─ tauri-action buildea + FIRMA (minisign) → instalador + .sig
        ├─ sube instalador + .sig a R2:  app/0.13.0/...   (inmutable, versionado)
        └─ (sigue creando el GitHub Release draft para changelog humano)
   └─> job de agregación (needs: [publish])
        └─ arma latest.json (ambas plataformas) → R2: app/stable/latest.json (pisable)

app en runtime
   ├─ al inicio (+ botón en Ajustes) → GET app/stable/latest.json
   ├─ compara semver; si hay nueva → banner discreto "v0.13.0 disponible"
   ├─ usuario "Actualizar ahora" → baja instalador, verifica .sig contra pubkey
   └─ "Reiniciar para aplicar" → installMode passive (NSIS, sin UAC) → relanza
```

Como el updater baja por cliente HTTP (no por el navegador), el instalador no recibe
Mark-of-the-Web y **no dispara SmartScreen** en cada update. El aviso de "editor
desconocido" solo aparece en la primera instalación manual desde la web (hasta que se
implemente la firma OS-level, diferida).

## Layout en R2 (bucket `bins-draffity`, servido en `bins.draffity.com`)

```
app/
  0.13.0/                         ← inmutable, una carpeta por versión (igual que whisper/<v>/)
    Draffity_0.13.0_x64-setup.exe
    Draffity_0.13.0_x64-setup.exe.sig
    Draffity_0.13.0_amd64.AppImage
    Draffity_0.13.0_amd64.AppImage.sig
  stable/
    latest.json                   ← pisable: puntero a la versión vigente
  staging/
    latest.json                   ← (opcional) para probar el ciclo E2E sin tocar usuarios
```

`latest.json` (URLs absolutas a R2):

```json
{
  "version": "0.13.0",
  "notes": "Mejoras de exportación y voz.",
  "pub_date": "2026-06-05T12:00:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "<contenido del .sig>",
      "url": "https://bins.draffity.com/app/0.13.0/Draffity_0.13.0_x64-setup.exe"
    },
    "linux-x86_64": {
      "signature": "<contenido del .sig>",
      "url": "https://bins.draffity.com/app/0.13.0/Draffity_0.13.0_amd64.AppImage"
    }
  }
}
```

- **Rollback** = re-apuntar `app/stable/latest.json` a una carpeta de versión previa. Las
  carpetas versionadas son inmutables y no se borran.
- **Beta futuro** = un `app/beta/latest.json` con su propio endpoint (no se implementa ahora).
- El `version` del manifiesto debe ser igual a la versión de la app
  ([`tauri.conf.json`](../../apps/desktop/tauri.conf.json) y
  [`Cargo.toml`](../../apps/desktop/Cargo.toml), hoy `0.12.0`), que a su vez deriva del
  tag `v*`. El updater compara semver.

## Cambios en el pipeline (`release.yml`)

1. Añadir al `env` del paso `tauri-action` los secrets de firma:
   `TAURI_SIGNING_PRIVATE_KEY` y `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Con eso
   `tauri-action` emite los `.sig` junto a los bundles.
2. Tras el build, cada job de la matrix sube su instalador + `.sig` a
   `s3://bins-draffity/app/<version>/` con `aws s3 cp` + `--endpoint-url "$R2_ENDPOINT"`
   y los secrets R2 ya existentes (`R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`,
   `R2_ENDPOINT`), idéntico al paso de [`build-whisper.yml`](../../.github/workflows/build-whisper.yml).
3. **Job de agregación** (`needs: [publish]`): junta los `.sig` de ambas plataformas
   (subidos como artifacts por cada job de build), arma `latest.json` con un script Node
   chico (estilo [`scripts/sync-voice-manifest.mjs`](../../scripts/sync-voice-manifest.mjs))
   y lo sube a `app/stable/`.
4. El paso de GitHub Release se mantiene como está (draft/prerelease) — es solo para
   changelog humano; el updater no lo usa.

> **Alternativa considerada y descartada:** endpoint con plantilla
> `…/app/stable/{{target}}-{{arch}}.json` y cada job sube su propio mini-manifiesto, sin
> job de agregación. Más simple en CI pero parte el "source of truth" en dos archivos.
> Se elige el `latest.json` único por ser inspeccionable y de rollback trivial (un archivo).

## Cambios en la app (Rust + config)

**Dependencias** — [`apps/desktop/Cargo.toml`](../../apps/desktop/Cargo.toml):

```toml
tauri-plugin-updater = "2"
tauri-plugin-process = "2"   # para relaunch() tras instalar
```

**JS** — `apps/ui/package.json`: `@tauri-apps/plugin-updater` + `@tauri-apps/plugin-process`.

**Registro** — [`apps/desktop/src/lib.rs`](../../apps/desktop/src/lib.rs):

```rust
.plugin(tauri_plugin_updater::Builder::new().build())
.plugin(tauri_plugin_process::init())
```

**[`apps/desktop/tauri.conf.json`](../../apps/desktop/tauri.conf.json):**

- `bundle.targets`: `["msi","appimage"]` → `["nsis","appimage"]`
- `bundle.createUpdaterArtifacts: true` (Tauri 2 lo exige para emitir los `.sig`)
- `bundle.windows.nsis.installMode: "currentUser"` (per-user → sin UAC)
- `plugins.updater`:
  ```json
  {
    "endpoints": ["https://bins.draffity.com/app/stable/latest.json"],
    "pubkey": "<clave pública minisign>",
    "windows": { "installMode": "passive" }
  }
  ```

**Capabilities** — [`apps/desktop/capabilities/default.json`](../../apps/desktop/capabilities/default.json):
sumar `"updater:default"` y `"process:allow-restart"`.

**Claves de firma** (paso manual, una vez):

- `tauri signer generate` → clave pública (base64) + clave privada protegida por password.
- Pública → `plugins.updater.pubkey` en `tauri.conf.json`.
- Privada + password → secrets de CI `TAURI_SIGNING_PRIVATE_KEY` /
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` en `aresoftlabs/draffity`.
- La privada **no** se commitea. Rotarla obliga a re-firmar todas las versiones servidas.

## UX del frontend

- **`apps/ui/src/composables/useUpdater.ts`** (mismo patrón que los demás `useX`): estado
  `idle | checking | available | downloading | ready | error`, más `version`, `notes`,
  `progress`. Métodos: `check({ silent })`, `downloadAndInstall()`, `relaunchApp()`.
  Envuelve `@tauri-apps/plugin-updater` y `@tauri-apps/plugin-process`.
- **Chequeo al inicio**: en [`App.vue`](../../apps/ui/src/App.vue), `check({ silent: true })`
  al montar. Si falla (offline) se traga el error, sin ruido.
- **`apps/ui/src/components/UpdateBanner.vue`**: aviso discreto (no modal) cuando hay
  update — versión + notas + `Actualizar ahora` / `Más tarde`. "Más tarde" lo descarta por
  la sesión (re-chequea al próximo arranque). Durante la descarga muestra barra de
  progreso; al terminar → `Reiniciar para aplicar` (`relaunchApp()`).
- **`apps/ui/src/components/SettingsUpdates.vue`** dentro de
  [`Settings.vue`](../../apps/ui/src/views/Settings.vue) (junto a `SettingsBackups` /
  `SettingsStats`): versión actual + botón **"Buscar actualizaciones"** (reusa el
  composable). Estado explícito "Estás al día" cuando no hay nada.
- **i18n**: strings nuevas en [`apps/ui/src/locales/`](../../apps/ui/src/locales/index.ts).

## Testing

- **Script del manifiesto** (Node, estilo `sync-voice-manifest.mjs`): test unitario →
  dados los `.sig` + versión + notas, produce el `latest.json` correcto (claves
  `windows-x86_64` / `linux-x86_64`, URLs R2 bien formadas, `signature` = contenido del
  `.sig`). TDD acá porque es el punto frágil.
- **`useUpdater`**: test unitario de la máquina de estados con el plugin mockeado
  (available / al día / error / progreso).
- **Ciclo E2E real** (manual, en staging): publicar a `app/staging/latest.json`, instalar
  vX, publicar vX+1, confirmar que la app detecta → baja → verifica firma → relanza. El
  path `staging` separado evita tocar usuarios reales durante la prueba.

## Unidades y responsabilidades

| Unidad                            | Qué hace                                                          | Depende de                                      |
| --------------------------------- | ----------------------------------------------------------------- | ----------------------------------------------- |
| `release.yml` (build jobs)        | Buildea+firma por plataforma, sube instalador+`.sig` a `app/<v>/` | secrets de firma + R2                           |
| `release.yml` (aggregate job)     | Arma y sube `latest.json` a `app/stable/`                         | script del manifiesto, `.sig` de los build jobs |
| script del manifiesto (Node)      | `.sig` + versión + notas → `latest.json` válido                   | — (función pura, testeable)                     |
| `tauri-plugin-updater` + `pubkey` | Chequea endpoint, verifica firma, descarga, instala               | endpoint R2 + clave pública                     |
| `useUpdater.ts`                   | Máquina de estados de update para la UI                           | plugins updater/process                         |
| `UpdateBanner.vue`                | Aviso no intrusivo + progreso + reinicio                          | `useUpdater`                                    |
| `SettingsUpdates.vue`             | Chequeo manual + versión actual                                   | `useUpdater`                                    |

## Prerequisitos / pasos manuales (fuera del código)

1. `tauri signer generate` y cargar la clave privada + password como secrets de CI.
2. Confirmar que `bins.draffity.com` sirve el prefijo `app/` igual que `whisper/` y `voices/`.
3. (Opcional) crear el path `app/staging/` para la prueba E2E.
