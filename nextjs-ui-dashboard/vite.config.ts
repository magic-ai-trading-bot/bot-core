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
    // Proxy API calls to backend services
    proxy: {
      "/api": {
        target: "http://rust-core-engine-dev:8080",
        changeOrigin: true,
        secure: false,
        timeout: 30000,
        configure: (proxy, options) => {
          proxy.on("error", (err, req, res) => {
            console.log("Proxy error:", err);
          });
        },
      },
      "/ws": {
        target: "ws://rust-core-engine-dev:8080",
        ws: true,
        changeOrigin: true,
        secure: false,
      },
    },
  },
  // Define environment variables for development
  define: {
    // Define these for both development and production
    "import.meta.env.VITE_RUST_API_URL": JSON.stringify(
      process.env.VITE_RUST_API_URL || "http://localhost:8080"
    ),
    "import.meta.env.VITE_WS_URL": JSON.stringify(
      process.env.VITE_WS_URL || "ws://localhost:8080/ws"
    ),
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
