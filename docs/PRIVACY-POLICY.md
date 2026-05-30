# Política de privacidad — Draffity

_Última actualización: 2026-05-28._

Draffity es una aplicación de escritorio para escritores. Está
diseñada con un principio simple: **tus textos son tuyos y se quedan
en tu computadora**.

## Qué datos guarda Draffity

Toda la información que ingresas vive **exclusivamente en tu equipo**
dentro del directorio de datos del sistema operativo:

- Manuscritos, capítulos, escenas y notas (archivo `draffity.db`).
- Snapshots históricos de cada documento.
- Imágenes y fuentes que subes a un proyecto (carpeta `media/`).
- Backups automáticos diarios y mensuales (carpeta `backups/`).
- Tu configuración del editor (tema, idioma, fuente, atajos).
- Estadísticas locales de escritura (rachas, palabras por día).

Ningún dato se sube a un servidor remoto. No hay cuenta, no hay
sincronización en la nube, no hay analítica.

## Qué datos NO captura Draffity

- **Telemetría de uso**: no enviamos qué botones tocas ni cuántas
  veces abres la app.
- **Crash reports**: por defecto están desactivados. Si los activas
  explícitamente desde Settings → Privacidad, Draffity enviará el
  stack trace y la versión de la app a la URL configurada por el
  build (Sentry self-hosted bajo control del owner del binario). No
  se incluye el contenido de tus documentos.
- **Identificadores**: no generamos UUID de usuario ni huella digital
  del dispositivo.

## Plugins de terceros

Draffity usa estos componentes que pueden tocar la red sólo si tú
los invocas:

- **Tauri Shell**: abrir archivos exportados con el navegador o
  editor predeterminado.
- **Tauri Dialog**: file pickers nativos del sistema.
- **Tauri FS**: leer/escribir archivos en rutas que tú eliges.

Ninguno de ellos transmite datos sin una acción tuya.

## Tu derecho a borrar

Para eliminar todos los datos de Draffity borra la carpeta
`<app_data>/Draffity` (la ruta exacta depende de tu sistema
operativo — `~/Library/Application Support/Draffity/` en macOS,
`%APPDATA%\Draffity\` en Windows, `~/.local/share/Draffity/` en
Linux). El uninstaller del binario no toca esa carpeta para que un
upgrade no te haga perder trabajo.

## Cambios a esta política

Si esta política cambia, la fecha de "Última actualización" arriba
se actualiza y el CHANGELOG del release menciona el cambio.

## Contacto

Bugs y consultas sobre privacidad: abre una issue en el repositorio
de Draffity.

---

[English version](./PRIVACY-POLICY.en.md)
