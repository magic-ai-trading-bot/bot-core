/**
 * Custom Vitest environment that extends jsdom with proper localStorage
 * This ensures localStorage is available BEFORE any modules (including MSW) are loaded
 */
import type { Environment } from 'vitest'
import { populateGlobal } from 'vitest/environments'

// Create a proper localStorage implementation
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

export default <Environment>{
  name: 'jsdom-with-storage',
  transformMode: 'web',
  async setup(global) {
    // First, set up localStorage BEFORE jsdom initializes
    const localStorageInstance = new LocalStorageMock()
    const sessionStorageInstance = new LocalStorageMock()

    // Now load jsdom environment
    const { jsdom } = await import('jsdom')
    const { JSDOM } = jsdom

    const dom = new JSDOM('<!DOCTYPE html>', {
      pretendToBeVisual: true,
      url: 'http://localhost:3000',
    })

    // Populate globals from jsdom
    const globalNames = [
      'window',
      'document',
      'navigator',
      'HTMLElement',
      'Node',
      'Element',
      'Event',
      'EventTarget',
      'DocumentFragment',
      'MutationObserver',
      'CustomEvent',
      'DOMParser',
      'XMLHttpRequest',
      'fetch',
      'Request',
      'Response',
      'Headers',
      'URL',
      'URLSearchParams',
    ]

    for (const name of globalNames) {
      if (name in dom.window) {
        (global as any)[name] = (dom.window as any)[name]
      }
    }

    // CRITICAL: Set localStorage on ALL relevant global objects
    // Set on global object (Node.js global)
    (global as any).localStorage = localStorageInstance;
    (global as any).sessionStorage = sessionStorageInstance;

    // Set on globalThis
    (globalThis as any).localStorage = localStorageInstance;
    (globalThis as any).sessionStorage = sessionStorageInstance;

    // Set on window object
    Object.defineProperty(global.window, 'localStorage', {
      value: localStorageInstance,
      writable: true,
      configurable: true,
      enumerable: true,
    })

    Object.defineProperty(global.window, 'sessionStorage', {
      value: sessionStorageInstance,
      writable: true,
      configurable: true,
      enumerable: true,
    })

    return {
      teardown() {
        dom.window.close()
      },
    }
  },
}
