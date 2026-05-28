import js from '@eslint/js';
import ts from '@typescript-eslint/eslint-plugin';
import tsParser from '@typescript-eslint/parser';
import vue from 'eslint-plugin-vue';
import vueParser from 'vue-eslint-parser';
import globals from 'globals';

export default [
  {
    ignores: [
      '**/node_modules/**',
      '**/dist/**',
      '**/target/**',
      '**/coverage/**',
      '**/playwright-report/**',
      '**/test-results/**',
      'apps/desktop/gen/**',
      '**/components.d.ts',
    ],
  },
  js.configs.recommended,
  {
    files: ['apps/ui/src/**/*.{ts,mts,cts,vue}'],
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: { ...globals.browser },
    },
    plugins: { '@typescript-eslint': ts },
    rules: {
      ...ts.configs.recommended.rules,
      '@typescript-eslint/no-unused-vars': ['warn', { argsIgnorePattern: '^_' }],
      'no-undef': 'off',
    },
  },
  {
    files: ['apps/ui/src/**/*.vue'],
    languageOptions: {
      parser: vueParser,
      parserOptions: {
        parser: tsParser,
        ecmaVersion: 'latest',
        sourceType: 'module',
        extraFileExtensions: ['.vue'],
      },
      globals: { ...globals.browser },
    },
    plugins: { vue, '@typescript-eslint': ts },
    rules: {
      ...vue.configs['flat/recommended'].rules,
      'vue/multi-word-component-names': 'off',
      'no-undef': 'off',
      // Bloquea strings hardcoded en <template>. Toda UI debe pasar por i18n.
      // Ver CLAUDE.md §5 + backlog v2 S0-08.
      'vue/no-bare-strings-in-template': [
        'error',
        {
          allowlist: [
            // Puntuación y símbolos universales
            '(',
            ')',
            ',',
            '.',
            '&',
            '+',
            '-',
            '=',
            '*',
            '/',
            '#',
            '%',
            '!',
            '?',
            ':',
            ';',
            "'",
            '"',
            '·',
            '–',
            '—',
            '…',
            '×',
            '✓',
            '✗',
            // Tokens UI no-traducibles (convención tipográfica universal o unidades SI)
            'H1',
            'H2',
            'H3',
            'H4',
            'H5',
            'H6',
            'U',
            'S',
            'B',
            'I',
            'R',
            'C',
            'ms',
            'px',
            'em',
            'rem',
            // Etiquetas de teclas (idénticas en ES/EN)
            'Esc',
            'Tab',
            'Enter',
            'Space',
            'Ctrl',
            'Shift',
            'Alt',
            'Cmd',
            'Meta',
          ],
          attributes: {
            '/.+/': [
              'title',
              'aria-label',
              'aria-placeholder',
              'aria-roledescription',
              'aria-valuetext',
              'placeholder',
            ],
          },
          directives: ['v-text'],
        },
      ],
    },
  },
  {
    files: [
      '**/*.config.{ts,js,mjs,cjs}',
      '**/vite.config.ts',
      '**/tailwind.config.ts',
      '**/postcss.config.js',
      '**/eslint.config.mjs',
      '**/commitlint.config.js',
    ],
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: { ...globals.node },
    },
    plugins: { '@typescript-eslint': ts },
    rules: {
      'no-undef': 'off',
    },
  },
  {
    files: ['packages/shared-types/**/*.ts'],
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 'latest',
      sourceType: 'module',
    },
    plugins: { '@typescript-eslint': ts },
    rules: {
      ...ts.configs.recommended.rules,
    },
  },
  {
    files: ['apps/ui/e2e/**/*.ts'],
    languageOptions: {
      parser: tsParser,
      ecmaVersion: 'latest',
      sourceType: 'module',
      globals: { ...globals.node, ...globals.browser },
    },
    plugins: { '@typescript-eslint': ts },
    rules: {
      ...ts.configs.recommended.rules,
      'no-undef': 'off',
      // Playwright fixtures legitimately use Function constructor for serialization.
      '@typescript-eslint/no-implied-eval': 'off',
      '@typescript-eslint/no-explicit-any': 'off',
    },
  },
];
