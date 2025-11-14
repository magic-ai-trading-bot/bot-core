/**
 * Vitest Global Setup
 * This runs ONCE before all test files and before environment setup
 * Sets up polyfills that MSW and other libraries need
 */

export async function setup() {
  // Polyfill localStorage for Node.js environment BEFORE anything else
  // MSW v2 requires this to be available globally
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

  // Set on global object (for Node.js)
  const localStorageInstance = new LocalStorageMock()
  const sessionStorageInstance = new LocalStorageMock()

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (global as any).localStorage = localStorageInstance;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (global as any).sessionStorage = sessionStorageInstance;

  // Also set on globalThis
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (globalThis as any).localStorage = localStorageInstance;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  (globalThis as any).sessionStorage = sessionStorageInstance

  // eslint-disable-next-line no-console
  console.log('✓ Global test environment initialized with localStorage polyfill')
}

export async function teardown() {
  // Global cleanup logic (runs once after all tests)
}
