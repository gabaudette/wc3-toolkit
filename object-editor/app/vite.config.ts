import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, searchForWorkspaceRoot } from "vite";

/** Port the Tauri window loads in development (see src-tauri/tauri.conf.json). */
const DEV_PORT = 1420;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: DEV_PORT,
    strictPort: true,
    fs: {
      // The "What's new" dialog bundles CHANGELOG.md from the repository root.
      allow: [searchForWorkspaceRoot(process.cwd()), "../../CHANGELOG.md"],
    },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
