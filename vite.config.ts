import { defineConfig } from "vite";
import solid from "vite-plugin-solid";
import tailwindcss from "@tailwindcss/vite";

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [solid(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: "ws",
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
  build: {
    // WebView2 on Windows tracks Chromium; WebKit on macOS and Linux is older.
    target: process.env.TAURI_ENV_PLATFORM === "windows" ? "chrome105" : "safari15",
    minify: "esbuild",
    sourcemap: false,
    // Keep SVG piece sets as standalone files so their licenses stay separable
    // from the application bundle and the WebView can cache each piece.
    assetsInlineLimit: 0,
  },
});
