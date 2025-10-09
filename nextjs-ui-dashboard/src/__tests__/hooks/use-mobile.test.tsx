import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { useIsMobile } from '../../hooks/use-mobile'

describe('useIsMobile', () => {
  let matchMediaMock: any
  let listeners: Array<(event: any) => void> = []

  beforeEach(() => {
    listeners = []

    matchMediaMock = vi.fn((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addEventListener: vi.fn((event: string, listener: (event: any) => void) => {
        listeners.push(listener)
      }),
      removeEventListener: vi.fn((event: string, listener: (event: any) => void) => {
        listeners = listeners.filter(l => l !== listener)
      }),
      dispatchEvent: vi.fn(),
    }))

    Object.defineProperty(window, 'matchMedia', {
      writable: true,
      configurable: true,
      value: matchMediaMock,
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
    listeners = []
  })

  it('initializes with false when window width is >= 768', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 1024,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(result.current).toBe(false)
  })

  it('initializes with true when window width is < 768', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 375,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(result.current).toBe(true)
  })

  it('returns true for mobile breakpoint (767px)', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 767,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(result.current).toBe(true)
  })

  it('returns false for desktop breakpoint (768px)', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 768,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(result.current).toBe(false)
  })

  it('updates when window is resized to mobile', async () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 1024,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(result.current).toBe(false)

    // Simulate resize to mobile
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 375,
    })

    // Trigger the change event
    listeners.forEach(listener => listener({ matches: true }))

    await waitFor(() => {
      expect(result.current).toBe(true)
    })
  })

  it('updates when window is resized to desktop', async () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 375,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(result.current).toBe(true)

    // Simulate resize to desktop
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 1024,
    })

    // Trigger the change event
    listeners.forEach(listener => listener({ matches: false }))

    await waitFor(() => {
      expect(result.current).toBe(false)
    })
  })

  it('uses correct media query', () => {
    renderHook(() => useIsMobile())

    expect(matchMediaMock).toHaveBeenCalledWith('(max-width: 767px)')
  })

  it('registers event listener on mount', () => {
    const { result } = renderHook(() => useIsMobile())

    const matchMediaInstance = matchMediaMock.mock.results[0].value
    expect(matchMediaInstance.addEventListener).toHaveBeenCalledWith(
      'change',
      expect.any(Function)
    )
  })

  it('removes event listener on unmount', () => {
    const { unmount } = renderHook(() => useIsMobile())

    const matchMediaInstance = matchMediaMock.mock.results[0].value

    unmount()

    expect(matchMediaInstance.removeEventListener).toHaveBeenCalledWith(
      'change',
      expect.any(Function)
    )
  })

  it('cleans up listeners properly', () => {
    const { unmount } = renderHook(() => useIsMobile())

    expect(listeners.length).toBeGreaterThan(0)

    unmount()

    expect(listeners.length).toBe(0)
  })

  it('returns boolean type', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 1024,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(typeof result.current).toBe('boolean')
  })

  it('handles rapid resize events', async () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 1024,
    })

    const { result } = renderHook(() => useIsMobile())

    // Simulate multiple rapid resizes
    for (let i = 0; i < 5; i++) {
      Object.defineProperty(window, 'innerWidth', {
        writable: true,
        configurable: true,
        value: i % 2 === 0 ? 375 : 1024,
      })

      listeners.forEach(listener => listener({ matches: i % 2 === 0 }))
    }

    await waitFor(() => {
      expect(typeof result.current).toBe('boolean')
    })
  })

  it('handles edge case at exact breakpoint boundary', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 768,
    })

    const { result } = renderHook(() => useIsMobile())

    // 768 should be considered desktop (not mobile)
    expect(result.current).toBe(false)
  })

  it('handles very small screen sizes', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 320,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(result.current).toBe(true)
  })

  it('handles very large screen sizes', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 3840,
    })

    const { result } = renderHook(() => useIsMobile())

    expect(result.current).toBe(false)
  })

  it('remains consistent across re-renders with same width', () => {
    Object.defineProperty(window, 'innerWidth', {
      writable: true,
      configurable: true,
      value: 1024,
    })

    const { result, rerender } = renderHook(() => useIsMobile())

    const firstValue = result.current

    rerender()

    expect(result.current).toBe(firstValue)
  })
})
