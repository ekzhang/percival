import { resolve } from "path";
import { defineConfig } from "vite";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import { buildParserFile } from "@lezer/generator";

/** A simple custom Rollup-compatible plugin to import Lezer grammars. */
function lezer() {
  return {
    name: "rollup-plugin-lezer",
    transform(src, id) {
      if (/\.grammar$/.test(id)) {
        return {
          code: buildParserFile(src).parser,
          map: null,
        };
      }
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig({
  build: {
    target: "esnext",
    chunkSizeWarningLimit: 1500,
  },
  resolve: {
    alias: {
      "@": resolve("src"),
    },
  },
  plugins: [svelte(), lezer()],
  server: {
    proxy: {
      "/api": {
        target: "http://localhost:3030",
        changeOrigin: true,
      },
    },
  },
});
