# Draffity — Guía de usuario (alpha)

> Aplicación 100% gratuita. Todo el contenido se almacena localmente en tu equipo.

## Instalación

- **Windows**: descarga `Draffity_<version>_x64-setup.exe` y ejecútalo (instalador NSIS; se instala para tu usuario, sin pedir permisos de administrador).
- **Linux**: descarga `Draffity_<version>_amd64.AppImage`, dale permisos de ejecución (`chmod +x ...`) y ábrelo.

La app se **actualiza sola**: al abrir avisa si hay una versión nueva y te deja instalarla con un clic.

Al primer arranque verás un onboarding con 3 pantallas — síguelo o sáltalo.

## Crear tu primer proyecto

1. Pulsa **Crear nuevo proyecto** en el dashboard.
2. **Paso 1 — Plantilla**: elige entre `Genérico`, `Novela en tres actos`, `Paper IMRaD` o `Manga shōnen`. Verás una previsualización de la estructura inicial.
3. **Paso 2 — Datos**: introduce el título y completa los metadatos que pida la plantilla (autor, género, etc.).
4. **Paso 3 — Confirmar**: revisa el resumen y pulsa **Crear**.

El proyecto se abre con la estructura ya generada en el binder izquierdo.

## La regla del proyecto activo

Draffity permite **un proyecto activo** a la vez. Los demás quedan **archivados en solo lectura**:

- Puedes seguir leyendo y exportando proyectos archivados.
- No puedes editar texto en archivados.
- Para volver a editar uno archivado, simplemente actívalo: el actual se archivará automáticamente. No se pierde ningún dato.

## Editor

- Tres paneles: **Binder** (estructura) · **Editor** (texto) · **Inspector** (metadatos + versiones).
- Auto-guardado tras dejar de escribir (configurable en Ajustes, por defecto 500 ms).
- **Binder con drag&drop**: arrastra capítulos y escenas para reordenarlos o moverlos entre carpetas. Los cambios se guardan al soltar.
- **Modo enfoque**: oculta el binder y el inspector para escribir sin distracciones. Botón en la cabecera o `F11`.
- Atajos:
  - `Ctrl+S` — forzar guardado inmediato
  - `Ctrl+N` — nuevo capítulo
  - `Ctrl+B` / `Ctrl+I` / `Ctrl+U` — formato (negrita / cursiva / subrayado)
  - `Ctrl+F` — buscar en el documento actual
  - `Ctrl+H` — buscar y reemplazar en el documento actual
  - `Ctrl+Shift+F` — buscar en todo el proyecto (full-text)
  - `F11` — modo enfoque on/off

## Búsqueda

- **En el documento (Ctrl+F)**: barra encima del editor con campo de búsqueda. `Enter` salta al siguiente match, `Shift+Enter` al anterior, `Esc` cierra.
- **Reemplazo (Ctrl+H)**: la misma barra con campo extra "Reemplazar con…" más los botones `Reemplazar` y `Reemplazar todo`.
- **Cross-proyecto (Ctrl+Shift+F)**: diálogo modal que busca en títulos y contenido de todos los documentos del proyecto activo, con snippets resaltados. Click en un resultado salta al documento.

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

## IA (opcional, BYOK)

Draffity puede asistir tu escritura con acciones de IA (Continuar, Expandir, Reescribir, Describir) y con validadores analíticos. Para activarlas necesitás tu propia API key de [OpenRouter](https://openrouter.ai/):

1. Creá una cuenta en openrouter.ai y generá una API key.
2. En **Ajustes → IA y modelos**, pegá la key en el campo correspondiente.
3. La key se guarda en el keyring del sistema operativo, nunca en texto plano.

Sin key la app es completamente funcional — la IA es un complemento opcional.

## Voz (opcional, binarios locales)

El dictado (Whisper) y la lectura en voz alta (Piper) no están incluidos en el instalador base. Para activarlos:

1. En **Ajustes → Audio y voz**, usá el botón **Descargar modelo** para instalar el modelo de Whisper (≈570 MB, se descarga con barra de progreso y verificación de checksum).
2. El binario de Piper se descarga junto al modelo de voz TTS predeterminado.
3. Una vez descargados, el dictado (`Ctrl+Shift+M`) y la lectura en voz alta (`Ctrl+Shift+R`) quedan habilitados.

Podés también importar binarios manualmente si tenés los archivos ya descargados.

## Datos y privacidad

- Todo se guarda en tu disco local: una sola base SQLite en `<carpeta de datos de la app>/draffity.db` (en Windows: `%APPDATA%\cl.aresoft.draffity`).
- Los logs van a `<carpeta de datos>/logs/draffity.log` con rotación diaria.
- No hay telemetría ni envío a servidores externos.

## Problemas conocidos en alpha

- Export PDF llegará en una próxima iteración.
- macOS no está soportado en esta alpha (se planea para post-MVP).

## Reportar un bug

Mira en `<carpeta de datos>/logs/draffity.log` y abre un issue en el repositorio adjuntando el log y los pasos para reproducir.
