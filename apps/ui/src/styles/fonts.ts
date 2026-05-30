// Self-hosted variable fonts so the desktop app renders identically offline,
// independent of what the user has installed. Importing for side effects only;
// each package injects its own @font-face rules at build time.
//   Inter Variable        → UI (menús, botones, inspector)
//   Fraunces Variable     → display/títulos (sello editorial)
//   Source Serif 4 Variable → texto de lectura del manuscrito
import '@fontsource-variable/inter';
import '@fontsource-variable/fraunces';
import '@fontsource-variable/source-serif-4';
