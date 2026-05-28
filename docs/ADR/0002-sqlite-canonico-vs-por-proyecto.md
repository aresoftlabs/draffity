# ADR-0002 — SQLite canónica única vs una DB por proyecto

- **Estado**: Accepted
- **Fecha**: 2026-05-08 (retroactivo)
- **Sprint / Épica**: Fase 1 del backlog v1

## Contexto

El diseño inicial del MVP (escrito antes de implementar) contemplaba "una DB SQLite por proyecto" en `~/.draffity/projects/<id>/project.db`. Al implementar la Fase 1, el código real adoptó **una sola DB canónica** en `<app_data_dir>/draffity.db` que contiene todos los proyectos del usuario. Este ADR documenta retroactivamente por qué.

Restricciones que aparecieron en implementación:

1. La regla "1 proyecto activo + N archivados read-only" (invariante del free) tiene que enforzarse de forma robusta. La defensa en profundidad usa un `UNIQUE INDEX idx_projects_one_active ON projects(status) WHERE status='active'`. Este índice **solo funciona si todos los proyectos viven en la misma tabla, en la misma DB**.
2. Features futuras (búsqueda full-text cross-proyecto en Sprint 1 v2, codex compartible en Sprint 7 v2, stats agregadas) son sensiblemente más simples con una sola DB.
3. Backup automático (Sprint 6 v2) es un único archivo, no N carpetas que coordinar.
4. Premium multi-active se logra con un `DROP INDEX` en una migración — sin reorganizar ficheros en disco.

## Decisión

Adoptar **una sola DB canónica** en `<app_data_dir>/draffity.db` con todos los proyectos del usuario y enforzar la regla del free mediante índice parcial UNIQUE en SQL + validación en `ProjectManager` (defensa en profundidad).

## Consecuencias

### Positivas

- Invariante "1 activo" enforzado a nivel SQL **sin coordinación entre archivos**. Imposible saltarlo accidentalmente desde el código.
- Backup, sync y restore son operaciones sobre un único archivo.
- Búsqueda full-text cross-proyecto (FTS5) y stats agregadas son SQL normal.
- Migraciones aplicadas una sola vez por arranque, no N veces.
- Premium multi-active = migración aditiva `DROP INDEX idx_projects_one_active`. No requiere mover datos.

### Negativas / costos

- Un proyecto corrupto en la DB podría arrastrar a todos. Mitigación: WAL + backup automático (Sprint 6 v2) + snapshots por documento ya implementados.
- Tamaño del archivo crece sin límite explícito por proyecto. Un proyecto de 1M palabras puede ralentizar queries que no usen índices adecuados. Mitigación: índices en `documents(project_id, parent_id, position)` ya implementados; benchmark de FTS pendiente para Sprint 1 v2.
- Compartir un proyecto individual entre dispositivos (sin sync premium) requiere export/import explícito, no copiar un fichero suelto.

### Neutras

- Cloud sync premium puede ser por-proyecto (sync incremental con diff) o por-DB. La decisión es independiente.

## Alternativas consideradas

### Una DB por proyecto (`~/.draffity/projects/<id>/project.db`)

Diseño inicial documentado en `ARCHITECTURE.md` antes de implementar.

**Descartado** porque: para enforzar "1 activo" habría que mantener un archivo índice separado (`~/.draffity/projects.json` o similar) coordinado con N DBs. Cualquier crash entre escribir el índice y escribir la DB del proyecto deja inconsistencia. Defensa en profundidad imposible — el invariante vive solo en código de aplicación, no en el storage. Además, search/stats cross-proyecto requieren attach-multiple en SQLite o agregación en código.

### Híbrido (DB canónica para índice + DBs por proyecto para contenido)

Combina lo peor de los dos mundos: complejidad de DB múltiples + necesidad de mantener consistencia entre DBs.

**Descartado** sin demasiada discusión.

## Referencias

- Implementación: `apps/desktop/src/lib.rs:42-45`, `apps/desktop/src/services/storage.rs`
- Índice partial UNIQUE: `apps/desktop/src/migrations/001_init.sql:25-27`
- Test que verifica el enforcement a nivel SQL: `apps/desktop/src/services/storage.rs` (`unique_active_project_constraint_enforced_by_db`)
- Plan premium multi-active: `docs/PREMIUM-INTEGRATION.md` sección "Multi-proyecto"
