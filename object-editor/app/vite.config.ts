import tailwindcss from "@tailwindcss/vite";
import react from "@vitejs/plugin-react";
import { defineConfig, searchForWorkspaceRoot } from "vite";

/** Port the Tauri window loads in development (see src-tauri/tauri.conf.json). */
const DEV_PORT = 1420;

const REACT = /node_modules[\\/](react|react-dom|scheduler)[\\/]/;
const REACT_ARIA =
  /node_modules[\\/](@react-aria|@react-stately|@react-types|@internationalized|react-aria|react-aria-components|react-stately)[\\/]/;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  build: {
    rolldownOptions: {
      output: {
        codeSplitting: {
          groups: [
            { name: "react", test: REACT },
            { name: "react-aria", test: REACT_ARIA },
          ],
        },
      },
    },
  },
  server: {
    port: DEV_PORT,
    strictPort: true,
    fs: {
      // The "What's new" dialog bundles CHANGELOG.md and the assistant prompt bundles
      // docs/assistant-prompt.md, both from the repository root.
      allow: [
        searchForWorkspaceRoot(process.cwd()),
        "../../CHANGELOG.md",
        "../../docs/assistant-prompt.md",
      ],
    },
    watch: {
      ignored: ["**/src-tauri/**"],
    },
  },
});
