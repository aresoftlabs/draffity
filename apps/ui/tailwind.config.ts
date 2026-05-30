import type { Config } from 'tailwindcss';
import primeui from 'tailwindcss-primeui';

export default {
  content: ['./index.html', './src/**/*.{vue,ts,tsx}'],
  darkMode: ['selector', '.app-dark'],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          'Inter Variable',
          'Inter',
          'ui-sans-serif',
          'system-ui',
          '-apple-system',
          'Segoe UI',
          'Roboto',
          'sans-serif',
        ],
        // Prosa / lectura del manuscrito.
        serif: ['Source Serif 4 Variable', 'Source Serif 4', 'Lora', 'Georgia', 'serif'],
        reading: ['Source Serif 4 Variable', 'Source Serif 4', 'Georgia', 'serif'],
        // Títulos y nombres de proyecto — el sello editorial.
        display: ['Fraunces Variable', 'Fraunces', 'Georgia', 'serif'],
        mono: ['JetBrains Mono', 'Menlo', 'Consolas', 'monospace'],
      },
    },
  },
  plugins: [primeui],
} satisfies Config;
