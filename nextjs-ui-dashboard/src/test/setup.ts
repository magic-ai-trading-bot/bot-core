import '@testing-library/jest-dom'
import { afterEach } from 'vitest'
import { cleanup } from '@testing-library/react'

/**
 * NOTE: MSW (Mock Service Worker) is currently disabled due to incompatibility
 * with vitest's test environment initialization.
 *
 * Issue: MSW v2 creates a global CookieStore at module load time that requires
 * localStorage, but jsdom's localStorage is not available until after the test
 * environment is fully initialized.
 *
 * Attempted solutions:
 * - Global setup scripts (run too late)
 * - Custom environment (MSW loads before environment setup)
 * - Node.js --require hooks (isolated per worker pool)
 * - Lazy MSW initialization (module-level exports still execute)
 *
 * Recommended fix: Upgrade MSW or use fetch mocking instead
 */

// MSW setup disabled - tests will use real fetch (will fail without backend)
// import { server } from './mocks/server'
// beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }))
// afterAll(() => server.close())

afterEach(() => {
  // server.resetHandlers()
  cleanup()
  // Clear storage after each test
  if (typeof localStorage !== 'undefined' && typeof localStorage.clear === 'function') {
    localStorage.clear()
  }
  if (typeof sessionStorage !== 'undefined' && typeof sessionStorage.clear === 'function') {
    sessionStorage.clear()
  }
})

// Mock ResizeObserver
global.ResizeObserver = class ResizeObserver {
  observe() {}
  unobserve() {}
  disconnect() {}
}

// Mock IntersectionObserver
global.IntersectionObserver = class IntersectionObserver {
  constructor() {}
  observe() {}
  unobserve() {}
  disconnect() {}
}

// Mock matchMedia
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: (query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: () => {},
    removeListener: () => {},
    addEventListener: () => {},
    removeEventListener: () => {},
    dispatchEvent: () => {},
  }),
})

// Mock WebSocket
global.WebSocket = class WebSocket {
  constructor(public url: string) {}
  send() {}
  close() {}
  addEventListener() {}
  removeEventListener() {}
  readyState = 1
  CONNECTING = 0
  OPEN = 1
  CLOSING = 2
  CLOSED = 3
} as unknown as typeof WebSocket

// Mock HTMLFormElement.requestSubmit - Force override jsdom stub
HTMLFormElement.prototype.requestSubmit = function(submitter?: HTMLElement) {
  if (submitter) {
    submitter.click()
  } else {
    this.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true }))
  }
}