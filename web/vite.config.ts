import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import wasm from "vite-plugin-wasm";
import eslint from "vite-plugin-eslint";
import path from "path";

export default defineConfig({
  plugins: [react(), wasm(), eslint()],
  server: {
    port: 3000,
  },
  build: {
    sourcemap: true,
  },
  resolve: {
    alias: {
      "@pkg-mediawiki": path.resolve(__dirname, "./pkg-mediawiki"),
      "@pkg-opencc": path.resolve(__dirname, "./pkg-opencc"),
      "@pkg-both": path.resolve(__dirname, "./pkg-both"),
    },
  },
});

