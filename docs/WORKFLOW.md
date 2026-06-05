# Workflow de contribución

Modelo de ramas, convención de commits y flujo de PR/release de Draffity.

## Ramas

| Rama                                                                                | Rol                              | Protegida | Recibe                                    |
| ----------------------------------------------------------------------------------- | -------------------------------- | --------- | ----------------------------------------- |
| `main`                                                                              | Releases (se taggea acá)         | ✅        | **fast-forward** desde `develop` (sin PR) |
| `develop`                                                                           | Integración (**default branch**) | ✅        | PR desde ramas de trabajo                 |
| `feat/*` `fix/*` `chore/*` `docs/*` `refactor/*` `test/*` `perf/*` `ci/*` `build/*` | Trabajo                          | —         | —                                         |

- Las ramas de trabajo **nacen de `develop`** y se nombran con el **tipo del Conventional Commit** + descripción corta: `feat/binder-drag`, `fix/autosave-debounce`, `docs/workflow`.
- `develop` es el **default branch**: tus PR apuntan ahí por defecto.
- `main` es **siempre un fast-forward** de un commit verde de `develop` (sin merge commits), así ambas quedan idénticas tras cada release.

## Convención de commits

[Conventional Commits](https://www.conventionalcommits.org/) — **enforzado por `commitlint`** (header ≤ 100 chars):

```
<tipo>(<scope opcional>)<!>: <descripción>
```

- **Tipos:** `feat`, `fix`, `chore`, `docs`, `refactor`, `test`, `perf`, `ci`, `build`, `style`.
- **Scope** (opcional): área tocada, p. ej. `voice`, `editor`, `ui`, `export`.
- **Breaking change:** `!` tras el tipo/scope (`feat(voice)!: ...`).
- **DCO obligatorio:** firmá cada commit con `git commit -s` (agrega `Signed-off-by`). Ver [CONTRIBUTING](../CONTRIBUTING.md).

## Flujo de PR

1. `git switch develop && git pull` → crear `feat/lo-que-sea`.
2. Commits chicos y enfocados (Conventional Commits + `-s`).
3. PR **hacia `develop`**. El CI corre lint + tests + coverage + EPUB + E2E.
4. Merge cuando el **CI está verde** (no se exige approval por ahora; se sumará con más maintainers). Historia lineal (squash/rebase).

## Flujo de release

`main` se actualiza por **fast-forward** desde un commit verde de `develop` — nunca
tiene merge commits, así `main` y `develop` quedan **idénticas** tras cada release.

1. Verificá que `develop` esté verde (el CI de su último PR).
2. Fast-forward de `main`:
   ```bash
   git checkout main
   git merge --ff-only develop
   git push origin main
   ```
3. Taggear la versión:
   ```bash
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```
4. El tag dispara `.github/workflows/release.yml` (publica el release). **`develop` nunca publica.**

El build de instalables corre en el push a `main` como **smoke** (no bloquea el
fast-forward — `develop` ya validó lint/tests/coverage/EPUB/E2E con los mismos commits).

## Hotfix

Arreglo urgente: rama `fix/*` desde `develop` → PR a `develop` (CI verde) → release
(fast-forward de `main` + tag de patch). Para una emergencia sobre una versión vieja
mientras `develop` tiene trabajo sin publicar, cherry-pickeá el fix a una rama de
release desde el tag (caso avanzado).

## Protección de ramas (resumen)

Ambas: **status checks verdes** + historia **lineal** + sin force-push ni borrado.

- `develop` (default): recibe PRs de ramas de trabajo (PR obligatorio, sin approval por ahora).
  Checks: lint/test JS+Rust · coverage · EPUB · E2E.
- `main`: **fast-forward de `develop`** (sin PR ni merge commit). Los mismos 5 checks ya
  corrieron en `develop` sobre el mismo commit; el build corre como smoke en el push.

Ver `.github/workflows/ci.yml`.
