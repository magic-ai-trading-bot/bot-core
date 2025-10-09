import { describe, it, expect, vi, afterEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { useMarketData } from '../../hooks/useMarketData'

describe('useMarketData', () => {
  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('initializes with default market data', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data).toEqual({
      price: 0,
      change24h: 0,
      volume: 0
    })
    expect(result.current.isLoading).toBe(true)
  })

  it('sets loading to false after timeout', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.isLoading).toBe(true)

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })
  })

  it('maintains market data structure while loading changes', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const initialData = result.current.data

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    // Data should remain unchanged
    expect(result.current.data).toEqual(initialData)
  })

  it('has correct market data structure', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data).toHaveProperty('price')
    expect(result.current.data).toHaveProperty('change24h')
    expect(result.current.data).toHaveProperty('volume')
  })

  it('has numeric price', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(typeof result.current.data.price).toBe('number')
    expect(result.current.data.price).toBe(0)
  })

  it('has numeric change24h', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(typeof result.current.data.change24h).toBe('number')
    expect(result.current.data.change24h).toBe(0)
  })

  it('has numeric volume', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(typeof result.current.data.volume).toBe('number')
    expect(result.current.data.volume).toBe(0)
  })

  it('returns both expected properties', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current).toHaveProperty('data')
    expect(result.current).toHaveProperty('isLoading')
  })

  it('does not have error property', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current).not.toHaveProperty('error')
  })

  it('does not re-trigger loading on re-render', async () => {
    const { result, rerender } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    rerender()

    // Should still be false after re-render
    expect(result.current.isLoading).toBe(false)
  })

  it('timeout executes and changes loading state', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    // Initially loading
    expect(result.current.isLoading).toBe(true)

    // After timeout, loading should be false
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })
  })

  it('maintains data integrity across timeout', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const beforeTimeout = result.current.data

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    const afterTimeout = result.current.data

    // Data should be exactly the same object
    expect(beforeTimeout).toBe(afterTimeout)
  })

  it('initializes all numeric fields to zero', async () => {
    const { result} = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.data.price).toBe(0)
    expect(result.current.data.change24h).toBe(0)
    expect(result.current.data.volume).toBe(0)
  })
})
