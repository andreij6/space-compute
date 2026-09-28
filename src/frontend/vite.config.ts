import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react';
import { icpBindgen } from '@icp-sdk/bindgen/plugins/vite';

export const CANISTERS = ['platform', 'payments', 'aaa', 'treasury'];

export default defineConfig({
  plugins: [
    react(),
    ...CANISTERS.map((c) => icpBindgen({ didFile: `../../crates/${c}/${c}.did`, outDir: './src/bindings' })),
  ],
  server: { port: 3000 },
  build: { outDir: 'dist', sourcemap: true },
  test: { include: ['src/**/*.test.ts'] },
});
