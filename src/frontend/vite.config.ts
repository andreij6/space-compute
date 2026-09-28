import { readFileSync } from 'node:fs';
import { loadEnv, type Plugin } from 'vite';
import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { icpBindgen } from '@icp-sdk/bindgen/plugins/vite';
import { renderHeaders } from './src/lib/csp';

export const CANISTERS = ['platform', 'payments', 'aaa', 'treasury'];

function headersPlugin(mode: string): Plugin {
  return {
    name: 'sc-headers',
    apply: 'build',
    generateBundle() {
      const template = readFileSync(new URL('./headers.template', import.meta.url), 'utf8');
      this.emitFile({ type: 'asset', fileName: '_headers', source: renderHeaders(template, loadEnv(mode, process.cwd(), 'VITE_')) });
    },
  };
}

export default defineConfig(({ mode }) => ({
  plugins: [
    react(),
    headersPlugin(mode),
    ...CANISTERS.map((c) => icpBindgen({ didFile: `../../crates/${c}/${c}.did`, outDir: './src/bindings' })),
  ],
  server: { port: 3000 },
  build: { outDir: 'dist', sourcemap: true, assetsInlineLimit: 0 },
  test: {
    include: ['src/**/*.test.{ts,tsx}'],
    coverage: {
      provider: 'v8',
      include: ['src/lib/**'],
      reporter: ['text', 'json-summary'],
      thresholds: { lines: 80 },
    },
  },
}));
