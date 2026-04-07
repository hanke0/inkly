import path from 'node:path';
import { fileURLToPath } from 'node:url';

import { defineConfig, loadEnv } from 'vite';
import react from '@vitejs/plugin-react';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, '..');

export default defineConfig(({ mode }) => {
  // Load repo-root `.env` so `HOST` matches the Axum bind address (same as `cargo run` from the workspace).
  const env = loadEnv(mode, repoRoot, '');
  const rawHost = env.HOST?.trim() || '127.0.0.1:15173';
  const proxyTarget = /^https?:\/\//i.test(rawHost)
    ? rawHost
    : `http://${rawHost}`;

  return {
    plugins: [react()],
    envDir: repoRoot,
    build: {
      rollupOptions: {
        output: {
          manualChunks(id) {
            if (!id.includes('node_modules')) {
              return undefined;
            }
            if (id.includes('/@tiptap/') || id.includes('/prosemirror-')) {
              return 'vendor-editor';
            }
            if (id.includes('/react/') || id.includes('/react-dom/')) {
              return 'vendor-react';
            }
            if (
              id.includes('/react-router-dom/') ||
              id.includes('/react-router/')
            ) {
              return 'vendor-router';
            }
            if (id.includes('/katex/')) {
              return 'vendor-katex';
            }
            if (
              id.includes('/marked/') ||
              id.includes('/turndown/') ||
              id.includes('/turndown-plugin-gfm/') ||
              id.includes('/chardet/')
            ) {
              return 'vendor-content';
            }
            if (id.includes('/dompurify/')) {
              return 'vendor-sanitize';
            }
            return 'vendor';
          },
        },
      },
    },
    server: {
      port: 15174,
      strictPort: true,
      // Same-origin `/v1/*` in the browser; forward to the Axum backend during `npm run dev`.
      proxy: {
        '/v1': { target: proxyTarget, changeOrigin: true },
        '/healthz': { target: proxyTarget, changeOrigin: true },
      },
    },
  };
});
