# Auto-update — preparación (feature futura)

> **Estado: NO implementado.** Este documento deja el terreno preparado para
> implementar la auto-actualización más adelante. Hoy el código **no** referencia
> ningún updater (sin `tauri-plugin-updater`, sin `plugins.updater` en
> `tauri.conf.json`, sin claves de firma). Repo definitivo: `aresoftlabs/draffity`.

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

## Prerequisitos (en orden de dependencia)

1. **Releases públicas.** La app debe poder bajar manifiesto + instalador sin token.
   Hoy el repo es **privado** → los assets de release dan **404** sin auth (es la
   misma causa por la que la descarga de binarios whisper falla). Se resuelve
   haciendo el repo **público** (requiere el readiness de open-source: licencia con
   titular correcto, THIRD-PARTY-NOTICES, escaneo de secretos en el historial) **o**
   sirviendo los artefactos desde un host público aparte (release de un repo público
   dedicado, bucket/CDN, GitHub Pages).
2. **Par de claves de firma.** `tauri signer generate` → clave privada + password.
   - Privada + password como **secrets de CI** (`TAURI_SIGNING_PRIVATE_KEY`,
     `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`) en `aresoftlabs/draffity`.
   - Pública en `tauri.conf.json → plugins.updater.pubkey`.
   - ⚠️ La clave privada **no** se commitea y **no** se rota sin re-firmar todo.
3. **Build de los 3 OS en el pipeline de release.** Hoy `release.yml` buildea solo
   **Windows + Linux**; falta **macOS** (target M-chip). `tauri-action` puede generar
   el manifiesto del updater (`includeUpdaterJson: true`) y adjuntarlo al release.
4. **Plugin + config.** Agregar el plugin (Rust + JS), `plugins.updater` con
   `endpoints` apuntando al `latest.json` del release público, y la UI de "buscar/
   instalar actualización".

## Enfoque recomendado

- **Updater plugin de Tauri + GitHub Releases como endpoint** (camino más trillado):
  `tauri-action` con `includeUpdaterJson: true` publica `latest.json` + instaladores
  firmados en el release del tag `v*`. `endpoints` apunta a la URL pública de ese
  `latest.json`. Firma con **minisign** vía los secrets de CI.
- Mantener el `release.yml` actual (ya usa `tauri-action`); sumarle macOS, la firma,
  y `includeUpdaterJson`. Quitar `releaseDraft`/`prerelease` cuando se publique en serio.

## Checklist para implementar (cuando se decida)

- [ ] Decidir y ejecutar **releases públicas** (repo público con readiness, o host público).
- [ ] `tauri signer generate`; cargar privada+password como secrets de CI; pública en config.
- [ ] Agregar `tauri-plugin-updater` (Rust) y `@tauri-apps/plugin-updater` (JS).
- [ ] `plugins.updater` en `tauri.conf.json` (`endpoints` + `pubkey`).
- [ ] Sumar **macOS** a `release.yml` + `includeUpdaterJson: true` + firma.
- [ ] UI mínima: chequeo de updates, descarga, "reiniciar para actualizar".
- [ ] Probar el ciclo end-to-end (vX → vX+1) en las 3 plataformas.

## Bloqueante actual relacionado

Mientras el repo siga **privado**, tanto el auto-update como la **descarga de binarios
de voz** (whisper) fallan con 404 para clientes sin token. Resolver la distribución
pública desbloquea **ambos** a la vez.
