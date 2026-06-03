# ADR-0006 — Draffity es 100% gratis

- **Estado**: Accepted
- **Fecha**: 2026-06-02

## Contexto

La arquitectura original de Draffity separaba el modelo de producto de las features,
pero esa capa de abstracción resultó innecesaria. Las features que requieren IA o voz
ya tienen un prerrequisito funcional real (`hasKey` para OpenRouter, `binaryInstalled`
para Whisper/Piper). El gate extra era una capa adicional sin valor para el usuario.

## Decisión

Todas las features quedan disponibles incondicionalmente.

Concretamente:

1. Se eliminan `enum Tier`, `enum Capability`, `TierService`, `CapabilityGate`,
   `LicenseValidator` y los comandos IPC asociados.
2. Se elimina `CloudSyncService` / `NoOpSync`.
3. Se elimina `TemplateTier` — todas las plantillas built-in son accesibles.
4. Los servicios pasan a depender **únicamente** del prerrequisito funcional real
   (clave de API presente, binario + modelo instalado).
5. Se eliminan los componentes de UI y claves i18n asociadas al modelo anterior.
6. Se conservan los **traits de servicio** (`AIService`, `ASRService`, `TTSService`,
   `ProjectManagerService`, etc.) porque son buen diseño de arquitectura.
7. Se conserva el invariante **"1 proyecto activo a la vez"** como decisión de UX/diseño.
   El índice SQL `idx_projects_one_active` permanece.

## Consecuencias

### Positivas

- El código se simplifica: desaparecen módulo capabilities y su andamiaje de tests.
- IA y voz responden al estado real del entorno del usuario.
- Todas las plantillas están disponibles para todos.

### Negativas / costos

- El viejo patrón de extensiones ya no existe. Si en el futuro se quisiera reintroducir
  algo similar, habría que diseñarlo desde cero.
- Los traits de servicio se mantienen como base arquitectónica.

## Alternativas consideradas

### Mantener el andamiaje pero abrir todos los gates

**Descartado** porque deja código muerto que contradice el diseño actual y confunde
a futuros desarrolladores.

### Eliminar solo el gate de UI y dejar el backend intacto

**Descartado** porque crearía una contradicción entre el comportamiento declarado
(100% gratis) y el código real.

## Referencias

- Invariante "1 activo": `apps/desktop/src/migrations/001_init.sql` — índice `idx_projects_one_active`
- Prerrequisito funcional de IA: `apps/desktop/src/services/ai_openrouter.rs` — `available()`
- Prerrequisito funcional de voz: `apps/desktop/src/services/voice/whisper.rs`, `piper.rs` — `available()`
