import { defineConfig, ViteDevServer } from "vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";
import { componentTagger } from "lovable-tagger";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => ({
  server: {
    host: "::",
    port: 3000,
    hmr: {
      // Fix WebSocket compatibility with Bun
      port: 24678,
      // Use polling if WebSocket fails
      clientPort: process.env.HMR_PORT ? parseInt(process.env.HMR_PORT) : 24678,
      // Enable overlay for better error display
      overlay: true,
    },
    // Improve Bun compatibility
    cors: true,
    strictPort: false,
  },
  plugins: [react(), mode === "development" && componentTagger()].filter(
    Boolean
  ),
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Bun compatibility optimizations
  optimizeDeps: {
    exclude: ["ws"],
  },
}));
