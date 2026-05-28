# Release Checklist — v0.x.y-alpha

Pasos manuales para verificar antes de cortar una release alpha.

## 1. Pre-flight (local)

- [ ] `pnpm install` limpio sin warnings críticos.
- [ ] `pnpm fmt:check` y `pnpm lint` verdes.
- [ ] `pnpm typecheck` verde.
- [ ] `pnpm test` (Vitest) verde.
- [ ] `cargo test --manifest-path apps/desktop/Cargo.toml` verde.
- [ ] `cargo clippy --manifest-path apps/desktop/Cargo.toml --all-targets -- -D warnings` verde.
- [ ] `pnpm tauri:build` produce un MSI en `apps/desktop/target/release/bundle/msi/` y/o un AppImage en `apps/desktop/target/release/bundle/appimage/`.
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
- [ ] Cambiar idioma a English y de vuelta a Español. Toda la UI cambia.
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

- [ ] Commit de release: `chore(release): vX.Y.Z-alpha`.
- [ ] `git tag vX.Y.Z-alpha`.
- [ ] `git push origin main --tags`.
- [ ] El workflow `release.yml` (`tauri-action`) correrá la matrix `windows-latest` + `ubuntu-latest` y subirá los artefactos a la GitHub Release como **draft**.
- [ ] Editar la draft release con las release notes (copiar de `CHANGELOG.md`).
- [ ] Marcar como `pre-release` (mientras siga en alpha/beta).
- [ ] Publicar.

## 5. Post-release

- [ ] Verificar descargas: bajar el MSI/AppImage de la release y reproducir el smoke test E2E.
- [ ] Anunciar en el canal correspondiente.
- [ ] Crear nuevo issue de tracking para la siguiente versión.

## Rollback

Si se descubre un bug bloqueante después de publicar:

1. Mover la release a `draft` (los assets dejan de ser públicos).
2. Comunicar en el canal pidiendo a los usuarios no actualizar.
3. Preparar fix → bump patch → repetir el flujo.
