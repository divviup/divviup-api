import { defineConfig } from "vite";
import eslint from "vite-plugin-eslint";
import react from "@vitejs/plugin-react-swc";
import postcssNesting from "postcss-nesting";

export default defineConfig(() => ({
  build: {
    sourcemap: true,
    outDir: process.env.BUILD_PATH || "build",
  },
  plugins: [react(), eslint()],
  server: {
    port: 8081,
    strictPort: true,
    watch: {
      // Workaround for workstations using MacOS and Colima as the container runtime. inotify does
      // work reliably for virtiofs volumes, so fall back to polling. This comes at the cost of
      // higher CPU usage.
      usePolling: true,
    },
  },
  css: {
    postcss: {
      plugins: [postcssNesting],
    },
  },
}));
