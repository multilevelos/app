import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import wasm from 'vite-plugin-wasm';

export default defineConfig({
  base: './',
  plugins: [svelte(), wasm()],
  build: {
    target: 'esnext',
    rollupOptions: {
      output: {
        manualChunks: {
          maplibre: ['maplibre-gl']
        }
      }
    }
  },
  server: {
    port: 5173,
    host: true
  }
});
