import path from "node:path";
import { fileURLToPath } from "node:url";

import { defineConfig, loadEnv } from "vite";
import react from "@vitejs/plugin-react";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = path.resolve(__dirname, "..");

export default defineConfig(({ mode }) => {
  // Load repo-root `.env` so `HOST` matches the Axum bind address (same as `cargo run` from the workspace).
  const env = loadEnv(mode, repoRoot, "");
  const rawHost = env.HOST?.trim() || "127.0.0.1:15173";
  const proxyTarget = /^https?:\/\//i.test(rawHost) ? rawHost : `http://${rawHost}`;

  return {
    plugins: [react()],
    envDir: repoRoot,
    server: {
      port: 15174,
      strictPort: true,
      // Same-origin `/v1/*` in the browser; forward to the Axum backend during `npm run dev`.
      proxy: {
        "/v1": { target: proxyTarget, changeOrigin: true },
        "/healthz": { target: proxyTarget, changeOrigin: true },
      },
    },
  };
});

