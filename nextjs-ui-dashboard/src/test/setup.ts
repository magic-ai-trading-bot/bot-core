import '@testing-library/jest-dom'
import { afterEach, beforeAll, afterAll } from 'vitest'
import { cleanup } from '@testing-library/react'
import { server } from './mocks/server'

// Setup MSW
beforeAll(() => server.listen({ onUnhandledRequest: 'warn' }))
afterAll(() => server.close())
afterEach(() => {
  server.resetHandlers()
  cleanup()
  localStorage.clear()
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

// Mock HTMLFormElement.requestSubmit
if (typeof HTMLFormElement.prototype.requestSubmit !== 'function') {
  HTMLFormElement.prototype.requestSubmit = function(submitter?: HTMLElement) {
    if (submitter) {
      submitter.click()
    } else {
      this.dispatchEvent(new Event('submit', { cancelable: true, bubbles: true }))
    }
  }
}