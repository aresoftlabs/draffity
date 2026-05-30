/**
 * Paleta tonal cálida y desaturada para las portadas de tarjeta del Dashboard
 * (spec §6.1). Tonos suaves dentro de la familia cálida — nada saturado.
 */
export const COVER_TONES = [
  '#e7dcc4',
  '#dde2d6',
  '#e3d9d0',
  '#e4dcc8',
  '#dad7c6',
  '#e6d6cd',
] as const;

/** Suma simple de char codes para indexar la paleta de forma determinística. */
function hash(id: string): number {
  let acc = 0;
  for (let i = 0; i < id.length; i++) acc = (acc + id.charCodeAt(i)) % 1000;
  return acc;
}

/** Tono de portada determinístico para un proyecto, a partir de su id. */
export function coverTone(id: string): string {
  return COVER_TONES[hash(id) % COVER_TONES.length];
}
