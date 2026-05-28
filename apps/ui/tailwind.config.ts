import type { Config } from 'tailwindcss';
import primeui from 'tailwindcss-primeui';

export default {
  content: ['./index.html', './src/**/*.{vue,ts,tsx}'],
  darkMode: ['selector', '.app-dark'],
  theme: {
    extend: {
      fontFamily: {
        sans: [
          'Inter',
          'ui-sans-serif',
          'system-ui',
          '-apple-system',
          'Segoe UI',
          'Roboto',
          'sans-serif',
        ],
        serif: ['Lora', 'Georgia', 'serif'],
        mono: ['JetBrains Mono', 'Menlo', 'Consolas', 'monospace'],
      },
    },
  },
  plugins: [primeui],
} satisfies Config;
