import { defineConfig, ViteDevServer } from "vite";
import react from "@vitejs/plugin-react";
import path from "path";
import { componentTagger } from "lovable-tagger";

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => ({
  server: {
    host: "::",
    port: 3000,
    // Allow ngrok and other tunnel hosts
    allowedHosts: [
      "120d-42-114-130-123.ngrok-free.app",
      "localhost",
      "127.0.0.1",
      "0.0.0.0",
      ".ngrok-free.app",
      ".ngrok.app",
      ".ngrok.io",
      ".tunnel.me",
      ".localtunnel.me",
    ],
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
        target: process.env.VITE_RUST_API_URL || "http://localhost:8080",
        changeOrigin: true,
        secure: false,
        timeout: 30000,
        configure: (proxy, options) => {
          proxy.on("error", (err, req, res) => {
            // Proxy errors are logged by Vite automatically
          });
        },
      },
      "/ws": {
        target: process.env.VITE_WS_URL?.replace('/ws', '') || "ws://localhost:8080",
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
    // Fix for Node.js globals in browser
    global: "globalThis",
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
    include: ["react", "react-dom"],
    // Force pre-bundling of problematic dependencies
    force: true,
  },
  // Additional configuration for Node.js compatibility
  build: {
    target: 'esnext',
    minify: 'esbuild',
    rollupOptions: {
      external: ["ws"],
      output: {
        manualChunks: {
          // React core
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          // React Query
          'query-vendor': ['@tanstack/react-query'],
          // UI Library - Radix primitives
          'radix-vendor': [
            '@radix-ui/react-dialog',
            '@radix-ui/react-dropdown-menu',
            '@radix-ui/react-select',
            '@radix-ui/react-tabs',
            '@radix-ui/react-toast',
            '@radix-ui/react-tooltip',
            '@radix-ui/react-popover',
            '@radix-ui/react-accordion',
            '@radix-ui/react-alert-dialog',
            '@radix-ui/react-avatar',
            '@radix-ui/react-checkbox',
            '@radix-ui/react-label',
            '@radix-ui/react-progress',
            '@radix-ui/react-scroll-area',
            '@radix-ui/react-separator',
            '@radix-ui/react-slider',
            '@radix-ui/react-switch',
          ],
          // Charts library
          'chart-vendor': ['recharts'],
          // 3D visualization libraries
          'three-vendor': ['three', '@react-three/fiber', '@react-three/drei'],
          // Form libraries
          'form-vendor': ['react-hook-form', '@hookform/resolvers', 'zod'],
          // Utilities
          'utils-vendor': ['axios', 'date-fns', 'clsx', 'class-variance-authority', 'tailwind-merge'],
        },
        // Optimize chunk size
        chunkFileNames: 'assets/[name]-[hash].js',
        entryFileNames: 'assets/[name]-[hash].js',
        assetFileNames: 'assets/[name]-[hash].[ext]',
      },
    },
    chunkSizeWarningLimit: 500,
    reportCompressedSize: true,
  },
  // Strip console.logs and debugger statements in production
  esbuild: {
    drop: mode === "production" ? ["console", "debugger"] : [],
  },
  // Test configuration
  test: {
    globalSetup: './vitest.globalSetup.ts', // Global setup (runs before everything)
    environment: './vitest-environment-jsdom-with-storage.ts', // Custom environment with localStorage
    environmentOptions: {
      jsdom: {
        resources: 'usable',
      },
    },
    setupFiles: [
      './src/test/vitest-setup.ts', // Global setup FIRST (before any imports)
      './src/test/setup.ts',        // Main test setup with MSW
    ],
    globals: true,
    css: true,
    pool: 'forks', // Use forks instead of threads for better isolation
    poolOptions: {
      forks: {
        singleFork: false,
      },
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov', 'json'],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.test.{ts,tsx}',
        '**/*.spec.{ts,tsx}',
        '**/mocks/**',
        '**/*.config.{ts,js}',
        '**/dist/**',
      ],
      thresholds: {
        lines: 90,
        functions: 90,
        branches: 90,
        statements: 90,
      },
    },
  },
}));
