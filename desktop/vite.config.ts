import { svelte } from "@sveltejs/vite-plugin-svelte";
import { defineConfig } from "vite";

const DEVELOPMENT_PORT = 1420;

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  server: { port: DEVELOPMENT_PORT, strictPort: true },
});
