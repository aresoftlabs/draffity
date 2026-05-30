import { definePreset } from '@primevue/themes';
import Aura from '@primevue/themes/aura';
import { WARM_PRIMARY, WARM_SURFACE } from './tokens';

/**
 * Overrides sobre el preset Aura. Se exporta por separado de `DraffityPreset`
 * para poder testear los valores sin depender de los internals de `definePreset`.
 * Aura usa la misma rampa `surface` en claro y oscuro (las pantallas eligen el
 * extremo), así que la aplicamos idéntica a ambos esquemas.
 */
export const presetOverrides = {
  semantic: {
    primary: { ...WARM_PRIMARY },
    colorScheme: {
      light: { surface: { ...WARM_SURFACE } },
      dark: { surface: { ...WARM_SURFACE } },
    },
  },
} as const;

/** Preset cálido de Draffity. Reemplaza a `Aura` en `main.ts`. */
export const DraffityPreset = definePreset(Aura, presetOverrides);
