import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";

export default defineConfig(() => ({
  build: {
    sourcemap: true,
    outDir: process.env.BUILD_PATH || "build",
  },
  plugins: [react()],
  server: {
    port: 8081,
    strictPort: true,
  },
}));
