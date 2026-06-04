# Workflow de contribución

Modelo de ramas, convención de commits y flujo de PR/release de Draffity.

## Ramas

| Rama                                                                                | Rol                              | Protegida | Recibe                                     |
| ----------------------------------------------------------------------------------- | -------------------------------- | --------- | ------------------------------------------ |
| `main`                                                                              | Releases (se taggea acá)         | ✅        | PR **solo** desde `develop` o `hotfix/*`   |
| `develop`                                                                           | Integración (**default branch**) | ✅        | PR desde ramas de trabajo                  |
| `feat/*` `fix/*` `chore/*` `docs/*` `refactor/*` `test/*` `perf/*` `ci/*` `build/*` | Trabajo                          | —         | —                                          |
| `hotfix/*`                                                                          | Arreglo urgente sobre release    | —         | PR a `main` (luego back-merge a `develop`) |

- Las ramas de trabajo **nacen de `develop`** y se nombran con el **tipo del Conventional Commit** + descripción corta: `feat/binder-drag`, `fix/autosave-debounce`, `docs/workflow`.
- `develop` es el **default branch**: tus PR apuntan ahí por defecto.

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

1. PR **`develop` → `main`**. Acá el CI corre **además** el build de instalables (Win/Linux) y el **job guardián** (que rechaza PRs a `main` que no vengan de `develop`/`hotfix/*`).
2. Merge a `main` con CI verde.
3. Taggear en `main`: `git tag vX.Y.Z && git push origin vX.Y.Z`.
4. El tag dispara `.github/workflows/release.yml` (publica el release). **`develop` nunca publica.**

## Hotfix

`hotfix/x` desde `main` → PR a `main` → merge → tag → **back-merge a `develop`** (`git switch develop && git merge main`).

## Protección de ramas (resumen)

`main` y `develop` requieren PR + **status checks verdes** + historia lineal; sin force-push ni borrado. `main` exige además los checks de Build + el guardián. Ver `.github/workflows/ci.yml`.
