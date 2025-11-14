/**
 * Vitest global setup - runs BEFORE any imports
 * This file must set up DOM APIs that MSW and other libraries depend on
 */

// CRITICAL: Set up localStorage IMMEDIATELY, before ANY imports (including vitest)
// MSW v2 requires localStorage to be available at module load time
if (typeof globalThis.localStorage === 'undefined' || typeof globalThis.localStorage.getItem !== 'function') {
  class LocalStorageMock implements Storage {
    private store = new Map<string, string>()

    getItem(key: string): string | null {
      return this.store.get(key) ?? null
    }

    setItem(key: string, value: string): void {
      this.store.set(key, String(value))
    }

    removeItem(key: string): void {
      this.store.delete(key)
    }

    clear(): void {
      this.store.clear()
    }

    key(index: number): string | null {
      return Array.from(this.store.keys())[index] ?? null
    }

    get length(): number {
      return this.store.size
    }
  }

  // Set on both globalThis and global
  const localStorageInstance = new LocalStorageMock()
  const sessionStorageInstance = new LocalStorageMock()

  Object.defineProperty(globalThis, 'localStorage', {
    value: localStorageInstance,
    writable: true,
    configurable: true,
  })

  Object.defineProperty(globalThis, 'sessionStorage', {
    value: sessionStorageInstance,
    writable: true,
    configurable: true,
  })

  // Also set on global for Node.js compatibility
  if (typeof global !== 'undefined') {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (global as any).localStorage = localStorageInstance;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (global as any).sessionStorage = sessionStorageInstance
  }
}

// NOW import vitest after localStorage is set up
import { vi } from 'vitest'

// Ensure window.localStorage and window.sessionStorage are properly set
// jsdom may set them to incomplete implementations
if (typeof window !== 'undefined') {
  // Reapply our full Storage implementation to window object
  if (!window.localStorage || typeof window.localStorage.clear !== 'function') {
    const localStorageInstance = globalThis.localStorage || global.localStorage
    Object.defineProperty(window, 'localStorage', {
      value: localStorageInstance,
      writable: true,
      configurable: true,
    })
  }

  if (!window.sessionStorage || typeof window.sessionStorage.clear !== 'function') {
    const sessionStorageInstance = globalThis.sessionStorage || global.sessionStorage
    Object.defineProperty(window, 'sessionStorage', {
      value: sessionStorageInstance,
      writable: true,
      configurable: true,
    })
  }
}

// Mock window.location
// eslint-disable-next-line @typescript-eslint/no-explicit-any
delete (window as any).location
window.location = {
  href: 'http://localhost:3000',
  origin: 'http://localhost:3000',
  protocol: 'http:',
  host: 'localhost:3000',
  hostname: 'localhost',
  port: '3000',
  pathname: '/',
  search: '',
  hash: '',
  assign: vi.fn(),
  reload: vi.fn(),
  replace: vi.fn(),
  toString: () => 'http://localhost:3000',
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
} as any

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  error: vi.fn(),
  warn: vi.fn(),
}
