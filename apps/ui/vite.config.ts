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
  },
}));
