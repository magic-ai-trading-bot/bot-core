import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { ThemeProvider, useTheme } from '../../contexts/ThemeContext'
import React from 'react'

// Mock localStorage
const createMockStorage = () => {
  let store: Record<string, string> = {}
  return {
    getItem: vi.fn((key: string) => store[key] || null),
    setItem: vi.fn((key: string, value: string) => {
      store[key] = value
    }),
    removeItem: vi.fn((key: string) => {
      delete store[key]
    }),
    clear: vi.fn(() => {
      store = {}
    }),
    key: vi.fn((index: number) => Object.keys(store)[index] || null),
    length: 0,
  }
}

// Mock matchMedia
const createMockMatchMedia = (matches: boolean) => {
  const listeners: Array<(e: MediaQueryListEvent) => void> = []
  return {
    matches,
    media: '(prefers-color-scheme: dark)',
    addEventListener: vi.fn((event: string, handler: any) => {
      listeners.push(handler)
    }),
    removeEventListener: vi.fn((event: string, handler: any) => {
      const index = listeners.indexOf(handler)
      if (index > -1) listeners.splice(index, 1)
    }),
    dispatchEvent: vi.fn((event: Event) => {
      listeners.forEach(listener => listener(event as MediaQueryListEvent))
      return true
    }),
    // Helper for tests
    _trigger: (matches: boolean) => {
      const event = { matches } as MediaQueryListEvent
      listeners.forEach(listener => listener(event))
    },
    _listeners: listeners,
  }
}

describe('ThemeContext', () => {
  let mockStorage: ReturnType<typeof createMockStorage>
  let mockMatchMedia: ReturnType<typeof createMockMatchMedia>

  beforeEach(() => {
    // Setup mocks
    mockStorage = createMockStorage()
    Object.defineProperty(window, 'localStorage', {
      value: mockStorage,
      writable: true,
      configurable: true,
    })

    mockMatchMedia = createMockMatchMedia(false) // default: light mode
    Object.defineProperty(window, 'matchMedia', {
      value: vi.fn(() => mockMatchMedia),
      writable: true,
      configurable: true,
    })

    // Mock document.documentElement for theme application
    Object.defineProperty(document, 'documentElement', {
      value: {
        classList: {
          add: vi.fn(),
          remove: vi.fn(),
        },
      },
      writable: true,
      configurable: true,
    })

    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.clearAllMocks()
    vi.useRealTimers()
  })

  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <ThemeProvider>{children}</ThemeProvider>
  )

  it('initializes with system theme by default', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('light') // matchMedia.matches = false
  })

  it('resolves system theme to dark when OS preference is dark', () => {
    mockMatchMedia = createMockMatchMedia(true) // dark mode
    Object.defineProperty(window, 'matchMedia', {
      value: vi.fn(() => mockMatchMedia),
      writable: true,
    })

    const { result } = renderHook(() => useTheme(), { wrapper })

    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('dark')
  })

  it('loads theme from localStorage on init', () => {
    mockStorage.getItem.mockReturnValue('dark')

    const { result } = renderHook(() => useTheme(), { wrapper })

    expect(result.current.theme).toBe('dark')
    expect(result.current.resolvedTheme).toBe('dark')
  })

  it('sets theme to light', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('light')
    })

    expect(result.current.theme).toBe('light')
    expect(result.current.resolvedTheme).toBe('light')
    expect(mockStorage.setItem).toHaveBeenCalledWith('theme', 'light')
  })

  it('sets theme to dark', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('dark')
    })

    expect(result.current.theme).toBe('dark')
    expect(result.current.resolvedTheme).toBe('dark')
    expect(mockStorage.setItem).toHaveBeenCalledWith('theme', 'dark')
  })

  it('sets theme to system', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('system')
    })

    expect(result.current.theme).toBe('system')
    expect(mockStorage.setItem).toHaveBeenCalledWith('theme', 'system')
  })

  it('applies dark class to document root when dark theme', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('dark')
    })

    act(() => {
      vi.runAllTimers()
    })

    expect(document.documentElement.classList.add).toHaveBeenCalledWith('dark')
  })

  it('removes dark class from document root when light theme', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    // First set to dark
    act(() => {
      result.current.setTheme('dark')
    })

    act(() => {
      vi.runAllTimers()
    })

    // Then set to light
    act(() => {
      result.current.setTheme('light')
    })

    act(() => {
      vi.runAllTimers()
    })

    expect(document.documentElement.classList.remove).toHaveBeenCalledWith('dark')
  })

  it('listens to system theme changes', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    // Start with system theme
    act(() => {
      result.current.setTheme('system')
    })

    expect(result.current.resolvedTheme).toBe('light')

    // Trigger OS theme change to dark
    act(() => {
      mockMatchMedia._trigger(true)
    })

    expect(result.current.resolvedTheme).toBe('dark')

    // Trigger OS theme change back to light
    act(() => {
      mockMatchMedia._trigger(false)
    })

    expect(result.current.resolvedTheme).toBe('light')
  })

  it('does not react to system changes when explicit theme is set', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    // Set explicit dark theme
    act(() => {
      result.current.setTheme('dark')
    })

    expect(result.current.resolvedTheme).toBe('dark')

    // Trigger OS theme change to light
    act(() => {
      mockMatchMedia._trigger(false)
    })

    // Should still be dark
    expect(result.current.resolvedTheme).toBe('dark')
  })

  it('handles localStorage errors gracefully', () => {
    mockStorage.setItem.mockImplementation(() => {
      throw new Error('Storage quota exceeded')
    })

    const { result } = renderHook(() => useTheme(), { wrapper })

    // Should not throw
    expect(() => {
      act(() => {
        result.current.setTheme('dark')
      })
    }).not.toThrow()

    // Theme should still be set in state
    expect(result.current.theme).toBe('dark')
  })

  it('handles localStorage getItem errors gracefully', () => {
    mockStorage.getItem.mockImplementation(() => {
      throw new Error('Storage access denied')
    })

    // Should not throw during initialization
    expect(() => {
      renderHook(() => useTheme(), { wrapper })
    }).not.toThrow()
  })

  it('ignores invalid theme values from localStorage', () => {
    mockStorage.getItem.mockReturnValue('invalid-theme')

    const { result } = renderHook(() => useTheme(), { wrapper })

    // Should fall back to system theme
    expect(result.current.theme).toBe('system')
  })

  it('adds transition class during theme change', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('dark')
    })

    expect(document.documentElement.classList.add).toHaveBeenCalledWith('theme-transitioning')
  })

  it('removes transition class after timeout', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('dark')
    })

    act(() => {
      vi.advanceTimersByTime(300)
    })

    expect(document.documentElement.classList.remove).toHaveBeenCalledWith('theme-transitioning')
  })

  it('throws error when useTheme is used outside provider', () => {
    expect(() => {
      renderHook(() => useTheme())
    }).toThrow('useTheme must be used within ThemeProvider')
  })

  it('has stable setTheme function reference', () => {
    const { result, rerender } = renderHook(() => useTheme(), { wrapper })

    const firstSetTheme = result.current.setTheme
    rerender()
    const secondSetTheme = result.current.setTheme

    expect(firstSetTheme).toBe(secondSetTheme)
  })

  it('updates resolvedTheme when switching from system to explicit theme', () => {
    mockMatchMedia = createMockMatchMedia(true) // OS prefers dark
    Object.defineProperty(window, 'matchMedia', {
      value: vi.fn(() => mockMatchMedia),
      writable: true,
    })

    const { result } = renderHook(() => useTheme(), { wrapper })

    // Start with system (resolves to dark)
    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('dark')

    // Switch to explicit light
    act(() => {
      result.current.setTheme('light')
    })

    expect(result.current.resolvedTheme).toBe('light')
  })

  it('cleans up event listeners on unmount', () => {
    const { unmount } = renderHook(() => useTheme(), { wrapper })

    unmount()

    expect(mockMatchMedia.removeEventListener).toHaveBeenCalled()
  })

  it('persists theme across multiple renders', () => {
    const { result, rerender } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('dark')
    })

    act(() => {
      vi.runAllTimers()
    })

    rerender()

    expect(result.current.theme).toBe('dark')
    expect(result.current.resolvedTheme).toBe('dark')
  })

  it('allows switching between all theme modes', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    // System -> Light
    act(() => {
      result.current.setTheme('light')
    })

    act(() => {
      vi.runAllTimers()
    })

    expect(result.current.theme).toBe('light')

    // Light -> Dark
    act(() => {
      result.current.setTheme('dark')
    })

    act(() => {
      vi.runAllTimers()
    })

    expect(result.current.theme).toBe('dark')

    // Dark -> System
    act(() => {
      result.current.setTheme('system')
    })

    act(() => {
      vi.runAllTimers()
    })

    expect(result.current.theme).toBe('system')
  })

  it('correctly resolves system theme after OS preference change', () => {
    const { result } = renderHook(() => useTheme(), { wrapper })

    act(() => {
      result.current.setTheme('system')
    })

    act(() => {
      vi.runAllTimers()
    })

    // OS switches to dark
    act(() => {
      mockMatchMedia._trigger(true)
    })

    act(() => {
      vi.runAllTimers()
    })

    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('dark')
  })
})
