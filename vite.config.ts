import { defineConfig } from 'vite';
import vue from '@vitejs/plugin-vue';
import checker from 'vite-plugin-checker';
import * as path from 'path';

export default defineConfig({
  plugins: [
    vue(),
    checker({
      typescript: true
    })
  ],
  publicDir: path.resolve(__dirname, 'public'),
  resolve: {
    alias: {
      path: 'path-browserify',
      '@fift': path.resolve(__dirname, 'fift-wasm/pkg')
    }
  },
  optimizeDeps: {
    include: ['path-browserify', 'onigasm', 'typescript', 'monaco-editor-core/esm/vs/editor/editor.worker']
  }
});
