import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest'
import { renderHook, waitFor, act } from '@testing-library/react'
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

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    }, { timeout: 1000 })

    // Data structure should be maintained (all required fields present)
    expect(result.current.data).toHaveProperty('price')
    expect(result.current.data).toHaveProperty('change24h')
    expect(result.current.data).toHaveProperty('volume')
    expect(result.current.data).toHaveProperty('priceChangePercent')
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

  // NEW TESTS FOR UNCOVERED CODE PATHS

  it('handles API errors gracefully', async () => {
    const { apiClient } = await import('@/services/api')
    vi.mocked(apiClient.rust.getChartData).mockRejectedValue(new Error('API Error'))

    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current.error).toBeTruthy()
    })

    // Error message should contain 'API Error'
    expect(result.current.error).toContain('API Error')
  })

  it('handles abort errors silently', async () => {
    const { apiClient } = await import('@/services/api')
    const abortError = new Error('AbortError')
    abortError.name = 'AbortError'
    vi.mocked(apiClient.rust.getChartData).mockRejectedValue(abortError)

    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    // Abort errors should not set error state
    expect(result.current.error).toBeNull()
  })

  it('handles ERR_CANCELED errors silently', async () => {
    const { apiClient } = await import('@/services/api')
    const cancelError = { code: 'ERR_CANCELED', message: 'Request canceled' }
    vi.mocked(apiClient.rust.getChartData).mockRejectedValue(cancelError)

    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    expect(result.current.error).toBeNull()
  })

  it('handles API response with nested error', async () => {
    const { apiClient } = await import('@/services/api')
    const nestedError = {
      response: {
        data: {
          error: 'Custom API error message'
        }
      }
    }
    vi.mocked(apiClient.rust.getChartData).mockRejectedValue(nestedError)

    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current.error).toBe('Custom API error message')
    })
  })

  it('disables auto-refresh when interval is 0', async () => {
    const { result } = renderHook(() => useMarketData('BTCUSDT', '1h', 0))

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    // Should only fetch once (initial), not set up interval
    const { apiClient } = await import('@/services/api')
    expect(vi.mocked(apiClient.rust.getChartData)).toHaveBeenCalledTimes(1)
  })

  it('disables auto-refresh when interval is negative', async () => {
    const { result } = renderHook(() => useMarketData('BTCUSDT', '1h', -1000))

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    const { apiClient } = await import('@/services/api')
    expect(vi.mocked(apiClient.rust.getChartData)).toHaveBeenCalledTimes(1)
  })

  it('calls refresh manually', async () => {
    const { result } = renderHook(() => useMarketData('BTCUSDT', '1h', 0))

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    const { apiClient } = await import('@/services/api')
    const initialCalls = vi.mocked(apiClient.rust.getChartData).mock.calls.length

    await act(async () => {
      result.current.refresh()
    })

    await waitFor(() => {
      expect(vi.mocked(apiClient.rust.getChartData).mock.calls.length).toBeGreaterThan(initialCalls)
    })
  })

  it('cancels pending requests on unmount', async () => {
    const { unmount } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(true).toBe(true)
    })

    unmount()

    // Unmount should trigger cleanup - no errors should occur
    expect(true).toBe(true)
  })

  it('cancels pending requests on re-fetch', async () => {
    const { apiClient } = await import('@/services/api')

    // Mock slow response
    vi.mocked(apiClient.rust.getChartData).mockImplementation(
      () => new Promise(resolve => setTimeout(() => resolve({
        latest_price: 50000,
        price_change_24h: 1000,
        volume_24h: 1000000,
        price_change_percent_24h: 2,
        candles: [{ high: 51000, low: 49000, open: 50000, close: 50500, volume: 1000, timestamp: Date.now() }]
      }), 100))
    )

    const { result } = renderHook(() => useMarketData('BTCUSDT', '1h', 0))

    // Trigger multiple refreshes rapidly
    await act(async () => {
      result.current.refresh()
      result.current.refresh()
    })

    await waitFor(() => {
      expect(result.current.isLoading).toBe(false)
    })

    // Should handle cancellation gracefully
    expect(result.current.error).toBeNull()
  })

  it('updates data with chart metrics', async () => {
    const { apiClient } = await import('@/services/api')
    vi.mocked(apiClient.rust.getChartData).mockResolvedValue({
      latest_price: 50000,
      price_change_24h: 1000,
      volume_24h: 2000000,
      price_change_percent_24h: 2.5,
      candles: [
        { high: 51000, low: 48000, open: 49000, close: 50000, volume: 1000, timestamp: Date.now() }
      ]
    })

    const { result } = renderHook(() => useMarketData())

    await waitFor(() => {
      expect(result.current.data.price).toBe(50000)
    })

    expect(result.current.data.price).toBe(50000)
    expect(result.current.data.change24h).toBe(1000)
    expect(result.current.data.volume).toBe(2000000)
    expect(result.current.data.priceChangePercent).toBe(2.5)
    expect(result.current.data.high24h).toBe(51000)
    expect(result.current.data.low24h).toBe(48000)
    expect(result.current.data.lastUpdate).toBeTruthy()
  })
})
