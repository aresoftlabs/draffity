import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import Components from 'unplugin-vue-components/vite';
import { PrimeVueResolver } from '@primevue/auto-import-resolver';
import { fileURLToPath, URL } from 'node:url';

// Tauri expone variables que ajustan dev server
const host = process.env.TAURI_DEV_HOST;

export default defineConfig(() => ({
  plugins: [
    vue(),
    Components({
      resolvers: [PrimeVueResolver()],
      dts: 'src/components.d.ts',
    }),
  ],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url)),
    },
  },
  // Vite ignora console.log de Tauri si no se enfoca aquí.
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Tauri reconstruye sólo cuando cambia src de Rust
      ignored: ['**/src-tauri/**', '**/apps/desktop/**'],
    },
  },
  build: {
    target: ['es2022', 'chrome110'],
    sourcemap: !!process.env.TAURI_DEBUG,
    minify: process.env.TAURI_DEBUG ? false : ('esbuild' as const),
  },
  test: {
    environment: 'happy-dom',
    globals: true,
    include: ['src/**/*.{test,spec}.{ts,js}', 'test/**/*.{test,spec}.{ts,js}'],
    setupFiles: ['./test/setup.ts'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'json-summary'],
      // We measure the surfaces that hold logic — `composables/` and
      // `stores/`. Everything else (Vue components, services boundary,
      // generated types) is exercised by E2E or has too much templating
      // to benefit from a line-coverage gate.
      include: ['src/composables/**/*.ts', 'src/stores/**/*.ts'],
      // Exclude the tests themselves so they don't inflate the numerator.
      exclude: ['**/*.test.ts', '**/*.spec.ts'],
      // Floor thresholds: locked to roughly the current measured
      // coverage (lines ~33%, functions ~55%, branches ~80%). The
      // backlog target is 70% lines and the plan is to ratchet up the
      // gate as we add tests for the untested stores/composables. The
      // important property today is regression-catching: dropping below
      // these numbers fails CI.
      thresholds: {
        lines: 30,
        functions: 55,
        statements: 30,
        branches: 70,
      },
    },
  },
}));
