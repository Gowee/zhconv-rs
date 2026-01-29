import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import path from "path";

export default defineConfig({
  plugins: [react(), wasm(), topLevelAwait()],
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
  test: {
    environment: "jsdom",
    setupFiles: ["./src/setupTests.ts"],
  },
});

