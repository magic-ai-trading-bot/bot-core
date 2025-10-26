/// <reference types="vitest" />
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import path from 'path'

export default defineConfig({
  plugins: [
    react({
      jsxRuntime: 'automatic',
      jsxImportSource: 'react',
    }),
  ],
  define: {
    'import.meta.env.MODE': JSON.stringify('test'),
    'import.meta.env.DEV': true,
    'import.meta.env.PROD': false,
    'import.meta.env.VITE_ENABLE_REALTIME': '"false"',
    'import.meta.env.VITE_WS_URL': '"ws://localhost:8080/ws"',
    'import.meta.env.VITE_API_URL': '"http://localhost:8080"',
    'process.env.NODE_ENV': JSON.stringify('test'),
  },
  test: {
    globals: true,
    environment: 'jsdom',
    setupFiles: ['./src/test/setup.ts'],
    include: ['src/__tests__/**/*.{test,spec}.{ts,tsx}'],
    testTimeout: 5000,
    hookTimeout: 5000,
    teardownTimeout: 5000,
    exclude: [
      'node_modules/**',
      'dist/**',
      'e2e/**',
      '**/*.e2e.{test,spec}.{ts,tsx}',
      '**/*.spec.ts', // Exclude Playwright spec files
    ],
    pool: 'forks',
    poolOptions: {
      forks: {
        singleFork: true,
        isolate: true,
      },
    },
    maxConcurrency: 1,
    fileParallelism: false,
    maxWorkers: 1,
    minWorkers: 1,
    sequence: {
      shuffle: false,
    },
    coverage: {
      provider: 'v8',
      reporter: ['text', 'json', 'html'],
      exclude: [
        'node_modules/',
        'src/test/',
        '**/*.d.ts',
        '**/*.config.*',
        '**/index.ts',
        'src/main.tsx',
        'public/',
        'dist/',
      ],
      thresholds: {
        global: {
          branches: 30,
          functions: 30,
          lines: 30,
          statements: 30,
        },
      },
    },
  },
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
})