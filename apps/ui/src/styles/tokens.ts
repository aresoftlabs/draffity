/**
 * Única fuente de verdad de la paleta cálida de Draffity (spec §3.1).
 * Consumida por `preset.ts` para construir el preset de PrimeVue. No importar
 * valores hex sueltos en componentes: usar utilidades Tailwind `surface-*` /
 * `primary-*` o tokens `--p-*`, que derivan de acá.
 */

/** Rampa de acento terracota. Claro usa 500 (#b5651d); oscuro usa 400 (#d08a4a). */
export const WARM_PRIMARY = {
  '50': '#fbf4ec',
  '100': '#f7e6d2',
  '200': '#efc9a0',
  '300': '#e3a86e',
  '400': '#d08a4a',
  '500': '#b5651d',
  '600': '#9a560f',
  '700': '#7c450e',
  '800': '#5f360f',
  '900': '#4d2c12',
  '950': '#2a1707',
} as const;

/**
 * Rampa de superficies sepia, de papel (0/50) a carbón cálido (900/950).
 * Se aplica idéntica a los esquemas claro y oscuro: las pantallas eligen el
 * extremo con `bg-surface-50 dark:bg-surface-950`, igual que hoy, pero cálido.
 */
export const WARM_SURFACE = {
  '0': '#ffffff',
  '50': '#faf7f0',
  '100': '#f3ecdd',
  '200': '#ece3cf',
  '300': '#ddd0b5',
  '400': '#c9b896',
  '500': '#a99a78',
  '600': '#7a6e57',
  '700': '#4a4338',
  '800': '#322c22',
  '900': '#221e17',
  '950': '#1b1813',
} as const;
