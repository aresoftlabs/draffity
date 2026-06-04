# Third-Party Notices

Draffity (GPLv3) distribuye o depende de los siguientes componentes de terceros.
Sus licencias se respetan y sus textos están disponibles en sus proyectos.

## Componentes empaquetados en el binario distribuido

| Componente                       | Uso                             | Licencia             |
| -------------------------------- | ------------------------------- | -------------------- |
| whisper.cpp                      | Transcripción de voz (ASR)      | MIT                  |
| ggml                             | Backend de inferencia (whisper) | MIT                  |
| Piper                            | Síntesis de voz (TTS)           | MIT                  |
| **espeak-ng** (+ espeak-ng-data) | Fonemización para Piper         | **GPLv3**            |
| Modelos ggml de Whisper          | Modelos ASR                     | MIT (OpenAI Whisper) |
| Silero VAD (silero-v5.1.2-ggml)  | Detección de actividad de voz   | MIT                  |
| Fuente Fraunces                  | Tipografía de títulos           | SIL OFL 1.1          |
| Fuente Inter                     | Tipografía de UI                | SIL OFL 1.1          |

> **Nota de compliance:** espeak-ng es GPLv3. Como Draffity también es GPLv3, la
> distribución del binario combinado es coherente con sus términos.

## Dependencias de build (no necesariamente empaquetadas)

Stack Rust (Tauri, reqwest, serde, rusqlite, etc.) y JS (Vue, Pinia, PrimeVue,
Tailwind, TipTap/ProseMirror, Vite) bajo licencias permisivas (MIT / Apache-2.0).
El detalle completo se genera con `cargo tree` y `pnpm licenses list`.
