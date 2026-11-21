import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
export default defineConfig({
  plugins: [react()],
  base: "./",
  build: { rollupOptions: { input: {
    main: "./index.html",
    simulator: "./battle-simulator.html",
  } } },
  server: { port: 5173, strictPort: true },
});
