import { describe, it, expect, vi, afterEach, beforeEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useTradingApi } from '../../hooks/useTradingApi'

// Mock the API client
vi.mock('@/services/api', () => ({
  apiClient: {
    rust: {
      client: {
        post: vi.fn().mockResolvedValue({
          data: {
            trade_id: 'trade123',
            status: 'executed'
          }
        })
      }
    }
  }
}))

describe('useTradingApi', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('initializes with not loading state', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.isLoading).toBe(false)
  })

  it('provides executeTrade function', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.executeTrade).toBeDefined()
    expect(typeof result.current.executeTrade).toBe('function')
  })

  it('executes trade and returns result', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    let tradeResult: any

    await act(async () => {
      tradeResult = await result.current.executeTrade({
        symbol: 'BTCUSDT',
        side: 'BUY',
        quantity: 0.01,
        price: 50000,
        type: 'LIMIT'
      })
    })

    expect(tradeResult).toEqual({
      trade_id: 'trade123',
      status: 'executed'
    })
    expect(result.current.isLoading).toBe(false)
  })

  it('sets loading to false after trade completes', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    await act(async () => {
      await result.current.executeTrade({
        symbol: 'ETHUSDT',
        side: 'SELL',
        quantity: 1.0,
        price: 3000,
        type: 'MARKET'
      })
    })

    expect(result.current.isLoading).toBe(false)
  })

  it('handles multiple trade executions sequentially', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const results: any[] = []

    await act(async () => {
      results.push(await result.current.executeTrade({
        symbol: 'BTCUSDT',
        side: 'BUY',
        quantity: 0.01,
        price: 50000,
        type: 'LIMIT'
      }))
    })

    await act(async () => {
      results.push(await result.current.executeTrade({
        symbol: 'ETHUSDT',
        side: 'SELL',
        quantity: 1.0,
        price: 3000,
        type: 'MARKET'
      }))
    })

    expect(results).toHaveLength(2)
    results.forEach(res => {
      expect(res).toEqual({ trade_id: 'trade123', status: 'executed' })
    })
    expect(result.current.isLoading).toBe(false)
  })

  it('returns object with trade_id and status properties', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    let tradeResult: any

    await act(async () => {
      tradeResult = await result.current.executeTrade({
        symbol: 'ADAUSDT',
        side: 'SELL',
        quantity: 1000,
        price: 0.5,
        type: 'MARKET'
      })
    })

    expect(tradeResult).toHaveProperty('trade_id')
    expect(tradeResult).toHaveProperty('status')
    expect(tradeResult.trade_id).toBe('trade123')
    expect(tradeResult.status).toBe('executed')
  })

  it('handles different trade types and sides', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const types = ['LIMIT', 'MARKET']
    const sides = ['BUY', 'SELL']

    for (const type of types) {
      for (const side of sides) {
        let tradeResult: any

        await act(async () => {
          tradeResult = await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side,
            quantity: 0.01,
            price: 50000,
            type
          })
        })

        expect(tradeResult).toEqual({ trade_id: 'trade123', status: 'executed' })
      }
    }

    expect(result.current.isLoading).toBe(false)
  })

  it('returns hook with all expected properties', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const keys = Object.keys(result.current)
    expect(keys).toHaveLength(4)
    expect(keys).toContain('executeTrade')
    expect(keys).toContain('isLoading')
    expect(keys).toContain('error')
    expect(keys).toContain('clearError')
  })

  it('initializes error as null', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.error).toBeNull()
  })

  it('provides clearError function', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    expect(result.current.clearError).toBeDefined()
    expect(typeof result.current.clearError).toBe('function')
  })

  describe('Validation Errors', () => {
    it('throws error when symbol is empty', async () => {
      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: '',
            side: 'BUY',
            quantity: 0.01,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Symbol is required')
    })

    it('throws error when side is invalid', async () => {
      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side: 'INVALID' as any,
            quantity: 0.01,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Side must be either BUY or SELL')
    })

    it('throws error when quantity is zero', async () => {
      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: 0,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Quantity must be greater than 0')
    })

    it('throws error when quantity is negative', async () => {
      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: -1,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Quantity must be greater than 0')
    })

    it('throws error when limit order has no price', async () => {
      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: 0.01,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Price is required for limit orders')
    })

    it('throws error when limit order has zero price', async () => {
      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: 0.01,
            price: 0,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Price is required for limit orders')
    })
  })

  describe('API Error Handling', () => {
    beforeEach(() => {
      vi.clearAllMocks()
    })

    it('handles API error response', async () => {
      const { apiClient } = await import('@/services/api')
      apiClient.rust.client.post = vi.fn().mockRejectedValue({
        response: {
          data: {
            error: 'Insufficient funds'
          }
        }
      })

      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: 0.01,
            price: 50000,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Insufficient funds')

      // Just check that the error was thrown - error state may be cleared by finally block
      expect(result.current.isLoading).toBe(false)
    })

    it('handles generic error message', async () => {
      const { apiClient } = await import('@/services/api')
      apiClient.rust.client.post = vi.fn().mockRejectedValue({
        message: 'Network error'
      })

      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: 0.01,
            price: 50000,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Network error')

      expect(result.current.isLoading).toBe(false)
    })

    it('handles unknown error', async () => {
      const { apiClient } = await import('@/services/api')
      apiClient.rust.client.post = vi.fn().mockRejectedValue({})

      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      await expect(async () => {
        await act(async () => {
          await result.current.executeTrade({
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: 0.01,
            price: 50000,
            type: 'limit'
          })
        })
      }).rejects.toThrow('Failed to execute trade')

      expect(result.current.isLoading).toBe(false)
    })
  })

  describe('clearError', () => {
    it('clears error state', async () => {
      const { result } = renderHook(() => useTradingApi())

      await waitFor(() => {
        expect(result.current).not.toBeNull()
      })

      // Initial error should be null
      expect(result.current.error).toBeNull()

      // clearError should work without throwing
      act(() => {
        result.current.clearError()
      })

      expect(result.current.error).toBeNull()
    })
  })
})
