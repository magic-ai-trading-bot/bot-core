import { describe, it, expect, vi, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useTradingApi } from '../../hooks/useTradingApi'

describe('useTradingApi', () => {
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

  it('returns hook with exactly two properties', async () => {
    const { result } = renderHook(() => useTradingApi())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    const keys = Object.keys(result.current)
    expect(keys).toHaveLength(2)
    expect(keys).toContain('executeTrade')
    expect(keys).toContain('isLoading')
  })
})
