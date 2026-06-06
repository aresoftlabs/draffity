# Release Checklist — v0.x.y (beta)

Pasos manuales para verificar antes de cortar una release. La distribución y el
auto-update se sirven desde Cloudflare R2 (`bins.draffity.com`), no desde GitHub
Releases — ver [`AUTO-UPDATE.md`](./AUTO-UPDATE.md).

## 1. Pre-flight (local)

- [ ] `pnpm install` limpio sin warnings críticos.
- [ ] `pnpm fmt:check` y `pnpm lint` verdes.
- [ ] `pnpm typecheck` verde.
- [ ] `pnpm test` (Vitest) verde.
- [ ] `cargo test --manifest-path apps/desktop/Cargo.toml` verde.
- [ ] `cargo clippy --manifest-path apps/desktop/Cargo.toml --all-targets -- -D warnings` verde.
- [ ] `pnpm tauri:build` produce un instalador NSIS en `apps/desktop/target/release/bundle/nsis/` (`*-setup.exe`) y/o un AppImage en `apps/desktop/target/release/bundle/appimage/`.
- [ ] El instalable abre la app sin crash.

## 2. Smoke test E2E manual

Ejecutar en el binario empaquetado (no en `tauri:dev`).

- [ ] Primera ejecución: aparece el onboarding de 3 pantallas.
- [ ] Saltar / completar onboarding: dashboard vacío con CTA "Crear nuevo proyecto".
- [ ] Crear proyecto **Novela en tres actos** → estructura inicial con 3 actos + capítulos seed.
- [ ] Escribir en el primer capítulo, esperar 1s, indicador pasa a "Guardado".
- [ ] Cerrar la app, reabrir → contenido íntegro.
- [ ] Volver al dashboard, crear segundo proyecto **Paper IMRaD** → modal advierte que el primero se archivará → confirmar.
- [ ] El primer proyecto aparece como **Solo lectura** en la grid de archivados.
- [ ] Abrir el archivado → editor en read-only, banner amarillo visible, botón export operativo.
- [ ] Reactivar el archivado → el segundo (paper) se archiva.
- [ ] **Export Markdown** del proyecto activo → abre en cualquier editor MD con frontmatter YAML.
- [ ] **Export DOCX** → abre en Word/LibreOffice con headings y formato.
- [ ] **Export EPUB** → opcional: validar con `epubcheck` (`epubcheck mi-novela.epub`).
- [ ] **Snapshot manual**: en Inspector pulsar `+`, dar etiqueta, guardar. Editar, luego pulsar restore. El contenido vuelve a la versión guardada y aparece un nuevo `auto-restore` snapshot.
- [ ] Cambiar el idioma global (English · Español · Français · Italiano · Português) y volver. Toda la UI (y la voz) cambia.
- [ ] Cambiar tema (claro / oscuro / sistema). Persiste tras reinicio.
- [ ] Eliminar un proyecto archivado → se elimina con confirmación.
- [ ] Revisar `<app_data>/logs/draffity.log` — al menos una línea `tracing initialised` está presente.

## 3. Versionado

- [ ] Bump version en:
  - [ ] `package.json` (raíz)
  - [ ] `apps/ui/package.json`
  - [ ] `apps/desktop/package.json`
  - [ ] `apps/desktop/Cargo.toml`
  - [ ] `apps/desktop/tauri.conf.json` (`version`)
- [ ] Actualizar `CHANGELOG.md` con la sección de la nueva versión y mover entradas desde `[Unreleased]`.

## 4. Tag y release

- [ ] Commit de release: `chore(release): vX.Y.Z`.
- [ ] `git tag vX.Y.Z`.
- [ ] `git push origin develop --tags` (la rama principal es `develop`).
- [ ] El workflow `release.yml` corre la matrix `windows-latest` + `ubuntu-latest` y, por plataforma:
  - buildea y **firma con minisign** los instaladores (NSIS `.exe` / `.AppImage`);
  - los sube a **Cloudflare R2** en `s3://bins-draffity/app/<version>/` (público vía `bins.draffity.com`);
  - genera un fragmento de manifiesto con `scripts/build-update-manifest.mjs`.
  - (`tauri-action` también crea una GitHub Release **draft / prerelease** como espejo de los binarios — opcional, no es el canal de updates.)
- [ ] El job `manifest` ensambla `app/stable/latest.json` y lo publica en R2. **Esto es lo que activa el auto-update** para los usuarios existentes.

## 5. Post-release

- [ ] Verificar que `https://bins.draffity.com/app/stable/latest.json` lista la nueva versión y apunta a los instaladores de `app/<version>/`.
- [ ] Bajar el instalador NSIS (`.exe`)/AppImage desde R2 (o [draffity.com](https://draffity.com)) y reproducir el smoke test E2E.
- [ ] Verificar el **ciclo de auto-update**: abrir una versión anterior y confirmar que detecta e instala la nueva.
- [ ] Anunciar en el canal correspondiente.
- [ ] Crear nuevo issue de tracking para la siguiente versión.

## Rollback

Si se descubre un bug bloqueante después de publicar:

1. **Republicar `app/stable/latest.json` apuntando a la versión anterior** (el manifiesto en R2 es la fuente de verdad del updater, no la GitHub Release). Con esto los clientes dejan de ofrecer la versión defectuosa.
2. Opcional: borrar los artefactos de `app/<version>/` de la versión rota.
3. Comunicar en el canal pidiendo a quienes ya actualizaron que esperen el fix.
4. Preparar fix → bump patch → repetir el flujo.
