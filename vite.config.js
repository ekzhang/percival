import { resolve } from "path";
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    target: "esnext",
    chunkSizeWarningLimit: 2500,
  },
  resolve: {
    alias: {
      "@": resolve("src"),
    },
  },
  plugins: [svelte()],
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3030",
        changeOrigin: true,
      },
    },
  },
});
