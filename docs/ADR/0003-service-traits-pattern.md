# ADR-0003 — Service traits pattern for additive architecture

- **Estado**: Historical
- **Fecha**: 2026-05-08 (retroactivo)
- **Sprint / Épica**: Fase 1 del backlog v1

> **Nota**: Este ADR documenta el patrón de traits de servicio que permite
> añadir capacidades sin modificar el core. El patrón arquitectónico sigue vigente.

## Contexto

Draffity necesita una arquitectura donde distintas capacidades (IA, sync, ASR, etc.)
puedan integrarse sin modificar el núcleo de la aplicación. La pregunta arquitectónica
es **cómo permitir que nuevo código llegue sin tocar las capas existentes**.

Anti-patrones comunes que queremos evitar:

1. Condicionales de tipo de feature salpicados por todo el código de negocio. Cada
   nueva capacidad requeriría editar N archivos del core. Capacidades y core quedan
   acoplados.
2. Versiones paralelas de los mismos archivos (`storage_a.rs`, `storage_b.rs`) con
   duplicación masiva y derivas.
3. Gates de feature visibles en la UI para capacidades desactivadas.

## Decisión

Implementar el contrato:

> **Las capacidades son aditivas. Nunca degradan la base. Nunca reemplazan
> implementaciones existentes. Siempre añaden nuevas implementaciones de traits ya
> definidos.**

Concretamente:

1. Toda capa de servicios con potencial de extensión se define como `trait`
   (`StorageService`, `ExportService`, `AIService`, `ASRService`,
   `TemplatesService`). Cada trait tiene una impl local funcional.
2. Extensiones implementan los mismos traits con backends/lógica adicional. La factory
   de servicios elige la impl según la configuración activa.
3. Feature flags pasan por un único punto de consulta.
4. Migraciones de nuevas capacidades viven en rangos numerados y se aplican
   selectivamente.

## Consecuencias

### Positivas

- Cero edits en archivos del core para añadir cualquier capacidad nueva. Verificable:
  todas marcadas "no toca core".
- Tests del core siguen verdes con o sin capacidades adicionales activadas.
- Capacidades pueden desarrollarse en paralelo sin merge conflicts.

### Negativas / costos

- Definir traits cuesta más al principio que escribir la lógica directa. Hay que
  pensar la abstracción.
- `Arc<dyn Trait>` tiene overhead vs llamada estática. Despreciable en la práctica
  para una app de escritura — no es un hot loop.

### Neutras (futuras opciones)

- Runtime switching de impls: la `ServiceFactory` puede swapear impls sin reiniciar
  la app.

## Alternativas consideradas

### Feature flags inline

Más rápido de escribir al principio.

**Descartado** porque contamina el core con condicionales, dificulta tests del core
y la promesa "nuevas capacidades nunca degradan el core" se vuelve imposible de auditar.

### "Plugin loader" dinámico (cargar `.so`/`.dll`)

Más extensible para terceros.

**Descartado** porque la complejidad es enorme para el beneficio actual (ABI estable Rust,
firma de plugins, sandboxing). Si en el futuro hace sentido, el patrón "trait + impl"
ya está listo para envolverse en un plugin loader sin reescribir el core.

### Capacidades como apps separadas

Garantiza no contaminación.

**Descartado** porque duplica el binario y genera fricción enorme para integrar capacidades.

## Referencias

- Service traits pattern: `apps/desktop/src/services/`
- Stubs que materializan el patrón: `apps/desktop/src/services/ai.rs`, `sync.rs`, `asr.rs`
- Trait + impl para multi-active: `ProjectManager::archive_active_if_needed_except`
