import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import path from "path";
import { visualizer } from "rollup-plugin-visualizer";

/**
 * Vite Configuration for Bundle Analysis
 *
 * This configuration extends the base vite.config.ts to add bundle
 * visualization and analysis capabilities. It generates an interactive
 * HTML report showing:
 * - Bundle composition
 * - Chunk sizes
 * - Dependency tree
 * - Optimization opportunities
 *
 * Usage: npm run build:analyze
 */
export default defineConfig({
  plugins: [
    react(),
    visualizer({
      filename: "./dist/stats.html",
      open: true, // Open in browser after build
      gzipSize: true, // Show gzip sizes
      brotliSize: true, // Show brotli sizes
      template: "treemap", // Options: treemap, sunburst, network
      sourcemap: false, // Don't include sourcemaps
    }),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  build: {
    target: "esnext",
    minify: "esbuild",
    sourcemap: true, // Generate sourcemaps for analysis
    rollupOptions: {
      external: ["ws"],
      output: {
        manualChunks: {
          // React core
          "react-vendor": ["react", "react-dom", "react-router-dom"],
          // React Query
          "query-vendor": ["@tanstack/react-query"],
          // UI Library - Radix primitives
          "radix-vendor": [
            "@radix-ui/react-dialog",
            "@radix-ui/react-dropdown-menu",
            "@radix-ui/react-select",
            "@radix-ui/react-tabs",
            "@radix-ui/react-toast",
            "@radix-ui/react-tooltip",
            "@radix-ui/react-popover",
            "@radix-ui/react-accordion",
            "@radix-ui/react-alert-dialog",
            "@radix-ui/react-avatar",
            "@radix-ui/react-checkbox",
            "@radix-ui/react-label",
            "@radix-ui/react-progress",
            "@radix-ui/react-scroll-area",
            "@radix-ui/react-separator",
            "@radix-ui/react-slider",
            "@radix-ui/react-switch",
          ],
          // Charts library
          "chart-vendor": ["recharts"],
          // 3D visualization libraries
          "three-vendor": ["three", "@react-three/fiber", "@react-three/drei"],
          // Form libraries
          "form-vendor": ["react-hook-form", "@hookform/resolvers", "zod"],
          // Utilities
          "utils-vendor": [
            "axios",
            "date-fns",
            "clsx",
            "class-variance-authority",
            "tailwind-merge",
          ],
        },
        // Optimize chunk size
        chunkFileNames: "assets/[name]-[hash].js",
        entryFileNames: "assets/[name]-[hash].js",
        assetFileNames: "assets/[name]-[hash].[ext]",
      },
    },
    chunkSizeWarningLimit: 500,
    reportCompressedSize: true,
  },
  // Strip console.logs and debugger statements
  esbuild: {
    drop: ["console", "debugger"],
  },
});
