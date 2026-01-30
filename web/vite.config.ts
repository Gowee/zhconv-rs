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
      "@pkg-opencc": path.resolve(__dirname, "./pkg-opencc"),
      "@pkg-default": path.resolve(__dirname, "./pkg-default"),
    },
  },
});

