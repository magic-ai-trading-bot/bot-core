/**
 * Polyfill localStorage for Node.js environment
 * This MUST run before MSW (or any other module) is loaded
 *
 * MSW v2 creates a global CookieStore at module load time which requires localStorage.
 * This polyfill ensures localStorage is available in the global scope before any modules load.
 */

// Create a proper Storage implementation
class LocalStorageMock {
  constructor() {
    this.store = new Map()
  }

  getItem(key) {
    return this.store.get(key) ?? null
  }

  setItem(key, value) {
    this.store.set(key, String(value))
  }

  removeItem(key) {
    this.store.delete(key)
  }

  clear() {
    this.store.clear()
  }

  key(index) {
    return Array.from(this.store.keys())[index] ?? null
  }

  get length() {
    return this.store.size
  }
}

// Set localStorage globally BEFORE any modules load
if (typeof global !== 'undefined') {
  if (!global.localStorage) {
    global.localStorage = new LocalStorageMock()
  }
  if (!global.sessionStorage) {
    global.sessionStorage = new LocalStorageMock()
  }
}

if (typeof globalThis !== 'undefined') {
  if (!globalThis.localStorage) {
    globalThis.localStorage = new LocalStorageMock()
  }
  if (!globalThis.sessionStorage) {
    globalThis.sessionStorage = new LocalStorageMock()
  }
}

console.log('✓ localStorage polyfill loaded')
