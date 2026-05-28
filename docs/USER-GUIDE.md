# Draffity — Guía de usuario (alpha)

> Versión gratuita. Todo el contenido se almacena localmente en tu equipo.

## Instalación

- **Windows**: descarga `draffity_<version>_x64_en-US.msi` y ejecuta el instalador.
- **Linux**: descarga `draffity_<version>_amd64.AppImage`, dale permisos de ejecución (`chmod +x ...`) y ábrelo.

Al primer arranque verás un onboarding con 3 pantallas — síguelo o sáltalo.

## Crear tu primer proyecto

1. Pulsa **Crear nuevo proyecto** en el dashboard.
2. **Paso 1 — Plantilla**: elige entre `Genérico`, `Novela en tres actos`, `Paper IMRaD` o `Manga shōnen`. Verás una previsualización de la estructura inicial.
3. **Paso 2 — Datos**: introduce el título y completa los metadatos que pida la plantilla (autor, género, etc.).
4. **Paso 3 — Confirmar**: revisa el resumen y pulsa **Crear**.

El proyecto se abre con la estructura ya generada en el binder izquierdo.

## La regla del proyecto activo

Draffity gratuito permite **un proyecto activo** a la vez. Los demás quedan **archivados en solo lectura**:

- Puedes seguir leyendo y exportando proyectos archivados.
- No puedes editar texto en archivados.
- Para volver a editar uno archivado, simplemente actívalo: el actual se archivará automáticamente. No se pierde ningún dato.

## Editor

- Tres paneles: **Binder** (estructura) · **Editor** (texto) · **Inspector** (metadatos + versiones).
- Auto-guardado tras dejar de escribir (configurable en Ajustes, por defecto 500 ms).
- Atajos:
  - `Ctrl+S` — forzar guardado inmediato
  - `Ctrl+N` — nuevo capítulo
  - `Ctrl+B` / `Ctrl+I` / `Ctrl+U` — formato (negrita / cursiva / subrayado)

## Versiones (snapshots)

En el Inspector, sección **Versiones**:

- Pulsa **+** para guardar la versión actual del documento, opcionalmente con etiqueta.
- Pasa el cursor sobre una versión y pulsa el icono **↺** para restaurarla.
- Cuando restauras, **el estado anterior se guarda automáticamente** como una versión `auto-restore`, así que puedes volver atrás.

## Exportar

1. En la cabecera del proyecto, pulsa **Exportar**.
2. Elige formato: `Markdown`, `Word (DOCX)` o `EPUB`.
3. Selecciona dónde guardar el archivo.

El export procesa el árbol completo del proyecto en orden de aparición.

## Ajustes

- **Tema** — claro, oscuro o según el sistema.
- **Idioma** — Español o English (toda la UI).
- **Fuente del editor** — serif (Lora), sans (Inter) o monospace (JetBrains Mono).
- **Intervalo de autoguardado** — 200 ms a 3 segundos.
- **Hábito de escritura** — racha actual y más larga (días consecutivos).

## Datos y privacidad

- Todo se guarda en tu disco local: una sola base SQLite en `<carpeta de datos de la app>/draffity.db` (en Windows: `%APPDATA%\cl.aresoft.draffity`).
- Los logs van a `<carpeta de datos>/logs/draffity.log` con rotación diaria.
- No hay telemetría ni envío a servidores externos en la versión gratuita.

## Problemas conocidos en alpha

- Reordenar nodos en el binder por arrastrar todavía no está disponible.
- Export PDF llegará en una próxima iteración.
- macOS no está soportado en esta alpha (se planea para post-MVP).

## Reportar un bug

Mira en `<carpeta de datos>/logs/draffity.log` y abre un issue en el repositorio adjuntando el log y los pasos para reproducir.
