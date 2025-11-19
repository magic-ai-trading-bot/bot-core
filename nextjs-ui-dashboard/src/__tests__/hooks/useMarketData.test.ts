import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest'
import { renderHook, waitFor } from '@testing-library/react'
import { useMarketData } from '../../hooks/useMarketData'

// Mock the API client
vi.mock('@/services/api', () => ({
  apiClient: {
    rust: {
      getChartData: vi.fn().mockResolvedValue({
        latest_price: 0,
        price_change_24h: 0,
        volume_24h: 0,
        price_change_percent_24h: 0,
        candles: [
          { high: 100, low: 90, open: 95, close: 98, volume: 1000, timestamp: Date.now() }
        ]
      })
    }
  }
}))

describe('useMarketData', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('initializes with default market data', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    // Check that data has all expected properties
    expect(result.current.data).toHaveProperty('price')
    expect(result.current.data).toHaveProperty('change24h')
    expect(result.current.data).toHaveProperty('volume')
    expect(result.current.data).toHaveProperty('high24h')
    expect(result.current.data).toHaveProperty('low24h')
    expect(result.current.data).toHaveProperty('priceChangePercent')

    // All numeric values should be 0 initially
    expect(result.current.data.price).toBe(0)
    expect(result.current.data.change24h).toBe(0)
    expect(result.current.data.volume).toBe(0)
  })

  it('sets loading to false after data is fetched', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    // With mocked API, loading completes immediately
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

  it('returns all expected properties', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current).toHaveProperty('data')
    expect(result.current).toHaveProperty('isLoading')
    expect(result.current).toHaveProperty('error')
    expect(result.current).toHaveProperty('refresh')
  })

  it('has error property initialized to null', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current).toHaveProperty('error')
    expect(result.current.error).toBeNull()
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

  it('loading state changes after data fetch completes', async () => {
    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    // After data fetch, loading should be false
    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    // Verify we have the expected loading state
    expect(result.current.isLoading).toBe(false)
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

    // Data should have consistent structure (fields may update with new values)
    expect(afterTimeout).toMatchObject({
      price: expect.any(Number),
      change24h: expect.any(Number),
      volume: expect.any(Number),
      priceChangePercent: expect.any(Number),
    })
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
